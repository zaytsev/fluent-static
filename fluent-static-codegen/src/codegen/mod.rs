use proc_macro2::TokenStream;

use crate::{bundle::MessageBundle, Error};

mod common;
mod msgfn;
mod msgstruct;

pub use msgfn::FunctionPerMessageCodeGenerator;
pub use msgstruct::MessageBundleCodeGenerator;

pub struct FluentBundleOptions {
    pub use_isolating: bool,
    pub transform_fn: Option<String>,
    pub format_fn: Option<String>,
}

impl Default for FluentBundleOptions {
    fn default() -> Self {
        Self {
            use_isolating: true,
            transform_fn: None,
            format_fn: None,
        }
    }
}

pub trait CodeGenerator {
    fn generate(&self, bundle: &MessageBundle) -> Result<TokenStream, Error>;
}
