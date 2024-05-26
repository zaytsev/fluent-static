use proc_macro2::TokenStream;

use crate::{bundle::MessageBundle, Error};

mod msgfn;

pub use msgfn::FunctionPerMessageCodeGenerator;

pub trait CodeGenerator {
    fn generate(&self, bundle: &MessageBundle) -> Result<TokenStream, Error>;
}
