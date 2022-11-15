# `rialight::localization`

## `LocaleBundle`

To use `LocaleBundle`, add these dependencies to `Cargo.toml`:

```toml
[dependencies]
rialight_localization = "1"
maplit = "1.0"
tokio = { version = "1", features = ["full"] }
```

Example asset located at `res/lang/en/_.json`:

```json
{
    "message_id": "Some message",
    "parameterized": "Here: $x",
    "contextual_male": "Male message",
    "contextual_female": "Female message",
    "contextual_other": "Other message",
    "qty_empty": "Empty ($number)",
    "qty_one": "One ($number)",
    "qty_multiple": "Multiple ($number)"
}
```

Example usage:

```rust
use rialight_localization::{
    LocaleBundle, LocaleBundleOptions, LocaleBundleOptionsForAssets,
    LocaleBundleLoadMethod,
    bundle_vars,
};
use maplit::hashmap;

#[tokio::main]
async fn main() {
    let mut bundle = LocaleBundle::new(
        LocaleBundleOptions::new()
            // Specify supported locale codes.
            // The form in which the locale code appears here
            // is a post-component for the assets "src" path. 
            // For example: "path/to/res/lang/en-US"
            .supported_locales(vec!["en", "pt-BR"])
            .default_locale("en-US")
            .fallbacks(hashmap! {
                "pt-BR" => vec!["en-US"],
            })
            .assets(LocaleBundleOptionsForAssets::new()
                .src("res/lang")
                .base_file_names(vec!["_"])
                // "clean_unused" indicates whether to clean previous unused locale data. 
                .clean_unused(true)
                // Specify LocaleBundleLoadMethod::FileSystem or LocaleBundleLoadMethod::Http
                .load_method(LocaleBundleLoadMethod::FileSystem))
    ); // bundle

    if !bundle.load(None).await {
        // failed to load
        return;
    }

    println!("{}", bundle.get("_.message_id"));
    println!("{}", bundle.get_formatted("_.parameterized", vec![ &bundle_vars!{
        "x" => "foo"
    } ]));
    println!("{}", bundle.get_formatted("_.contextual", vec![ &"female" ]));
}
```