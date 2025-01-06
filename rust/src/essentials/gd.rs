use godot::builtin::{Callable, StringName};
use godot::meta::AsArg;
use godot::obj::WithBaseField;

pub trait BindableCallable {
    /// Bind a method of the object to a callable.
    fn bind(&self, method_name: impl AsArg<StringName>) -> Callable;
}

impl<T: WithBaseField> BindableCallable for T {
    fn bind(&self, method_name: impl AsArg<StringName>) -> Callable {
        Callable::from_object_method(&self.to_gd(), method_name)
    }
}
