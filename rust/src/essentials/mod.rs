use godot::{
    builtin::{Dictionary, Variant, dict},
    meta::ToGodot,
};

pub struct GdResult;

impl GdResult {
    pub fn ok<T: ToGodot>(value: T) -> Dictionary {
        dict! {
            "error": Variant::nil(),
            "data": value.to_variant(),
        }
    }

    pub fn err<T: ToGodot>(error: T) -> Dictionary {
        dict! {
            "error": error.to_variant(),
            "data": Variant::nil(),
        }
    }
}
