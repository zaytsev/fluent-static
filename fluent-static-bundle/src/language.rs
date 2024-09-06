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
    types::{FluentId, FluentMessage, FluentVariable, PublicFluentId},
    Error,
};

#[derive(Debug, Clone)]
enum ExpressionContext {
    Inline,
    Selector { plural_rules: bool },
    Argument,
}

pub struct LanguageBuilder {
    pending_fns: Vec<FluentMessage>,
    expression_contexts: Vec<ExpressionContext>,

    pub language_id: LanguageIdentifier,
    pub prefix: String,
    pub registered_fns: BTreeMap<Ident, FluentMessage>,
    pub registered_message_fns: BTreeMap<PublicFluentId, FluentMessage>,
    pub pending_message_refs: BTreeMap<FluentId, BTreeSet<PublicFluentId>>,
    pub pending_term_refs: BTreeMap<FluentId, String>,
}

impl LanguageBuilder {
    pub fn new(language_id: &LanguageIdentifier) -> Self {
        Self {
            language_id: language_id.clone(),
            prefix: language_id.to_string().to_case(Case::Snake),
            pending_fns: Vec::new(),
            registered_fns: BTreeMap::new(),
            registered_message_fns: BTreeMap::new(),
            pending_message_refs: BTreeMap::new(),
            pending_term_refs: BTreeMap::new(),
            expression_contexts: Vec::new(),
        }
    }

    fn push_message<S: ToString>(&mut self, message: &ast::Message<S>) {
        let id = &message.id;
        self.pending_fns.push(FluentMessage::new(
            id,
            self.make_fn_ident(&message.id, None),
            false,
        ));
    }

    fn push_term<S: ToString>(&mut self, term: &ast::Term<S>) {
        self.pending_fns.push(FluentMessage::new(
            &term.id,
            self.make_fn_ident(&term.id, None),
            true,
        ));
    }

    fn push_attribute<S: ToString>(&mut self, attribute: &ast::Attribute<S>) -> Result<(), Error> {
        if let Some(parent) = self.pending_fns.last() {
            let id = parent.id().join(&attribute.id);
            let fn_ident = self.make_fn_ident(parent.id().clone(), Some(attribute.into()));
            self.pending_fns
                .push(FluentMessage::new(id, fn_ident, parent.is_private()));
            Ok(())
        } else {
            Err(Error::UnexpectedContextState)
        }
    }

    fn generate_message_code(&self, msg: &FluentMessage, body: TokenStream2) -> TokenStream2 {
        let fn_ident = msg.fn_ident();
        let var_idents: BTreeSet<Ident> = msg.vars().into_iter().map(|v| v.var_ident).collect();

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

    fn register_pending_fn(&mut self, body: TokenStream2) -> Result<TokenStream2, Error> {
        let f = self
            .pending_fns
            .pop()
            .ok_or(Error::UnexpectedContextState)?;

        let result = self.generate_message_code(&f, body);

        if self
            .registered_fns
            .insert(f.fn_ident(), f.clone())
            .is_some()
        {
            Err(Error::DuplicateEntryId(f.fn_ident().to_string()))
        } else {
            self.registered_message_fns.insert(f.public_id(), f.clone());
            Ok(result)
        }
    }

    fn append_var<S: ToString>(&mut self, id: &ast::Identifier<S>) -> Result<Ident, Error> {
        if let Some(item) = self.pending_fns.last_mut() {
            let var_name = id.name.to_string();
            let var_ident = format_ident!("{}", var_name.to_case(Case::Snake));
            let var = FluentVariable::new(var_name, var_ident.clone());
            item.add_var(var);
            Ok(var_ident)
        } else {
            Err(Error::UnexpectedContextState)
        }
    }

    fn current_context(&self) -> Result<&FluentMessage, Error> {
        self.pending_fns.last().ok_or(Error::UnexpectedContextState)
    }

    fn make_fn_ident<I: Into<FluentId>>(&self, id: I, attribute: Option<I>) -> Ident {
        let id = id.into().as_ref().to_case(Case::Snake);
        if let Some(attribute) = attribute {
            let attr_id = attribute.into().as_ref().to_case(Case::Snake);
            format_ident!("{}_{}_{}", self.prefix, id, attr_id,)
        } else {
            format_ident!("{}_{}", self.prefix, id)
        }
    }

    fn enter_expr_context(&mut self, ctx: ExpressionContext) {
        self.expression_contexts.push(ctx);
    }

    fn leave_expr_context(&mut self) -> Result<ExpressionContext, Error> {
        self.expression_contexts
            .pop()
            .ok_or(Error::UnexpectedContextState)
    }

    fn current_expr_context(&self) -> &ExpressionContext {
        self.expression_contexts
            .last()
            .unwrap_or(&ExpressionContext::Inline)
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
        self.enter_expr_context(ExpressionContext::Argument);
        todo!()
    }

    fn visit_named_argument(&mut self, _argument: &ast::NamedArgument<S>) -> Self::Output {
        todo!()
    }

    fn visit_inline_expression(&mut self, expression: &ast::InlineExpression<S>) -> Self::Output {
        // TODO: avoid clone
        let ctx = self.current_expr_context().clone();
        match ctx {
            ExpressionContext::Inline => match expression {
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
                ast::InlineExpression::MessageReference { id, attribute } => {
                    let msg_id = if let Some(attribute_id) = attribute {
                        FluentId::from(id).join(attribute_id)
                    } else {
                        FluentId::from(id)
                    };
                    let fn_ident = if let Some(msg) = self
                        .registered_message_fns
                        .get(&PublicFluentId::from(msg_id.clone()))
                    {
                        msg.fn_ident()
                    } else {
                        let current_msg = self.current_context()?.public_id();
                        self.pending_message_refs
                            .entry(msg_id.clone())
                            .or_insert_with(|| BTreeSet::new())
                            .insert(current_msg);
                        self.make_fn_ident(id, attribute.as_ref())
                    };
                    Ok(quote! {
                       self.#fn_ident(out)?;
                    })
                }
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
            },
            ExpressionContext::Selector { plural_rules } => match expression {
                ast::InlineExpression::VariableReference { id } => {
                    let var_ident = self.append_var(id)?;
                    let number_expr = if plural_rules {
                        quote! {
                            {
                                let plural_category = self.language.plural_rules_cardinal().select(n.value).ok();
                                (None, Some(n.value), plural_category)
                            }
                        }
                    } else {
                        quote! {
                            (None, Some(n.value), None)
                        }
                    };
                    Ok(quote! {
                        {
                            match &#var_ident {
                                ::fluent_static::fluent_bundle::FluentValue::String(s) => (Some(s.as_ref()), None, None),
                                ::fluent_static::fluent_bundle::FluentValue::Number(n) => #number_expr,
                                ::fluent_static::fluent_bundle::FluentValue::Custom(_) => unimplemented!("Custom types are not supported"),
                                _ => (None, None, None)
                            }
                        }
                    })
                }
                ast::InlineExpression::FunctionReference {
                    id: _,
                    arguments: _,
                } => unimplemented!("Function refs are not yet supported in selector"),
                ast::InlineExpression::TermReference {
                    id: _,
                    attribute: _,
                    arguments: _,
                } => unimplemented!("Term refs are not yet supprted in selector"),
                _ => Err(Error::UnsupportedFeature {
                    feature: "Unsupported selector expression".to_string(),
                    id: self.current_context()?.id().to_string(),
                }),
            },
            ExpressionContext::Argument => todo!(),
        }
    }

    fn visit_expression(&mut self, expression: &ast::Expression<S>) -> Self::Output {
        match expression {
            ast::Expression::Select { selector, variants } => {
                let mut variants: Vec<&ast::Variant<S>> = variants.iter().collect();
                // make default to the end of variants list
                variants.sort_by_key(|variant| variant.default);

                let default_variants: Vec<&&ast::Variant<S>> =
                    variants.iter().filter(|variant| variant.default).collect();

                if default_variants.len() != 1 {
                    let msg_id = self.current_context()?.id().to_string();
                    Err(Error::InvalidSelectorDefaultVariant { message_id: msg_id })
                } else {
                    let plural_rules = variants
                        .iter()
                        .find(|variant| get_plural_category(&variant.key).is_some())
                        .is_some();

                    self.enter_expr_context(ExpressionContext::Selector { plural_rules });
                    let selector_expr = selector.accept(self)?;
                    self.leave_expr_context()?;
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
