use std::{borrow::Cow, fmt::Display, ops::Deref};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Message(pub(crate) Cow<'static, str>);

impl Message {
    pub const fn new(value: Cow<'static, str>) -> Self {
        Self(value)
    }
}

impl From<String> for Message {
    fn from(value: String) -> Self {
        Self::new(Cow::Owned(value))
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Deref for Message {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl PartialEq<str> for Message {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<Message> for str {
    fn eq(&self, other: &Message) -> bool {
        self == other.0
    }
}

impl PartialEq<&str> for Message {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<Message> for &str {
    fn eq(&self, other: &Message) -> bool {
        *self == other.0
    }
}
