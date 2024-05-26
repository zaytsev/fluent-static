use proc_macro2::TokenStream;

use crate::{bundle::MessageBundle, Error};

mod common;
mod msgfn;
mod msgstruct;

pub use msgfn::FunctionPerMessageCodeGenerator;
pub use msgstruct::MessageBundleCodeGenerator;

pub trait CodeGenerator {
    fn generate(&self, bundle: &MessageBundle) -> Result<TokenStream, Error>;
}
