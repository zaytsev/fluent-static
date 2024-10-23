use std::any::type_name;

use fluent_static_value::Value;

pub mod builtins;

pub type FluentFunction<'a, 'b> = fn(&'a [Value<'a>], &'a [(&'a str, Value<'a>)]) -> Value<'b>;

pub trait FluentFunctionDescriptor {
    fn type_name(&self) -> &'static str;
}

impl<'b, F> FluentFunctionDescriptor for F
where
    F: for<'a> Fn(&'a [Value<'a>], &'a [(&'a str, Value<'a>)]) -> Value<'b> + 'static,
{
    fn type_name(&self) -> &'static str {
        type_name::<Self>()
    }
}
