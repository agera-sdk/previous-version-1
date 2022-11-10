pub mod localization;
pub mod prelude {
    pub use rialight::prelude::*;
}

#[tokio::main]
pub fn main() {
    initialize_rialight();
}

/// **DO NOT MODIFY THIS FUNCTION.**
fn initialize_rialight() {
    include!(concat!(env!("OUT_DIR"), "/rialight_entry.rs"));
}