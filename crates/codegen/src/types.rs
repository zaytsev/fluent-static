use std::{cell::RefCell, collections::BTreeSet, fmt::Display, ops::Deref, rc::Rc};

use fluent_syntax::ast;
use syn::Ident;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FluentId(Rc<str>);

impl FluentId {
    pub fn new<S: ToString>(s: &S) -> Self {
        Self(Rc::from(s.to_string()))
    }

    pub fn join(&self, id: impl Into<FluentId>) -> Self {
        Self(Rc::from(format!("{}.{}", self.0, id.into().0)))
    }
}

impl Deref for FluentId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for FluentId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::borrow::Borrow<str> for FluentId {
    fn borrow(&self) -> &str {
        self.0.borrow()
    }
}

impl Display for FluentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<S: ToString> From<&ast::Identifier<S>> for FluentId {
    fn from(value: &ast::Identifier<S>) -> Self {
        Self::new(&value.name)
    }
}

impl<S: ToString> From<&ast::Attribute<S>> for FluentId {
    fn from(value: &ast::Attribute<S>) -> Self {
        Self::from(&value.id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PublicFluentId(FluentId);

impl PublicFluentId {
    pub fn new(s: String) -> Self {
        Self(FluentId::new(&s))
    }
}

impl From<FluentId> for PublicFluentId {
    fn from(value: FluentId) -> Self {
        Self(value.clone())
    }
}

impl std::borrow::Borrow<str> for PublicFluentId {
    fn borrow(&self) -> &str {
        self.0.borrow()
    }
}

impl Display for PublicFluentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone)]
struct FluentMessageAttrs {
    id: FluentId,
    private: bool,
    fn_ident: Ident,
    vars: Vec<FluentVariable>,
    var_idents: BTreeSet<Ident>,
    unique_vars: BTreeSet<FluentVariable>,
}

#[derive(Debug, Clone)]
pub struct FluentMessage {
    attrs: Rc<RefCell<FluentMessageAttrs>>,
}

impl FluentMessage {
    pub fn new(id: impl Into<FluentId>, fn_ident: Ident, private: bool) -> Self {
        let attrs = Rc::new(RefCell::new(FluentMessageAttrs {
            id: id.into(),
            fn_ident,
            private,
            vars: Vec::new(),
            var_idents: BTreeSet::new(),
            unique_vars: BTreeSet::new(),
        }));
        Self { attrs }
    }

    pub fn id(&self) -> FluentId {
        self.attrs.borrow().id.clone()
    }

    pub fn fn_ident(&self) -> Ident {
        self.attrs.borrow().fn_ident.clone()
    }

    pub fn is_private(&self) -> bool {
        self.attrs.borrow().private
    }

    pub fn add_var(&self, var: FluentVariable) {
        let mut attrs = self.attrs.borrow_mut();
        if !attrs.unique_vars.contains(&var) {
            attrs.unique_vars.insert(var.clone());
            attrs.var_idents.insert(var.var_ident.clone());
            attrs.vars.push(var);
        }
    }

    pub fn has_vars(&self) -> bool {
        let attrs = self.attrs.borrow();
        !attrs.unique_vars.is_empty()
    }

    pub fn declared_vars(&self) -> Vec<FluentVariable> {
        let attrs = self.attrs.borrow();
        attrs.vars.iter().cloned().collect()
    }

    pub fn vars(&self) -> BTreeSet<FluentVariable> {
        let attrs = self.attrs.borrow();
        attrs.vars.iter().cloned().collect()
    }

    pub fn public_id(&self) -> PublicFluentId {
        let attrs = self.attrs.borrow();
        if attrs.unique_vars.is_empty() {
            attrs.id.clone().into()
        } else {
            let vars = attrs
                .unique_vars
                .iter()
                .map(|var| var.var_name.as_str())
                .collect::<Vec<&str>>()
                .join(",");

            PublicFluentId::new(format!("{}={}", attrs.id, vars))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FluentVariable {
    pub var_ident: Ident,
    pub var_name: String,
}

impl FluentVariable {
    pub fn new(var_name: String, var_ident: Ident) -> Self {
        Self {
            var_ident,
            var_name,
        }
    }
}
