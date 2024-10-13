use std::collections::{BTreeMap, BTreeSet};

use convert_case::{Case, Casing};
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
    Selector {
        plural_rules: bool,
    },
    TermArguments {
        term: FluentMessage,
    },
    FunctionCall {
        function_id: String,
        positional_args: Ident,
        named_args: Ident,
    },
}

pub struct LanguageBuilder {
    pending_fns: Vec<FluentMessage>,
    expression_contexts: Vec<ExpressionContext>,

    #[allow(dead_code)]
    pub language_id: LanguageIdentifier,
    pub prefix: String,
    pub registered_fns: BTreeMap<FluentId, FluentMessage>,
    pub registered_message_fns: BTreeMap<PublicFluentId, FluentMessage>,
}

impl LanguageBuilder {
    pub fn new(language_id: &LanguageIdentifier) -> Self {
        Self {
            language_id: language_id.clone(),
            prefix: language_id.to_string().to_case(Case::Snake),
            pending_fns: Vec::new(),
            registered_fns: BTreeMap::new(),
            registered_message_fns: BTreeMap::new(),
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
                #(#var_idents: ::fluent_static::value::Value<'a>),*
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

        if self.registered_fns.insert(f.id(), f.clone()).is_some() {
            Err(Error::DuplicateEntryId(f.id().to_string()))
        } else {
            if !f.is_private() {
                self.registered_message_fns.insert(f.public_id(), f.clone());
            }
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

    fn find_entry<S: ToString>(
        &self,
        id: &ast::Identifier<S>,
        attribute: Option<&ast::Identifier<S>>,
    ) -> (FluentId, Option<FluentMessage>) {
        let msg_id = if let Some(attribute_id) = attribute {
            FluentId::from(id).join(attribute_id)
        } else {
            FluentId::from(id)
        };
        (msg_id.clone(), self.registered_fns.get(&msg_id).cloned())
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

    fn visit_text_element(&mut self, value: &S) -> Self::Output {
        let text = Literal::string(value.to_string().as_str());
        Ok(quote! {
            out.write_str(#text)?;
        })
    }

    fn visit_attribute(&mut self, attribute: &ast::Attribute<S>) -> Self::Output {
        self.push_attribute(attribute)?;
        let body = attribute.value.accept(self)?;
        self.register_pending_fn(body)
    }

    fn visit_variant(
        &mut self,
        variant_key: &ast::VariantKey<S>,
        pattern: &ast::Pattern<S>,
        is_default: bool,
    ) -> Self::Output {
        let match_key = if is_default {
            quote! {
                _
            }
        } else {
            match variant_key {
                ast::VariantKey::Identifier { name } => {
                    let name = name.to_string();
                    let lit = Literal::string(&name);
                    if get_plural_category(variant_key).is_some() {
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
                    let number = mk_number(value)?;
                    quote! {
                        (None, Some(n), _) if n == & #number
                    }
                }
            }
        };

        let body = pattern.accept(self)?;

        Ok(quote! {
            #match_key => {
                #body
            }
        })
    }

    fn visit_comment(&mut self, _comment: &ast::Comment<S>) -> Self::Output {
        todo!()
    }

    fn visit_call_arguments(&mut self, arguments: &ast::CallArguments<S>) -> Self::Output {
        match self.current_expr_context() {
            ExpressionContext::TermArguments { term } => {
                let term = term.to_owned();
                let vars = term.vars();
                let vars_by_name: BTreeMap<&str, &Ident> = vars
                    .iter()
                    .map(|var| (var.var_name.as_str(), &var.var_ident))
                    .collect();
                let mut sorted_args: BTreeMap<&Ident, TokenStream2> = BTreeMap::new();
                for named_arg in arguments.named.iter() {
                    let name = named_arg.name.name.to_string();
                    if let Some(ident) = vars_by_name.get(name.as_str()) {
                        let tokens = named_arg.accept(self)?;
                        sorted_args.insert(ident, tokens);
                    } else {
                        let term_id = term.id().to_string();
                        return Err(Error::UndeclaredTermArgument {
                            term_id,
                            arg_name: name,
                        });
                    };
                }
                let args: Vec<TokenStream2> = sorted_args.into_values().collect();
                Ok(quote! {
                    #(#args),*
                })
            }
            ExpressionContext::Inline => Err(Error::UnexpectedContextState),
            ExpressionContext::Selector { .. } => Err(Error::UnexpectedContextState),
            ExpressionContext::FunctionCall {
                positional_args,
                named_args,
                ..
            } => {
                let positional_args_ident = positional_args.clone();
                let named_args_ident = named_args.clone();

                let positional: Vec<TokenStream2> = arguments
                    .positional
                    .iter()
                    .map(|expr| expr.accept(self))
                    .collect::<Result<Vec<TokenStream2>, Error>>()?;

                let named = arguments
                    .named
                    .iter()
                    .map(|arg| {
                        arg.value.accept(self).map(|val| {
                            let name = Literal::string(&arg.name.name.to_string());
                            quote! {
                                (#name, #val)
                            }
                        })
                    })
                    .collect::<Result<Vec<TokenStream2>, Error>>()?;
                Ok(quote! {
                    {
                        let #positional_args_ident = [ #(#positional),* ];
                        let #named_args_ident = [ #(#named),* ];

                    }
                })
            }
        }
    }

    fn visit_named_argument(&mut self, argument: &ast::NamedArgument<S>) -> Self::Output {
        argument.value.accept(self)
    }

    fn visit_string_literal(&mut self, value: &S) -> Self::Output {
        match self.current_expr_context() {
            ExpressionContext::Inline => {
                let literal = Literal::string(&value.to_string());
                Ok(quote! {
                    out.write_str(#literal)?;
                })
            }
            ExpressionContext::Selector { .. } => Err(Error::UnsupportedFeature {
                feature: "Usage of string literal as a selector".to_string(),
                id: self.current_context()?.id().to_string(),
            }),
            ExpressionContext::TermArguments { .. } | ExpressionContext::FunctionCall { .. } => {
                let lit = Literal::string(&value.to_string());
                Ok(quote! {
                    ::fluent_static::value::Value::from(#lit)
                })
            }
        }
    }

    fn visit_number_literal(&mut self, value: &S) -> Self::Output {
        match self.current_expr_context() {
            ExpressionContext::Inline => {
                let value = value.to_string();
                let literal = Literal::string(&value);
                Ok(quote! {
                    out.write_str(#literal)?;
                })
            }
            ExpressionContext::Selector { .. } => Err(Error::UnsupportedFeature {
                feature: "Usage of number literal as a selector".to_string(),
                id: self.current_context()?.id().to_string(),
            }),
            ExpressionContext::TermArguments { .. } | ExpressionContext::FunctionCall { .. } => {
                let number = mk_number(value)?;
                Ok(quote! {
                    ::fluent_static::value::Value::Number { value: #number, format: None }
                })
            }
        }
    }

    fn visit_function_reference(
        &mut self,
        id: &ast::Identifier<S>,
        arguments: &ast::CallArguments<S>,
    ) -> Self::Output {
        let function_id = id.name.to_string();
        let positional_args = format_ident!("positional_args");
        let named_args = format_ident!("named_args");

        self.enter_expr_context(ExpressionContext::FunctionCall {
            function_id: function_id.clone(),
            positional_args,
            named_args,
        });

        let args = arguments.accept(self)?;

        self.leave_expr_context()?;

        Ok(quote! {
            {
                #args
            }
        })
    }

    fn visit_message_reference(
        &mut self,
        id: &ast::Identifier<S>,
        attribute: Option<&ast::Identifier<S>>,
    ) -> Self::Output {
        match self.current_expr_context() {
            ExpressionContext::Inline => {
                let (msg_id, msg) = self.find_entry(id, attribute);
                if let Some(msg) = msg {
                    let fn_ident = msg.fn_ident();
                    Ok(quote! {
                       self.#fn_ident(out)?;
                    })
                } else {
                    let entry_id = self.current_context()?.id().to_string();
                    let reference_id = msg_id.to_string();
                    Err(Error::UndeclaredMessageReference {
                        entry_id,
                        reference_id,
                    })
                }
            }
            ExpressionContext::Selector { .. } => Err(Error::UnsupportedFeature {
                feature: "Usage of message reference as selector".to_string(),
                id: self.current_context()?.id().to_string(),
            }),
            ExpressionContext::TermArguments { .. } | ExpressionContext::FunctionCall { .. } => {
                let (msg_id, msg) = self.find_entry(id, attribute);
                if let Some(msg) = msg {
                    let fn_ident = msg.fn_ident();
                    Ok(quote! {
                        {
                            let mut out = String::new();
                            self.#fn_ident(&mut out)?;
                            ::fluent_static::value::Value::from(out)
                        }
                    })
                } else {
                    let entry_id = self.current_context()?.id().to_string();
                    let reference_id = msg_id.to_string();
                    Err(Error::UndeclaredMessageReference {
                        entry_id,
                        reference_id,
                    })
                }
            }
        }
    }

    fn visit_term_reference(
        &mut self,
        id: &ast::Identifier<S>,
        attribute: Option<&ast::Identifier<S>>,
        arguments: Option<&ast::CallArguments<S>>,
    ) -> Self::Output {
        match self.current_expr_context() {
            ExpressionContext::Inline => {
                let (term_id, term) = self.find_entry(id, attribute);
                if let Some(term) = term.as_ref() {
                    let fn_ident = term.fn_ident();
                    let args = if let Some(args) = arguments.as_ref() {
                        self.enter_expr_context(ExpressionContext::TermArguments {
                            term: term.clone(),
                        });
                        let result = args.accept(self);
                        self.leave_expr_context()?;
                        result?
                    } else {
                        quote! {}
                    };
                    Ok(quote! {
                       self.#fn_ident(out, #args)?;
                    })
                } else {
                    let entry_id = self.current_context()?.id().to_string();
                    let reference_id = term_id.to_string();
                    Err(Error::UndeclaredTermReference {
                        entry_id,
                        reference_id,
                    })
                }
            }
            ExpressionContext::Selector { .. } => Err(Error::UnsupportedFeature {
                feature: "Usage of term reference as selector".to_string(),
                id: self.current_context()?.id().to_string(),
            }),
            ExpressionContext::TermArguments { .. } | ExpressionContext::FunctionCall { .. } => {
                let (term_id, term) = self.find_entry(id, attribute);
                if let Some(term) = term.as_ref() {
                    let fn_ident = term.fn_ident();
                    let args = if let Some(args) = arguments.as_ref() {
                        self.enter_expr_context(ExpressionContext::TermArguments {
                            term: term.clone(),
                        });
                        let result = args.accept(self);
                        self.leave_expr_context()?;
                        result?
                    } else {
                        quote! {}
                    };
                    Ok(quote! {
                        {
                            let mut out = String::new();
                            self.#fn_ident(&mut out, #args)?;
                            ::fluent_static::value::Value::from(out)
                        }
                    })
                } else {
                    let entry_id = self.current_context()?.id().to_string();
                    let reference_id = term_id.to_string();
                    Err(Error::UndeclaredTermReference {
                        entry_id,
                        reference_id,
                    })
                }
            }
        }
    }

    fn visit_variable_reference(&mut self, id: &ast::Identifier<S>) -> Self::Output {
        match self.current_expr_context() {
            ExpressionContext::Inline => {
                let var_ident = self.append_var(id)?;
                // TODO add formatter support
                // TODO add unicode isolating marks support
                // TODO add unicode escaping
                // TODO apply NUMBER format
                Ok(quote! {
                    match &#var_ident {
                        ::fluent_static::value::Value::String(s) => out.write_str(&s)?,
                        ::fluent_static::value::Value::Number{ value, .. } => out.write_str(&value.as_string())?,
                        ::fluent_static::value::Value::Empty => (),
                        ::fluent_static::value::Value::Error => (),
                    };
                })
            }
            ExpressionContext::Selector { plural_rules } => {
                let has_plural_rules = *plural_rules;
                let var_ident = self.append_var(id)?;
                let number_expr = if has_plural_rules {
                    quote! {
                        {
                            let plural_category = self.language.plural_rules_cardinal().select(n.as_f64()).ok();
                            (None, Some(n), plural_category)
                        }
                    }
                } else {
                    quote! {
                        (None, Some(n), None)
                    }
                };
                Ok(quote! {
                    {
                        match &#var_ident {
                            ::fluent_static::value::Value::String(s) => (Some(s.as_ref()), None, None),
                            ::fluent_static::value::Value::Number { value: n, .. } => #number_expr,
                            _ => (None, None, None)
                        }
                    }
                })
            }
            ExpressionContext::TermArguments { .. } => Err(Error::UnsupportedFeature {
                feature: "Usage of variable reference as a term argument".to_string(),
                id: self.current_context()?.id().to_string(),
            }),
            ExpressionContext::FunctionCall { .. } => {
                let var_ident = self.append_var(id)?;
                // TODO should value be cloned? just in case if it already used
                Ok(quote! {
                    #var_ident
                })
            }
        }
    }

    fn visit_select_expression<'a, I>(
        &mut self,
        selector: &'a ast::InlineExpression<S>,
        variants: I,
    ) -> Self::Output
    where
        I: Iterator<Item = &'a ast::Variant<S>>,
    {
        let mut variants: Vec<&ast::Variant<S>> = variants.collect();
        // make default variant(s) to be at the end of the list
        variants.sort_by_key(|variant| variant.default);

        let default_variants: Vec<&&ast::Variant<S>> =
            variants.iter().filter(|variant| variant.default).collect();

        // TODO this is probably redundant because parser should catch it
        if default_variants.len() != 1 {
            let msg_id = self.current_context()?.id().to_string();
            Err(Error::InvalidSelectorDefaultVariant { message_id: msg_id })
        } else {
            // TODO ignore plural rules if there is only one category and it is default variant
            // e.g. *[other] =
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
                match #selector_expr as (Option<&str>, Option<&::fluent_static::value::Number>, Option<::fluent_static::intl_pluralrules::PluralCategory>) {
                    #(#selector_variants),*
                }
            })
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

fn mk_number<S: ToString>(value: &S) -> Result<TokenStream2, Error> {
    let value = value.to_string();
    if let Ok(n) = value.parse::<i64>() {
        Ok(quote! {
            ::fluent_static::value::Number::I64(#n)
        })
    } else if let Ok(n) = value.parse::<u64>() {
        Ok(quote! {
            ::fluent_static::value::Number::U64(#n)
        })
    } else if let Ok(n) = value.parse::<i128>() {
        Ok(quote! {
            ::fluent_static::value::Number::I128(#n)
        })
    } else if let Ok(n) = value.parse::<u128>() {
        Ok(quote! {
            ::fluent_static::value::Number::U128(#n)
        })
    } else if let Ok(n) = value.parse::<f64>() {
        Ok(quote! {
            ::fluent_static::value::Number::F64(#n)
        })
    } else {
        Err(Error::InvalidLiteral(value))
    }
}
