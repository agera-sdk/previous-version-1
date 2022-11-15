# Localization

Rialight projects by default already ship with localization resources, located at `res/lang`.

The file `res/lang/en-US/_.json` defines message identifiers starting with the `_.` prefix. For example, `_.json` could be the following:

```json
{
  "hello_world": "Hello, world!"
}
```

This message can be referred as `_.hello_world`.

## Locale bundle

The module [`rialight::localization`](https://crates.io/crates/rialight_localization) is used for resolving message strings and working with locales in general.

The module `crate::localization`, which is defined at the file `src/localization.rs`, contains setup for a global `LocaleBundle` instance. Edit the file as needed. For example, you may add more `supported_locales`.

The module `crate::localization` provides three utilities:
- `B` (a `LocaleBundle` instance),
- `t` (translate function) and
- `tf` (translate function with formatting).

Here is an example using `crate::localization`:

```rust
use crate::localization;
use rialight::localization::{bundle_vars};

println!(localization::t("_.hello_world"));
println!(localization::tf("_.parameterized_msg", vec![ &bundle_vars!{
    "x" => "foo"
} ]));
// refer to "_.msg_female"
println!(localization::tf("_.msg", vec![ &"female" ]));
```