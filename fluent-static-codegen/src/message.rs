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
mod tests {}
