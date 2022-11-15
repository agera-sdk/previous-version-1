use crate::prelude::*;
use rialight::localization::{
    LocaleBundle,
    LocaleBundleFormatArgument,
    LocaleBundleOptions,
    LocaleBundleOptionsForAssets,
    LocaleBundleLoadMethod,
    Language,
    Region,
    Direction,
};
use rialight::util::{AnyStringType};
use maplit::hashmap;

lazy_static! {
    pub static ref B: LocaleBundle = LocaleBundle::new(
        LocaleBundleOptions::new()
            .supported_locales(vec!["en-US"])
            .default_locale("en-US")
            .fallbacks(hashmap! {})
            .assets(LocaleBundleOptionsForAssets::new()
                .src("app://res/lang")
                .base_file_names(
                    vec!["_"] // _.json
                )
                .clean_unused(true)
                .load_method(LocaleBundleLoadMethod::FileSystem))
    ); // msg_locator
}

pub fn t(id: impl AnyStringType) -> String {
    tf(id, vec![])
}

pub fn tf(id: impl AnyStringType, options: Vec<&dyn LocaleBundleFormatArgument>) -> String {
    B.get_formatted(id, options)
}