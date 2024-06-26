use std::collections::BTreeSet;

use convert_case::{Case, Casing};
use fluent_syntax::ast;
use proc_macro2::Literal;
use quote::format_ident;
use syn::Ident;

use crate::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Var {
    pub(crate) name: String,
}

impl Var {
    pub fn new(name: impl AsRef<str>) -> Self {
        Self {
            name: name.as_ref().to_string(),
        }
    }
    pub fn ident(&self) -> Ident {
        format_ident!("{}", self.name.to_case(Case::Snake))
    }

    pub fn literal(&self) -> Literal {
        Literal::string(&self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Message {
    pub(crate) name: String,
    pub(crate) vars: BTreeSet<Var>,
}

impl Message {
    pub fn parse<T: AsRef<str>>(message: &ast::Message<T>) -> Result<Self, Error> {
        let name = message.id.name.as_ref().to_string();
        let vars = extract_variables(message.value.as_ref())?;
        Ok(Self { name, vars })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn function_ident(&self) -> Ident {
        format_ident!("{}", self.name.to_case(Case::Snake))
    }

    pub fn message_name_literal(&self) -> Literal {
        Literal::string(&self.name)
    }

    pub fn vars(&self) -> BTreeSet<&Var> {
        self.vars.iter().collect()
    }
}

impl<T: AsRef<str>> TryFrom<&ast::Message<T>> for Message {
    type Error = Error;

    fn try_from(value: &ast::Message<T>) -> Result<Self, Self::Error> {
        Message::parse(value)
    }
}

fn extract_variables<T: AsRef<str>>(
    value: Option<&ast::Pattern<T>>,
) -> Result<BTreeSet<Var>, Error> {
    let mut result = BTreeSet::new();
    if let Some(pattern) = value {
        for element in pattern.elements.iter() {
            if let ast::PatternElement::Placeable { expression } = element {
                result.extend(parse_expression(expression)?)
            }
        }
    }
    Ok(result)
}

fn parse_expression<T: AsRef<str>>(expression: &ast::Expression<T>) -> Result<Vec<Var>, Error> {
    match expression {
        ast::Expression::Select { selector, variants } => {
            let mut result = vec![];
            result.extend(parse_inline_expression(selector)?);
            for variant in variants {
                result.extend(extract_variables(Some(&variant.value))?.into_iter());
            }
            Ok(result)
        }
        ast::Expression::Inline(e) => parse_inline_expression(e),
    }
}

fn parse_inline_expression<T: AsRef<str>>(
    inline_expression: &ast::InlineExpression<T>,
) -> Result<Vec<Var>, Error> {
    match inline_expression {
        ast::InlineExpression::StringLiteral { .. }
        | ast::InlineExpression::NumberLiteral { .. }
        | ast::InlineExpression::MessageReference { .. } => Ok(Vec::default()),
        ast::InlineExpression::FunctionReference { arguments, .. } => {
            parse_call_arguments(Some(arguments))
        }

        ast::InlineExpression::TermReference { arguments, .. } => {
            parse_call_arguments(arguments.as_ref())
        }

        ast::InlineExpression::VariableReference { id } => Ok(vec![Var::new(&id.name)]),
        ast::InlineExpression::Placeable { expression } => parse_expression(expression),
    }
}

fn parse_call_arguments<T: AsRef<str>>(
    arguments: Option<&ast::CallArguments<T>>,
) -> Result<Vec<Var>, Error> {
    if let Some(arguments) = arguments {
        let mut result = vec![];
        for expr in &arguments.positional {
            result.extend(parse_inline_expression(expr)?)
        }
        for ast::NamedArgument { value, .. } in &arguments.named {
            result.extend(parse_inline_expression(value)?)
        }

        Ok(result)
    } else {
        Ok(Vec::default())
    }
}

#[cfg(test)]
mod tests {
    use fluent_syntax::{ast, parser};

    use super::Message;

    fn parse(content: &'static str) -> Vec<ast::Message<&'static str>> {
        let resource = parser::parse(content).unwrap();

        resource
            .body
            .into_iter()
            .filter_map(|entry| {
                if let ast::Entry::Message(message) = entry {
                    Some(message)
                } else {
                    None
                }
            })
            .collect::<Vec<ast::Message<_>>>()
    }

    #[test]
    fn multiline_message() {
        let resource = "test = foo\n    bar\n\ntest1 = foo bar";
        let msg = parse(resource).into_iter().next().unwrap();

        let msg = Message::parse(&msg).unwrap();

        assert_eq!("test", msg.name())
    }

    #[test]
    fn message_with_selector() {
        let resource = r#"
test = 
  {
    $foo ->
       [one] -> bar
       *[other] -> { $baz }
  }
"#;
        let msg = parse(resource).into_iter().next().unwrap();

        let msg = Message::parse(&msg).unwrap();

        assert_eq!("test", msg.name());
        assert_eq!(2, msg.vars().len());
        let mut vars = msg.vars().into_iter();
        assert_eq!("baz", vars.next().unwrap().name);
        assert_eq!("foo", vars.next().unwrap().name);
    }

    #[test]
    fn message_with_term_reference() {
        let resource = "test = foo { -test1 }\n-test1 = foo bar";
        let msg = parse(resource).into_iter().next().unwrap();

        let msg = Message::parse(&msg).unwrap();

        assert_eq!("test", msg.name());
        assert_eq!(0, msg.vars().len());
    }

    #[test]
    fn message_with_simple_function() {
        let resource = "test = foo { FOO($baz) }\n-test1 = foo bar";
        let msg = parse(resource).into_iter().next().unwrap();

        let msg = Message::parse(&msg).unwrap();

        assert_eq!("test", msg.name());
        assert_eq!(1, msg.vars().len());
    }

    #[test]
    fn message_with_function_with_named_args() {
        let resource = "test = foo { FOO($baz, bar: \"foo\") }\n-test1 = foo bar";
        let msg = parse(resource).into_iter().next().unwrap();

        let msg = Message::parse(&msg).unwrap();

        assert_eq!("test", msg.name());
        assert_eq!(1, msg.vars().len());
    }

    #[test]
    // TODO: how to pass arguments to term/message reference?
    fn message_with_message_reference() {
        let resource = "test = foo { test1 }\ntest1 = foo { $bar }";
        let msg = parse(resource).into_iter().next().unwrap();

        let msg = Message::parse(&msg).unwrap();

        assert_eq!("test", msg.name());
        assert_eq!(0, msg.vars().len());
    }

    #[test]
    // TODO: how to pass arguments to term/message reference?
    fn message_with_parameterized_term_reference() {
        let resource = "test = foo { -test1 }\n-test1 = foo { $bar }";
        let msg = parse(resource).into_iter().next().unwrap();

        let msg = Message::parse(&msg).unwrap();

        assert_eq!("test", msg.name());
        assert_eq!(0, msg.vars().len());
    }
}
