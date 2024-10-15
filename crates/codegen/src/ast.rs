use fluent_syntax::ast;

pub trait Node<S> {
    fn accept<V: Visitor<S>>(&self, visitor: &mut V) -> V::Output;
}

impl<S> Node<S> for ast::Resource<S> {
    fn accept<V: Visitor<S>>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_resource(self)
    }
}

impl<S> Node<S> for ast::Entry<S> {
    fn accept<V: Visitor<S>>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_entry(self)
    }
}

impl<S> Node<S> for ast::Message<S> {
    fn accept<V: Visitor<S>>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_message(self)
    }
}

impl<S> Node<S> for ast::Term<S> {
    fn accept<V: Visitor<S>>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_term(self)
    }
}

impl<S> Node<S> for ast::Pattern<S> {
    fn accept<V: Visitor<S>>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_pattern(self)
    }
}

impl<S> Node<S> for ast::PatternElement<S> {
    fn accept<V: Visitor<S>>(&self, visitor: &mut V) -> V::Output {
        match self {
            ast::PatternElement::TextElement { value } => visitor.visit_text_element(value),
            ast::PatternElement::Placeable { expression } => expression.accept(visitor),
        }
    }
}

impl<S> Node<S> for ast::Attribute<S> {
    fn accept<V: Visitor<S>>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_attribute(self)
    }
}

impl<S> Node<S> for ast::Variant<S> {
    fn accept<V: Visitor<S>>(&self, visitor: &mut V) -> V::Output {
        visitor.begin_variant(&self);
        let result = visitor.visit_variant(&self.key, &self.value, self.default);
        visitor.end_variant(&self);
        result
    }
}

impl<S> Node<S> for ast::Comment<S> {
    fn accept<V: Visitor<S>>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_comment(self)
    }
}

impl<S> Node<S> for ast::CallArguments<S> {
    fn accept<V: Visitor<S>>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_call_arguments(self)
    }
}

impl<S> Node<S> for ast::NamedArgument<S> {
    fn accept<V: Visitor<S>>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_named_argument(self)
    }
}

impl<S> Node<S> for ast::InlineExpression<S> {
    fn accept<V: Visitor<S>>(&self, visitor: &mut V) -> V::Output {
        visitor.begin_inline_expression(&self);
        let result = match self {
            ast::InlineExpression::StringLiteral { value } => visitor.visit_string_literal(value),
            ast::InlineExpression::NumberLiteral { value } => visitor.visit_number_literal(value),
            ast::InlineExpression::FunctionReference { id, arguments } => {
                visitor.visit_function_reference(id, arguments)
            }
            ast::InlineExpression::MessageReference { id, attribute } => {
                visitor.visit_message_reference(id, attribute.as_ref())
            }
            ast::InlineExpression::TermReference {
                id,
                attribute,
                arguments,
            } => visitor.visit_term_reference(id, attribute.as_ref(), arguments.as_ref()),
            ast::InlineExpression::VariableReference { id } => visitor.visit_variable_reference(id),
            ast::InlineExpression::Placeable { expression } => expression.accept(visitor),
        };
        visitor.end_inline_expression(&self);
        result
    }
}

impl<S> Node<S> for ast::Expression<S> {
    fn accept<V: Visitor<S>>(&self, visitor: &mut V) -> V::Output {
        match self {
            ast::Expression::Select { selector, variants } => {
                visitor.visit_select_expression(selector, variants.iter())
            }
            ast::Expression::Inline(expr) => expr.accept(visitor),
        }
    }
}

pub trait Visitor<S> {
    type Output;

    fn visit_resource(&mut self, resource: &ast::Resource<S>) -> Self::Output;
    fn visit_entry(&mut self, entry: &ast::Entry<S>) -> Self::Output;
    fn visit_message(&mut self, message: &ast::Message<S>) -> Self::Output;
    fn visit_term(&mut self, term: &ast::Term<S>) -> Self::Output;
    fn visit_pattern(&mut self, pattern: &ast::Pattern<S>) -> Self::Output;
    fn visit_text_element(&mut self, value: &S) -> Self::Output;
    fn visit_attribute(&mut self, attribute: &ast::Attribute<S>) -> Self::Output;

    #[allow(unused_variables)]
    fn begin_variant(&mut self, variant: &ast::Variant<S>) {}

    fn visit_variant(
        &mut self,
        variant_key: &ast::VariantKey<S>,
        pattern: &ast::Pattern<S>,
        is_default: bool,
    ) -> Self::Output;

    #[allow(unused_variables)]
    fn end_variant(&mut self, variant: &ast::Variant<S>) {}

    fn visit_comment(&mut self, comment: &ast::Comment<S>) -> Self::Output;
    fn visit_call_arguments(&mut self, arguments: &ast::CallArguments<S>) -> Self::Output;
    fn visit_named_argument(&mut self, argument: &ast::NamedArgument<S>) -> Self::Output;

    #[allow(unused_variables)]
    fn begin_inline_expression(&mut self, inline_expression: &ast::InlineExpression<S>) {}

    fn visit_string_literal(&mut self, value: &S) -> Self::Output;
    fn visit_number_literal(&mut self, value: &S) -> Self::Output;
    fn visit_function_reference(
        &mut self,
        id: &ast::Identifier<S>,
        arguments: &ast::CallArguments<S>,
    ) -> Self::Output;
    fn visit_message_reference(
        &mut self,
        id: &ast::Identifier<S>,
        attribute: Option<&ast::Identifier<S>>,
    ) -> Self::Output;
    fn visit_term_reference(
        &mut self,
        id: &ast::Identifier<S>,
        attribute: Option<&ast::Identifier<S>>,
        arguments: Option<&ast::CallArguments<S>>,
    ) -> Self::Output;
    fn visit_variable_reference(&mut self, id: &ast::Identifier<S>) -> Self::Output;

    #[allow(unused_variables)]
    fn end_inline_expression(&mut self, inline_expression: &ast::InlineExpression<S>) {}

    #[allow(unused_variables)]
    fn begin_expression(&mut self, expression: &ast::Expression<S>) {}
    fn visit_select_expression<'a, I>(
        &mut self,
        selector: &'a ast::InlineExpression<S>,
        variants: I,
    ) -> Self::Output
    where
        I: Iterator<Item = &'a ast::Variant<S>>;

    #[allow(unused_variables)]
    fn end_expression(&mut self, expression: &ast::Expression<S>) {}
}
