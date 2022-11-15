/*
pub use rialight_display as display;
pub use rialight_filesystem as filesystem;
pub use rialight_localization as localization;
pub use rialight_net as net;
pub use rialight_sound as sound;
pub use rialight_util as util;
*/

pub mod prelude {
    pub use lazy_regex::{
        lazy_regex, regex, regex_captures,
        regex_find, regex_is_match,
        regex_replace, regex_replace_all,

        BytesRegex, BytesRegexBuilder,
        Captures, Lazy, Regex, RegexBuilder,
    };
}