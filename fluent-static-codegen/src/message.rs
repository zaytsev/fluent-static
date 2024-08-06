use std::collections::BTreeSet;

use convert_case::{Case, Casing};
use fluent_syntax::ast;
use proc_macro2::{Literal, TokenStream};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub name: String,
    pub attribute_name: Option<String>,
    pub vars: Vec<Var>,
    pub attrs: Option<Vec<Message>>,
}

impl Message {
    pub fn parse<T: AsRef<str>>(message: &ast::Message<T>) -> Result<Self, Error> {
        let name = message.id.name.as_ref().to_string();
        let vars = extract_variables(message.value.as_ref())?;
        let attrs = extract_attributes(&name, &message.attributes)?;
        Ok(Self {
            name,
            vars,
            attrs,
            attribute_name: None,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn function_ident(&self) -> Ident {
        let base_name = self.name().to_case(Case::Snake);

        self.attribute_name.as_ref().map_or_else(
            || format_ident!("{}", base_name),
            |attr| format_ident!("{}_{}", base_name, attr.to_case(Case::Snake)),
        )
    }

    pub fn message_name_literal(&self) -> Literal {
        Literal::string(&self.name)
    }

    pub fn maybe_attribute_name_literal(&self) -> TokenStream {
        self.attribute_name.as_ref().map_or_else(
            || quote::quote!(None),
            |name| {
                let name_literal = Literal::string(name);
                quote::quote!(Some(#name_literal))
            },
        )
    }

    pub fn vars(&self) -> Vec<&Var> {
        self.vars.iter().collect()
    }

    pub fn has_attrs(&self) -> bool {
        self.attrs.is_some()
    }

    pub fn attrs(&self) -> Vec<&Message> {
        if let Some(attrs) = self.attrs.as_ref() {
            attrs.iter().collect()
        } else {
            Vec::default()
        }
    }

    pub(crate) fn normalize(&self) -> NormalizedMessage {
        NormalizedMessage {
            name: self.name.clone(),
            attribute_name: self.attribute_name.clone(),
            vars: self.vars.iter().cloned().collect(),
            attrs: self
                .attrs
                .as_ref()
                .map(|attrs| attrs.iter().map(Message::normalize).collect()),
        }
    }
}

impl<T: AsRef<str>> TryFrom<&ast::Message<T>> for Message {
    type Error = Error;

    fn try_from(value: &ast::Message<T>) -> Result<Self, Self::Error> {
        Message::parse(value)
    }
}

fn extract_attributes<T: AsRef<str>>(
    parent: &str,
    attributes: &[ast::Attribute<T>],
) -> Result<Option<Vec<Message>>, Error> {
    if !attributes.is_empty() {
        let mut result = Vec::new();
        for attr in attributes {
            let msg = Message {
                name: parent.to_string(),
                vars: extract_variables(Some(&attr.value))?,
                attrs: None,
                attribute_name: Some(attr.id.name.as_ref().to_string()),
            };
            result.push(msg);
        }
        Ok(Some(result))
    } else {
        Ok(None)
    }
}

fn extract_variables<T: AsRef<str>>(value: Option<&ast::Pattern<T>>) -> Result<Vec<Var>, Error> {
    let mut result = Vec::new();
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NormalizedMessage {
    pub name: String,
    pub attribute_name: Option<String>,
    pub vars: BTreeSet<Var>,
    pub attrs: Option<BTreeSet<NormalizedMessage>>,
}

#[cfg(test)]
mod tests {
    use fluent_syntax::{ast, parser};
    use quote::format_ident;

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
        assert_eq!("foo", vars.next().unwrap().name);
        assert_eq!("baz", vars.next().unwrap().name);
    }

    #[test]
    fn message_variables_order() {
        let resource = "test = foo { $b } { $a }";
        let msg = parse(resource).into_iter().next().unwrap();

        let msg = Message::parse(&msg).unwrap();

        assert_eq!("test", msg.name());
        assert_eq!(2, msg.vars().len());
        let mut vars = msg.vars().into_iter();
        assert_eq!("b", &vars.next().unwrap().name);
        assert_eq!("a", &vars.next().unwrap().name);
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
    // TODO: how to pass arguments to term/message reference?
    fn message_with_parameterized_term_reference() {
        let resource = "test = foo { -test1 }\n-test1 = foo { $bar }";
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
        let mut vars = msg.vars().into_iter();
        assert_eq!("baz", vars.next().unwrap().name);
    }

    #[test]
    fn message_with_function_with_named_args() {
        let resource = "test = foo { FOO($baz, bar: \"foo\") }\n-test1 = foo bar";
        let msg = parse(resource).into_iter().next().unwrap();

        let msg = Message::parse(&msg).unwrap();

        assert_eq!("test", msg.name());
        assert_eq!(1, msg.vars().len());
        let mut vars = msg.vars().into_iter();
        assert_eq!("baz", vars.next().unwrap().name);
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
    fn message_with_simple_attributes() {
        let resource =
            "test-attrs = foo { $test1 }\n  .attr2=bar\n  .attr1=baz\ntest1 = foo { $bar }";
        let msg = parse(resource).into_iter().next().unwrap();

        let msg = Message::parse(&msg).unwrap();

        assert_eq!("test-attrs", msg.name());
        assert_eq!(1, msg.vars().len());

        let mut attrs = msg.attrs().into_iter();
        let attr = attrs.next().unwrap();
        assert_eq!("test-attrs", attr.name());
        assert!(attr.vars().is_empty());
        assert_eq!(format_ident!("test_attrs_attr_2"), attr.function_ident());
        let attr = attrs.next().unwrap();
        assert_eq!("test-attrs", attr.name());
        assert!(attr.vars().is_empty());
        assert_eq!(format_ident!("test_attrs_attr_1"), attr.function_ident());
    }

    #[test]
    fn message_with_variable_attributes() {
        let resource =
            "test-attrs = foo { $test1 }\n  .attr2=bar\n  .attr1=attr { $baz }\ntest1 = foo { $bar }";
        let msg = parse(resource).into_iter().next().unwrap();

        let msg = Message::parse(&msg).unwrap();

        assert_eq!("test-attrs", msg.name());
        assert_eq!(1, msg.vars().len());

        let mut attrs = msg.attrs().into_iter();
        let attr = attrs.next().unwrap();
        assert_eq!("test-attrs", attr.name());
        assert!(attr.vars().is_empty());
        let attr = attrs.next().unwrap();
        assert_eq!("test-attrs", attr.name());
        assert_eq!(1, attr.vars().len());
        assert_eq!("baz", attr.vars().into_iter().next().unwrap().name);
    }
}
