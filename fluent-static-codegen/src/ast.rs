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
        visitor.visit_pattern_element(self)
    }
}

impl<S> Node<S> for ast::Attribute<S> {
    fn accept<V: Visitor<S>>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_attribute(self)
    }
}

impl<S> Node<S> for ast::Identifier<S> {
    fn accept<V: Visitor<S>>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_identifier(self)
    }
}

impl<S> Node<S> for ast::Variant<S> {
    fn accept<V: Visitor<S>>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_variant(self)
    }
}

impl<S> Node<S> for ast::VariantKey<S> {
    fn accept<V: Visitor<S>>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_variant_key(self)
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
        visitor.visit_inline_expression(self)
    }
}

impl<S> Node<S> for ast::Expression<S> {
    fn accept<V: Visitor<S>>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_expression(self)
    }
}

pub trait Visitor<S> {
    type Output;

    fn visit_resource(&mut self, resource: &ast::Resource<S>) -> Self::Output;
    fn visit_entry(&mut self, entry: &ast::Entry<S>) -> Self::Output;
    fn visit_message(&mut self, message: &ast::Message<S>) -> Self::Output;
    fn visit_term(&mut self, term: &ast::Term<S>) -> Self::Output;
    fn visit_pattern(&mut self, pattern: &ast::Pattern<S>) -> Self::Output;
    fn visit_pattern_element(&mut self, element: &ast::PatternElement<S>) -> Self::Output;
    fn visit_attribute(&mut self, attribute: &ast::Attribute<S>) -> Self::Output;
    fn visit_identifier(&mut self, identifier: &ast::Identifier<S>) -> Self::Output;
    fn visit_variant(&mut self, variant: &ast::Variant<S>) -> Self::Output;
    fn visit_variant_key(&mut self, key: &ast::VariantKey<S>) -> Self::Output;
    fn visit_comment(&mut self, comment: &ast::Comment<S>) -> Self::Output;
    fn visit_call_arguments(&mut self, arguments: &ast::CallArguments<S>) -> Self::Output;
    fn visit_named_argument(&mut self, argument: &ast::NamedArgument<S>) -> Self::Output;
    fn visit_inline_expression(&mut self, expression: &ast::InlineExpression<S>) -> Self::Output;
    fn visit_expression(&mut self, expression: &ast::Expression<S>) -> Self::Output;
}
