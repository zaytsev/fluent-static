use std::{
    collections::{BTreeMap, BTreeSet},
    str::FromStr,
};

use convert_case::{Case, Casing};
use fluent_bundle::FluentValue;
use fluent_syntax::ast;
use intl_pluralrules::PluralCategory;
use proc_macro2::{Ident, Literal, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use unic_langid::LanguageIdentifier;

use crate::{
    ast::{Node, Visitor},
    Error,
};

pub struct LanguageBuilder {
    pub language_id: String,
    pub prefix: String,
    pub pending_fns: Vec<Callable>,
    pub registered_fns: BTreeSet<Ident>,
    pub registered_message_fns: BTreeMap<String, Callable>,
    pub pending_message_refs: BTreeMap<String, String>,
    pub pending_term_refs: BTreeMap<String, String>,
}

impl LanguageBuilder {
    pub fn new(language_id: &str) -> Self {
        Self {
            language_id: language_id.to_string(),
            prefix: language_id.to_case(Case::Snake),
            pending_fns: Vec::new(),
            registered_fns: BTreeSet::new(),
            registered_message_fns: BTreeMap::new(),
            pending_message_refs: BTreeMap::new(),
            pending_term_refs: BTreeMap::new(),
        }
    }

    fn push_message<S: ToString>(&mut self, message: &ast::Message<S>) {
        let id = &message.id;
        self.pending_fns.push(Callable::new(
            id.name.to_string(),
            self.message_fn_ident(message),
        ));
    }

    fn push_term<S: ToString>(&mut self, term: &ast::Term<S>) {
        self.pending_fns.push(Callable::new(
            term.id.name.to_string(),
            self.term_fn_ident(term),
        ));
    }

    fn push_attribute<S: ToString>(&mut self, attribute: &ast::Attribute<S>) -> Result<(), Error> {
        if let Some(parent) = self.pending_fns.last() {
            self.pending_fns.push(Callable::new(
                format!("{}.{}", parent.id.as_str(), attribute.id.name.to_string()),
                self.attribute_fn_ident(&parent.fn_ident, attribute),
            ));
            Ok(())
        } else {
            Err(Error::UnexpectedContextState)
        }
    }

    fn register_pending_fn(&mut self, body: TokenStream2) -> Result<TokenStream2, Error> {
        let f = self
            .pending_fns
            .pop()
            .ok_or(Error::UnexpectedContextState)?;

        let result = f.to_tokens(body);

        if !self.registered_fns.insert(f.fn_ident.clone()) {
            Err(Error::DuplicateEntryId(f.fn_ident.to_string()))
        } else {
            self.registered_message_fns.insert(f.ext_id(), f.clone());
            Ok(result)
        }
    }

    fn append_var<S: ToString>(&mut self, id: &ast::Identifier<S>) -> Result<Ident, Error> {
        if let Some(item) = self.pending_fns.last_mut() {
            let var = Var::new(id);
            let ident = var.var_ident.clone();
            item.add_var(var);
            Ok(ident)
        } else {
            Err(Error::UnexpectedContextState)
        }
    }

    fn current_context(&self) -> Option<&Callable> {
        self.pending_fns.last()
    }

    fn message_fn_ident<S: ToString>(&self, message: &ast::Message<S>) -> Ident {
        format_ident!(
            "{}_{}",
            self.prefix,
            message.id.name.to_string().to_case(Case::Snake)
        )
    }

    fn term_fn_ident<S: ToString>(&self, term: &ast::Term<S>) -> Ident {
        format_ident!(
            "{}_term_{}",
            self.prefix,
            term.id.name.to_string().to_case(Case::Snake)
        )
    }

    fn attribute_fn_ident<S: ToString>(
        &self,
        parent: &Ident,
        attribute: &ast::Attribute<S>,
    ) -> Ident {
        format_ident!(
            "{}_{}",
            parent,
            attribute.id.name.to_string().to_case(Case::Snake)
        )
    }
}

#[derive(Debug, Clone)]
pub struct Callable {
    pub id: String,
    pub fn_ident: Ident,
    pub vars: Vec<Var>,
    pub var_idents: BTreeSet<Ident>,
    pub unique_vars: BTreeSet<Var>,
}

impl Callable {
    fn new(id: String, fn_ident: Ident) -> Self {
        Self {
            id,
            fn_ident,
            vars: Vec::new(),
            var_idents: BTreeSet::new(),
            unique_vars: BTreeSet::new(),
        }
    }

    fn add_var(&mut self, var: Var) {
        if !self.unique_vars.contains(&var) {
            self.unique_vars.insert(var.clone());
            self.var_idents.insert(var.var_ident.clone());
            self.vars.push(var);
        }
    }

    fn ext_id(&self) -> String {
        if self.unique_vars.is_empty() {
            self.id.clone()
        } else {
            let vars = self
                .unique_vars
                .iter()
                .map(|var| var.var_name.as_str())
                .collect::<Vec<&str>>()
                .join(",");

            format!("{}={}", self.id, vars)
        }
    }

    fn to_tokens(&self, body: TokenStream2) -> TokenStream2 {
        let fn_ident = &self.fn_ident;
        let var_idents = &self.var_idents;

        let fn_generics = if var_idents.is_empty() {
            quote! {
                <W: ::std::fmt::Write>
            }
        } else {
            quote! {
                <'a, W: ::std::fmt::Write>
            }
        };

        quote! {
            #[inline]
            fn #fn_ident #fn_generics(
                &self,
                out: &mut W,
                #(#var_idents: ::fluent_static::fluent_bundle::FluentValue<'a>),*
            ) -> ::std::fmt::Result {
                #body
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Var {
    pub var_ident: Ident,
    pub var_name: String,
}

impl Var {
    fn new<S: ToString>(id: &ast::Identifier<S>) -> Self {
        let var_name = id.name.to_string();
        let var_ident = format_ident!("{}", var_name.to_case(Case::Snake));
        Self {
            var_ident,
            var_name,
        }
    }
}

impl<S: ToString> Visitor<S> for LanguageBuilder {
    type Output = Result<TokenStream2, Error>;

    fn visit_resource(&mut self, resource: &ast::Resource<S>) -> Self::Output {
        resource
            .body
            .iter()
            .try_fold(TokenStream2::new(), |mut result, entry| {
                let tokens = entry.accept(self)?;
                result.extend(tokens);
                Ok(result)
            })
    }

    fn visit_entry(&mut self, entry: &ast::Entry<S>) -> Self::Output {
        match entry {
            ast::Entry::Message(message) => message.accept(self),
            ast::Entry::Term(term) => term.accept(self),
            _ => Ok(TokenStream2::new()),
        }
    }

    fn visit_message(&mut self, message: &ast::Message<S>) -> Self::Output {
        self.push_message(message);
        let body = message
            .value
            .as_ref()
            .map(|pattern| pattern.accept(self))
            .unwrap_or_else(|| Ok(TokenStream2::new()))?;

        let attribute_fns = message
            .attributes
            .iter()
            .map(|attribute| attribute.accept(self))
            .collect::<Result<Vec<TokenStream2>, Error>>()?;

        let message_fn = self.register_pending_fn(body)?;

        Ok(quote! {
            #message_fn
            #(#attribute_fns)*
        })
    }

    fn visit_term(&mut self, term: &ast::Term<S>) -> Self::Output {
        self.push_term(term);
        let body = term.value.accept(self)?;

        let attribute_fns = term
            .attributes
            .iter()
            .map(|attribute| attribute.accept(self))
            .collect::<Result<Vec<TokenStream2>, Error>>()?;

        let term_fn = self.register_pending_fn(body)?;

        Ok(quote! {
            #term_fn
            #(#attribute_fns)*
        })
    }

    fn visit_pattern(&mut self, pattern: &ast::Pattern<S>) -> Self::Output {
        let elements: Vec<TokenStream2> = pattern
            .elements
            .iter()
            .map(|element| element.accept(self))
            .collect::<Result<Vec<TokenStream2>, Error>>()?;
        Ok(quote! {
            #(#elements)*
        })
    }

    fn visit_pattern_element(&mut self, element: &ast::PatternElement<S>) -> Self::Output {
        match element {
            ast::PatternElement::TextElement { value } => {
                let text = Literal::string(value.to_string().as_str());
                Ok(quote! {
                    out.write_str(#text)?;
                })
            }
            ast::PatternElement::Placeable { expression } => expression.accept(self),
        }
    }

    fn visit_attribute(&mut self, attribute: &ast::Attribute<S>) -> Self::Output {
        self.push_attribute(attribute)?;
        let body = attribute.value.accept(self)?;
        self.register_pending_fn(body)
    }

    fn visit_identifier(&mut self, _identifier: &ast::Identifier<S>) -> Self::Output {
        unimplemented!()
    }

    fn visit_variant(&mut self, variant: &ast::Variant<S>) -> Self::Output {
        let match_key = if variant.default {
            quote! {
                _
            }
        } else {
            variant.key.accept(self)?
        };

        let body = variant.value.accept(self)?;

        Ok(quote! {
            #match_key => {
                #body
            }
        })
    }

    fn visit_variant_key(&mut self, key: &ast::VariantKey<S>) -> Self::Output {
        Ok(match key {
            ast::VariantKey::Identifier { name } => {
                let name = name.to_string();
                let lit = Literal::string(&name);
                if get_plural_category(key).is_some() {
                    let category_ident = format_ident!("{}", &name.to_uppercase());
                    quote! {
                       (Some(#lit), _, _) | (_, _, Some(::fluent_static::intl_pluralrules::PluralCategory::#category_ident))
                    }
                } else {
                    quote! {
                        (Some(#lit), None, None)
                    }
                }
            }
            ast::VariantKey::NumberLiteral { value } => {
                let f = f64::from_str(&value.to_string())
                    .map_err(|_| Error::InvalidLiteral(value.to_string()))?;
                let lit = Literal::f64_suffixed(f);
                quote! {
                    (None, Some(v), _) if f64::abs(#lit - v) < f64::EPSILON
                }
            }
        })
    }

    fn visit_comment(&mut self, _comment: &ast::Comment<S>) -> Self::Output {
        todo!()
    }

    fn visit_call_arguments(&mut self, _arguments: &ast::CallArguments<S>) -> Self::Output {
        todo!()
    }

    fn visit_named_argument(&mut self, _argument: &ast::NamedArgument<S>) -> Self::Output {
        todo!()
    }

    fn visit_inline_expression(&mut self, expression: &ast::InlineExpression<S>) -> Self::Output {
        match expression {
            ast::InlineExpression::StringLiteral { value } => {
                let literal = Literal::string(&value.to_string());
                Ok(quote! {
                    out.write_str(#literal)?;
                })
            }
            ast::InlineExpression::NumberLiteral { value } => {
                let s = value.to_string();
                if let FluentValue::Number(n) = FluentValue::try_number(&s) {
                    let literal = Literal::string(&n.as_string());
                    Ok(quote! {
                        out.write_str(#literal)?;
                    })
                } else {
                    Err(Error::InvalidLiteral(s))
                }
            }
            ast::InlineExpression::FunctionReference {
                id: _,
                arguments: _,
            } => todo!("Add support for function refs"),
            ast::InlineExpression::MessageReference {
                id: _,
                attribute: _,
            } => todo!("Add support for message refs"),
            ast::InlineExpression::TermReference {
                id: _,
                attribute: _,
                arguments: _,
            } => todo!("Add support for term refs"),
            ast::InlineExpression::VariableReference { id } => {
                let var_ident = self.append_var(id)?;
                // TODO add formatter support
                // TODO add unicode isolating marks support
                // TODO add unicode escaping
                Ok(quote! {
                    match &#var_ident {
                        ::fluent_static::fluent_bundle::FluentValue::String(s) => out.write_str(&s)?,
                        ::fluent_static::fluent_bundle::FluentValue::Number(n) => out.write_str(&n.as_string())?,
                        ::fluent_static::fluent_bundle::FluentValue::Custom(_) => unimplemented!("Custom types are not supported"),
                        ::fluent_static::fluent_bundle::FluentValue::None => (),
                        ::fluent_static::fluent_bundle::FluentValue::Error => (),
                    };
                })
            }
            ast::InlineExpression::Placeable { expression } => expression.accept(self),
        }
    }

    fn visit_expression(&mut self, expression: &ast::Expression<S>) -> Self::Output {
        match expression {
            ast::Expression::Select { selector, variants } => {
                let mut variants: Vec<&ast::Variant<S>> = variants.iter().collect();
                // put default variant to the last position
                variants.sort_by_key(|variant| variant.default);

                let mut default_variant: Vec<&&ast::Variant<S>> =
                    variants.iter().filter(|variant| variant.default).collect();

                let default_variant = if default_variant.len() != 1 {
                    let msg_id = self
                        .current_context()
                        .map(|f| f.id.clone())
                        .ok_or(Error::UnexpectedContextState)?;
                    Err(Error::InvalidSelectorDefaultVariant { message_id: msg_id })
                } else {
                    Ok(default_variant
                        .pop()
                        .expect("Vec::pop returned None for non-empty Vec"))
                }?;

                let has_plural_rules = variants
                    .iter()
                    .find(|variant| get_plural_category(&variant.key).is_some())
                    .is_some();

                let f = 0.1;
                match (Some("a"), Some(f as f64), None as Option<&str>) {
                    (None, Some(v), None) if f64::abs(1.0 - v) < f64::EPSILON => "",
                    _ => "",
                };

                match selector {
                    ast::InlineExpression::StringLiteral { value } => variants
                        .iter()
                        .find(|variant| {
                            if let ast::VariantKey::Identifier { name } = &variant.key {
                                value.to_string() == name.to_string()
                            } else {
                                false
                            }
                        })
                        .unwrap_or(default_variant)
                        .accept(self),
                    ast::InlineExpression::NumberLiteral { value } => {
                        variants
                            .iter()
                            .find(|variant| {
                                if let ast::VariantKey::NumberLiteral {
                                    value: variant_value,
                                } = &variant.key
                                {
                                    // FIXME not sure if it is correct to compare number literals like that
                                    value.to_string() == variant_value.to_string()
                                } else {
                                    false
                                }
                            })
                            .unwrap_or(default_variant)
                            .accept(self)
                    }
                    ast::InlineExpression::VariableReference { id } => {
                        let var_ident = self.append_var(id)?;
                        let selector_expr = if has_plural_rules {
                            quote! {
                                {
                                    match &#var_ident {
                                        ::fluent_static::fluent_bundle::FluentValue::String(s) => (Some(s.as_ref()), None, None),
                                        ::fluent_static::fluent_bundle::FluentValue::Number(n) => {
                                            let plural_category = self.language.plural_rules_cardinal().select(n.value).ok();
                                            (None, Some(n.value), plural_category)
                                        },
                                        ::fluent_static::fluent_bundle::FluentValue::Custom(_) => unimplemented!("Custom types are not supported"),
                                        _ => (None, None, None)
                                    }
                                }
                            }
                        } else {
                            quote! {
                                {
                                    match &#var_ident {
                                        ::fluent_static::fluent_bundle::FluentValue::String(s) => (Some(s.as_ref()), None, None),
                                        ::fluent_static::fluent_bundle::FluentValue::Number(n) => (None, Some(n.value), None),
                                        ::fluent_static::fluent_bundle::FluentValue::Custom(_) => unimplemented!("Custom types are not supported"),
                                        _ => (None, None, None)
                                    }
                                }
                            }
                        };
                        let selector_variants = variants
                            .iter()
                            .map(|variant| variant.accept(self))
                            .collect::<Result<Vec<TokenStream2>, Error>>()?;

                        Ok(quote! {
                            match #selector_expr as (Option<&str>, Option<f64>, Option<::fluent_static::intl_pluralrules::PluralCategory>) {
                                #(#selector_variants),*
                            }
                        })
                    }
                    _ => todo!("Unsupported selector type"),
                }
            }
            ast::Expression::Inline(inline_expression) => inline_expression.accept(self),
        }
    }
}

fn get_plural_category<S: ToString>(key: &ast::VariantKey<S>) -> Option<PluralCategory> {
    if let ast::VariantKey::Identifier { name } = key {
        match name.to_string().as_str() {
            "zero" => Some(PluralCategory::ZERO),
            "one" => Some(PluralCategory::ONE),
            "two" => Some(PluralCategory::TWO),
            "few" => Some(PluralCategory::FEW),
            "many" => Some(PluralCategory::MANY),
            "other" => Some(PluralCategory::OTHER),
            _ => None,
        }
    } else {
        None
    }
}
