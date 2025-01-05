pub mod logger;
use godot::{
    builtin::{Dictionary, Variant, dict},
    meta::{FromGodot, GodotConvert, ToGodot},
    obj::Gd,
    prelude::{ConvertError, GodotClass},
};
use std::fmt::Debug;

#[derive(Debug, GodotClass)]
#[class(no_init)]
pub struct GdResult {
    #[export]
    pub error: Variant,
    #[export]
    pub data: Variant,
}

impl GdResult {
    // pub fn ok<T: ToGodot>(value: T) -> Dictionary {
    //     dict! {
    //         "error": Variant::nil(),
    //         "data": value.to_variant(),
    //     }
    // }

    pub fn ok<T: ToGodot>(value: T) -> GdResult {
        GdResult {
            error: Variant::nil(),
            data: value.to_variant(),
        }
    }

    pub fn err<T: ToGodot>(error: T) -> GdResult {
        GdResult {
            error: error.to_variant(),
            data: Variant::nil(),
        }
    }
}

impl From<GdResult> for Dictionary {
    fn from(result: GdResult) -> Self {
        dict! {
            "error": result.error,
            "data": result.data,
        }
    }
}

impl<T: ToGodot, E: Debug> From<Result<T, E>> for GdResult {
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(data) => GdResult::ok(data),
            Err(error) => GdResult::err(format!("{:?}", error)),
        }
    }
}

impl GodotConvert for GdResult {
    type Via = Dictionary;
}

impl FromGodot for GdResult {
    fn try_from_variant(variant: &Variant) -> Result<Self, ConvertError> {
        let dict = variant.try_to::<Dictionary>()?;
        Ok(GdResult {
            error: dict.get("error").unwrap_or(Variant::nil()).clone(),
            data: dict.get("data").unwrap_or(Variant::nil()).clone(),
        })
    }

    fn try_from_godot(via: Dictionary) -> Result<Self, ConvertError> {
        Ok(GdResult {
            error: via.get("error").unwrap_or(Variant::nil()).clone(),
            data: via.get("data").unwrap_or(Variant::nil()).clone(),
        })
    }
}

impl ToGodot for GdResult {
    type ToVia<'v> = Dictionary;

    fn to_variant(&self) -> Variant {
        let dict = dict! {
            "error": self.error.clone(),
            "data": self.data.clone(),
        };

        dict.to_variant()
    }

    fn to_godot(&self) -> Self::ToVia<'_> {
        dict! {
            "error": self.error.clone(),
            "data": self.data.clone(),
        }
    }
}
