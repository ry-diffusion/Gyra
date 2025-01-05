use godot::{
    builtin::{Dictionary, dict},
    prelude::{GodotClass, godot_api},
};
use gyra_net::query::fetch_status_of;

use crate::essentials::GdResult;

#[derive(GodotClass)]
#[class(no_init)]
struct NetUtils;

#[godot_api]
impl NetUtils {
    #[func]
    fn query_server_info(address: String) -> Dictionary {
        let result = fetch_status_of(address);
        match result {
            Ok(server_info) => GdResult::ok(dict! {
                "latency": server_info.latency,
                "information": server_info.server_info
            }),
            Err(error) => GdResult::err(error.to_string()),
        }
    }
}
