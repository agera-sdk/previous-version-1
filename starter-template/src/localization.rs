use crate::prelude::*;
use rialight::localization::{
    MessageLocator,
    MessageLocatorFormatArgument,
    MessageLocatorOptions,
    MessageLocatorAssetOptions,
    MessageLocatorLoadMethod,
    Language,
    Region,
    Direction,
};
use maplit::hashmap;

lazy_static! {
    pub static ref L: MessageLocator = MessageLocator::new(
        MessageLocatorOptions::new()
            .supported_locales(vec!["en-US"])
            .default_locale("en-US")
            .fallbacks(hashmap! {})
            .assets(MessageLocatorAssetOptions::new()
                .src("app://res/lang")
                .base_file_names(
                    vec!["_"] // _.json
                )
                .clean_unused(true)
                .load_method(MessageLocatorLoadMethod::FileSystem))
    ); // msg_locator
}

pub fn t<S: ToString>(id: S) -> String {
    tf(id, vec![])
}

pub fn tf<S: ToString>(id: S, options: Vec<&dyn MessageLocatorFormatArgument>) -> String {
    L.get_formatted(id, options)
}