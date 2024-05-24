use std::collections::BTreeSet;

use convert_case::{Case, Casing};
use fluent_syntax::ast;
use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};
use syn::Ident;

use crate::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Var {
    name: String,
}

impl Var {
    pub fn new(name: String) -> Self {
        Self { name }
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
    name: String,
    vars: BTreeSet<Var>,
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

    pub fn function_code(&self, delegate_fn: &Ident) -> TokenStream {
        if self.vars.is_empty() {
            self.no_args_function_code(delegate_fn)
        } else {
            self.args_function_code(delegate_fn)
        }
    }

    fn no_args_function_code(&self, delegate_fn: &Ident) -> TokenStream {
        let function_ident = self.function_ident();
        let message_name_literal = self.message_name_literal();
        quote! {
            pub fn #function_ident<'b>(lang_id: impl AsRef<str>) -> Result<Message<'b>, FluentError> {
                #delegate_fn(lang_id.as_ref(), #message_name_literal, None)
            }
        }
    }

    fn args_function_code(&self, delegate_fn: &Ident) -> TokenStream {
        let function_ident = self.function_ident();
        let message_name_literal = self.message_name_literal();
        let function_args = self.function_args();
        let capacity = Literal::usize_unsuffixed(self.vars.len());
        let fluent_args = self.fluent_args();
        quote! {
            pub fn #function_ident<'a, 'b>(lang_id: impl AsRef<str>, #(#function_args: impl Into<FluentValue<'a>>),*) -> Result<Message<'b>, FluentError> {
                let mut args = FluentArgs::with_capacity(#capacity);
                #(#fluent_args)*
                #delegate_fn(lang_id.as_ref(), #message_name_literal, Some(&args))
            }
        }
    }

    fn function_args(&self) -> Vec<Ident> {
        self.vars.iter().map(Var::ident).collect()
    }

    fn fluent_args(&self) -> Vec<TokenStream> {
        self.vars
            .iter()
            .map(|var| {
                let name = var.literal();
                let value = var.ident();
                quote! {
                    args.set(#name, #value);
                }
            })
            .collect()
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
                match expression {
                    ast::Expression::Select { selector, variants } => {
                        if let Some(var) = parse_expression(selector)? {
                            result.insert(Var::new(var));
                        }
                        for variant in variants {
                            result.extend(extract_variables(Some(&variant.value))?.into_iter())
                        }
                    }
                    ast::Expression::Inline(e) => {
                        if let Some(var) = parse_expression(e)? {
                            result.insert(Var::new(var));
                        }
                    }
                }
            }
        }
    }
    Ok(result)
}

fn parse_expression<T: AsRef<str>>(
    inline_expression: &ast::InlineExpression<T>,
) -> Result<Option<String>, Error> {
    match inline_expression {
        ast::InlineExpression::StringLiteral { .. } => Ok(None),
        ast::InlineExpression::NumberLiteral { .. } => Ok(None),
        ast::InlineExpression::FunctionReference { id, .. } => Err(Error::UnsupportedFeature {
            feature: "function reference".to_string(),
            id: id.name.as_ref().to_string(),
        }),
        ast::InlineExpression::MessageReference { id, .. } => Err(Error::UnsupportedFeature {
            feature: "message reference".to_string(),
            id: id.name.as_ref().to_string(),
        }),
        ast::InlineExpression::TermReference { id, .. } => Err(Error::UnsupportedFeature {
            feature: "term reference".to_string(),
            id: id.name.as_ref().to_string(),
        }),

        ast::InlineExpression::VariableReference { id } => Ok(Some(id.name.as_ref().to_string())),
        ast::InlineExpression::Placeable { .. } => Err(Error::UnsupportedFeature {
            feature: "nested expression".to_string(),
            // TODO better diagnostics
            id: "".to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use quote::{format_ident, quote};

    use crate::message::Var;

    use super::Message;

    #[test]
    pub fn message_no_args_code() {
        let expected = quote! {
            pub fn hello<'b>(lang_id: impl AsRef<str>) -> Result<Message<'b>, FluentError> {
                format_message(lang_id.as_ref(), "hello", None)
            }
        };

        let delegate_fn = format_ident!("{}", "format_message");

        let actual = Message {
            name: "hello".to_string(),
            vars: BTreeSet::new(),
        }
        .function_code(&delegate_fn);

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    pub fn message_with_args_code() {
        let expected = quote! {
            pub fn hello<'a, 'b>(lang_id: impl AsRef<str>, first: impl Into<FluentValue<'a>>, second: impl Into<FluentValue<'a>>) -> Result<Message<'b>, FluentError> {
                let mut args = FluentArgs::with_capacity(2);
                args.set("first", first);
                args.set("second", second);
                format_message(lang_id.as_ref(), "hello", Some(&args))
            }
        };

        let delegate_fn = format_ident!("{}", "format_message");

        let actual = Message {
            name: "hello".to_string(),
            vars: vec!["second", "first"]
                .iter()
                .map(|v| Var::new(v.to_string()))
                .collect(),
        }
        .function_code(&delegate_fn);

        assert_eq!(actual.to_string(), expected.to_string());
    }
}
