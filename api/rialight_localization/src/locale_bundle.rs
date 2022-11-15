use std::{cell::{Cell, RefCell}, collections::{HashMap, HashSet}, sync::Arc};
use maplit::{hashmap, hashset};
use super::{Language};
use lazy_static::lazy_static;
use lazy_regex::regex;
use rialight_util::AnyStringType;

/// Creates a `HashMap<String, String>` from a list of key-value pairs.
///
/// ## Example
///
/// ```
/// use rialight_localization::bundle_vars;
/// fn main() {
///     let map = bundle_vars!{
///         "a" => "foo",
///         "b" => "bar",
///     };
///     assert_eq!(map[&"a".to_owned()], "foo");
///     assert_eq!(map[&"b".to_owned()], "bar");
///     assert_eq!(map.get(&"c".to_owned()), None);
/// }
/// ```
#[macro_export]
macro_rules! bundle_vars {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(bundle_vars!(@single $rest)),*]));

    ($($key:expr => $value:expr,)+) => { bundle_vars!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            let r_cap = bundle_vars!(@count $($key),*);
            let mut r_map = ::std::collections::HashMap::<String, String>::with_capacity(r_cap);
            $(
                let _ = r_map.insert($key.to_string(), $value.to_string());
            )*
            r_map
        }
    };
}

pub struct LocaleBundle {
    m_current_locale: Option<Language>,
    m_locale_path_components: Arc<HashMap<Language, String>>,
    m_supported_locales: Arc<HashSet<Language>>,
    m_default_locale: Language,
    m_fallbacks: Arc<HashMap<Language, Vec<Language>>>,
    m_assets: Arc<HashMap<Language, serde_json::Value>>,
    m_assets_src: String,
    m_assets_base_file_names: Vec<String>,
    m_assets_clean_unused: bool,
    m_assets_load_method: LocaleBundleLoadMethod,
}

impl LocaleBundle {
    /// Constructs a `LocaleBundle` object.
    pub fn new(options: &LocaleBundleOptions) -> Self {
        let mut locale_path_components = HashMap::<Language, String>::new();
        let mut supported_locales = HashSet::<Language>::new();
        for code in options.m_supported_locales.borrow().iter() {
            let locale_parse = Language::parse(code.clone()).unwrap();
            locale_path_components.insert(locale_parse.clone(), code.clone());
            supported_locales.insert(locale_parse);
        }
        let mut fallbacks = HashMap::<Language, Vec<Language>>::new();
        for (k, v) in options.m_fallbacks.borrow().iter() {
            fallbacks.insert(Language::parse(k.clone()).unwrap(), v.iter().map(|s| Language::parse(s.clone()).unwrap()).collect());
        }
        let default_locale = options.m_default_locale.borrow().clone();
        Self {
            m_current_locale: None,
            m_locale_path_components: Arc::new(locale_path_components),
            m_supported_locales: Arc::new(supported_locales),
            m_default_locale: Language::parse(default_locale).unwrap(),
            m_fallbacks: Arc::new(fallbacks),
            m_assets: Arc::new(HashMap::new()),
            m_assets_src: options.m_assets.borrow().m_src.borrow().clone(),
            m_assets_base_file_names: options.m_assets.borrow().m_base_file_names.borrow().iter().map(|s| s.clone()).collect(),
            m_assets_clean_unused: options.m_assets.borrow().m_clean_unused.get(),
            m_assets_load_method: options.m_assets.borrow().m_load_method.get(),
        }
    }

    /// Returns a set of supported locale codes, reflecting
    /// the ones that were specified when constructing the `LocaleBundle`.
    pub fn supported_locales(&self) -> HashSet<Language> {
        self.m_supported_locales.as_ref().clone()
    }

    /// Returns `true` if the locale is one of the supported locales
    /// that were specified when constructing the `LocaleBundle`,
    /// otherwise `false`.
    pub fn supports_locale(&self, arg: &Language) -> bool {
        self.m_supported_locales.contains(arg)
    }

    /// Returns the currently loaded locale.
    pub fn current_locale(&self) -> Option<Language> {
        self.m_current_locale.clone()
    }

    /// Returns the currently loaded locale followed by its fallbacks or empty if no locale is loaded.
    pub fn current_locale_seq(&self) -> HashSet<Language> {
        if let Some(c) = self.current_locale() {
            let mut r: HashSet<Language> = hashset![c.clone()];
            self.enumerate_fallbacks(c.clone(), &mut r);
            return r;
        }
        hashset![]
    }

    /// Attempts to load the specified locale and its fallbacks.
    /// If any resource fails to load, the method returns `false`, otherwise `true`.
    pub async fn update_locale(&mut self, new_locale: Language) -> bool {
        self.load(Some(new_locale)).await
    }

    /// Attempts to load a locale and its fallbacks.
    /// If the locale argument is specified, it is loaded.
    /// Otherwise, if there is a default locale, it is loaded, and if not,
    /// the method panics.
    ///
    /// If any resource fails to load, the method returns `false`, otherwise `true`.
    pub async fn load(&mut self, mut new_locale: Option<Language>) -> bool {
        if new_locale.is_none() { new_locale = Some(self.m_default_locale.clone()); }
        let new_locale = new_locale.unwrap();
        if !self.supports_locale(&new_locale) {
            panic!("Unsupported locale {}", new_locale.tag());
        }
        let mut to_load: HashSet<Language> = hashset![new_locale.clone()];
        self.enumerate_fallbacks(new_locale.clone(), &mut to_load);

        let mut new_assets: HashMap<Language, serde_json::Value> = hashmap![];
        for locale in to_load {
            let res = self.load_single_locale(&locale).await;
            if res.is_none() {
                return false;
            }
            new_assets.insert(locale.clone(), res.unwrap());
        }
        if self.m_assets_clean_unused {
            Arc::get_mut(&mut self.m_assets).unwrap().clear();
        }

        for (locale, root) in new_assets {
            Arc::get_mut(&mut self.m_assets).unwrap().insert(locale, root);
        }
        self.m_current_locale = Some(new_locale.clone());
        // let new_locale_code = unic_langid::LanguageIdentifier::from_bytes(new_locale.clone().standard_tag().to_string().as_ref()).unwrap();

        true
    }

    async fn load_single_locale(&self, locale: &Language) -> Option<serde_json::Value> {
        let mut r = serde_json::Value::Object(serde_json::Map::new());
        match self.m_assets_load_method {
            LocaleBundleLoadMethod::FileSystem => {
                for base_name in self.m_assets_base_file_names.iter() {
                    let locale_path_comp = self.m_locale_path_components.get(locale);
                    if locale_path_comp.is_none() {
                        panic!("Fallback locale is not supported a locale: {}", locale.tag());
                    }
                    let res_path = format!("{}/{}/{}.json", self.m_assets_src, locale_path_comp.unwrap(), base_name);
                    let content = std::fs::read(res_path.clone());
                    if content.is_err() {
                        println!("Failed to load resource at {}.", res_path);
                        return None;
                    }
                    LocaleBundle::apply_deep(base_name, serde_json::from_str(String::from_utf8(content.unwrap()).unwrap().as_ref()).unwrap(), &mut r);
                }
            },
            LocaleBundleLoadMethod::Http => {
                for base_name in self.m_assets_base_file_names.iter() {
                    let locale_path_comp = self.m_locale_path_components.get(locale);
                    if locale_path_comp.is_none() {
                        panic!("Fallback locale is not supported a locale: {}", locale.tag());
                    }
                    let res_path = format!("{}/{}/{}.json", self.m_assets_src, locale_path_comp.unwrap(), base_name);
                    let content = reqwest::get(reqwest::Url::parse(res_path.clone().as_ref()).unwrap()).await;
                    if content.is_err() {
                        println!("Failed to load resource at {}.", res_path);
                        return None;
                    }
                    let content = if content.is_ok() { Some(content.unwrap().text().await) } else { None };
                    LocaleBundle::apply_deep(base_name, serde_json::from_str(content.unwrap().unwrap().as_ref()).unwrap(), &mut r);
                }
            },
        }
        Some(r)
    }

    fn apply_deep(name: &String, assign: serde_json::Value, mut output: &mut serde_json::Value) {
        let mut names: Vec<&str> = name.split("/").collect();
        let last_name = names.pop();
        for name in names {
            let r = output.get(name);
            if r.is_none() || r.unwrap().as_object().is_none() {
                let r = serde_json::Value::Object(serde_json::Map::new());
                output.as_object_mut().unwrap().insert(String::from(name), r);
            }
            output = output.get_mut(name).unwrap();
        }
        output.as_object_mut().unwrap().insert(String::from(last_name.unwrap()), assign);
    }

    fn enumerate_fallbacks(&self, locale: Language, output: &mut HashSet<Language>) {
        for list in self.m_fallbacks.get(&locale).iter() {
            for item in list.iter() {
                output.insert(item.clone());
                self.enumerate_fallbacks(item.clone(), output);
            }
        }
    }

    /// Retrieves message by identifier.
    pub fn get(&self, id: impl AnyStringType) -> String {
        self.get_formatted(id, vec![])
    }

    /// Retrieves message by identifier with formatting arguments.
    pub fn get_formatted(&self, id: impl AnyStringType, options: Vec<&dyn LocaleBundleFormatArgument>) -> String {
        let mut variables: Option<HashMap<String, String>> = None;
        let mut id = id.convert().to_owned();

        for option in options.iter() {
            if let Some(r) = option.as_str() {
                id.push('_');
                id.push_str(r);
            }
            else if let Some(r) = option.as_string() {
                id.push('_');
                id.push_str(r.as_str());
            }
            else if let Some(r) = option.as_string_map() {
                variables = Some(r.iter().map(|(k, v)| (k.clone(), v.clone())).collect());
            }
        }

        if variables.is_none() { variables = Some(HashMap::new()); }
        let variables = variables.unwrap();

        let id: Vec<String> = id.split(".").map(|s| s.to_string()).collect();
        if self.m_current_locale.is_none() {
            return id.join(".");
        }
        let r = self.get_formatted_with_locale(self.m_current_locale.clone().unwrap(), &id, &variables);
        if let Some(r) = r { r } else { id.join(".") }
    }

    fn get_formatted_with_locale(&self, locale: Language, id: &Vec<String>, vars: &HashMap<String, String>) -> Option<String> {
        let message = self.resolve_id(self.m_assets.get(&locale), id);
        if message.is_some() {
            return Some(self.apply_message(message.unwrap(), vars));
        }

        let fallbacks = self.m_fallbacks.get(&locale);
        if fallbacks.is_some() {
            for fl in fallbacks.unwrap().iter() {
                let r = self.get_formatted_with_locale(fl.clone(), id, vars);
                if r.is_some() {
                    return r;
                }
            }
        }
        None
    }

    fn apply_message(&self, message: String, vars: &HashMap<String, String>) -> String {
        // regex!(r"\$(\$|[A-Za-z0-9]+)").replace_all(&message, R { m_vars: vars }).as_ref().to_string()
        regex!(r"\$(\$|[A-Za-z0-9]+)").replace_all(&message, |s: &regex::Captures<'_>| {
            let s = s.get(0).unwrap().as_str();
            if s == "$$" {
                "$"
            } else {
                let v = vars.get(&s.to_string().replace("$", ""));
                if let Some(v) = v { v } else { "undefined" }
            }
        }).as_ref().to_string()
    }

    fn resolve_id(&self, root: Option<&serde_json::Value>, id: &Vec<String>) -> Option<String> {
        let mut r = root;
        for frag in id.iter() {
            if r.is_none() {
                return None;
            }
            r = r.unwrap().get(frag);
        }
        if r.is_none() {
            return None;
        }
        let r = r.unwrap().as_str();
        if let Some(r) = r { Some(r.to_string()) } else { None }
    }
}

impl Clone for LocaleBundle {
    /// Clones the locator, sharing the same
    /// resources.
    fn clone(&self) -> Self {
        Self {
            m_current_locale: self.m_current_locale.clone(),
            m_locale_path_components: self.m_locale_path_components.clone(),
            m_supported_locales: self.m_supported_locales.clone(),
            m_default_locale: self.m_default_locale.clone(),
            m_fallbacks: self.m_fallbacks.clone(),
            m_assets: self.m_assets.clone(),
            m_assets_src: self.m_assets_src.clone(),
            m_assets_base_file_names: self.m_assets_base_file_names.clone(),
            m_assets_clean_unused: self.m_assets_clean_unused,
            m_assets_load_method: self.m_assets_load_method,
        }
    }
}

pub trait LocaleBundleFormatArgument {
    fn as_str(&self) -> Option<&'static str> { None }
    fn as_string(&self) -> Option<String> { None }
    fn as_string_map(&self) -> Option<HashMap<String, String>> { None }
}

impl LocaleBundleFormatArgument for &'static str {
    fn as_str(&self) -> Option<&'static str> { Some(self) }
}

impl LocaleBundleFormatArgument for String {
    fn as_string(&self) -> Option<String> { Some(self.clone()) }
}

impl LocaleBundleFormatArgument for HashMap<String, String> {
    fn as_string_map(&self) -> Option<HashMap<String, String>> { Some(self.clone()) }
}

impl LocaleBundleFormatArgument for i8 { fn as_string(&self) -> Option<String> { Some(self.to_string()) } }
impl LocaleBundleFormatArgument for i16 { fn as_string(&self) -> Option<String> { Some(self.to_string()) } }
impl LocaleBundleFormatArgument for i32 { fn as_string(&self) -> Option<String> { Some(self.to_string()) } }
impl LocaleBundleFormatArgument for i64 { fn as_string(&self) -> Option<String> { Some(self.to_string()) } }
impl LocaleBundleFormatArgument for i128 { fn as_string(&self) -> Option<String> { Some(self.to_string()) } }
impl LocaleBundleFormatArgument for isize { fn as_string(&self) -> Option<String> { Some(self.to_string()) } }
impl LocaleBundleFormatArgument for u8 { fn as_string(&self) -> Option<String> { Some(self.to_string()) } }
impl LocaleBundleFormatArgument for u16 { fn as_string(&self) -> Option<String> { Some(self.to_string()) } }
impl LocaleBundleFormatArgument for u32 { fn as_string(&self) -> Option<String> { Some(self.to_string()) } }
impl LocaleBundleFormatArgument for u64 { fn as_string(&self) -> Option<String> { Some(self.to_string()) } }
impl LocaleBundleFormatArgument for u128 { fn as_string(&self) -> Option<String> { Some(self.to_string()) } }
impl LocaleBundleFormatArgument for usize { fn as_string(&self) -> Option<String> { Some(self.to_string()) } }
impl LocaleBundleFormatArgument for f32 { fn as_string(&self) -> Option<String> { Some(self.to_string()) } }
impl LocaleBundleFormatArgument for f64 { fn as_string(&self) -> Option<String> { Some(self.to_string()) } }

pub struct LocaleBundleOptions {
    m_default_locale: RefCell<String>,
    m_supported_locales: RefCell<Vec<String>>,
    m_fallbacks: RefCell<HashMap<String, Vec<String>>>,
    m_assets: RefCell<LocaleBundleOptionsForAssets>,
}

impl LocaleBundleOptions {
    pub fn new() -> Self {
        LocaleBundleOptions {
            m_default_locale: RefCell::new("en".to_string()),
            m_supported_locales: RefCell::new(vec!["en".to_string()]),
            m_fallbacks: RefCell::new(hashmap! {}),
            m_assets: RefCell::new(LocaleBundleOptionsForAssets::new()),
        }
    }

    pub fn default_locale(&self, value: impl AnyStringType) -> &Self {
        self.m_default_locale.replace(value.convert().to_owned());
        self
    }

    pub fn supported_locales(&self, list: Vec<impl AnyStringType>) -> &Self {
        self.m_supported_locales.replace(list.iter().map(|name| name.convert().to_owned()).collect());
        self
    }

    pub fn fallbacks(&self, map: HashMap<impl AnyStringType, Vec<impl AnyStringType>>) -> &Self {
        self.m_fallbacks.replace(map.iter().map(|(k, v)| (
            k.convert().to_owned(),
            v.iter().map(|s| s.convert().to_owned()).collect()
        )).collect());
        self
    }

    pub fn assets(&self, options: &LocaleBundleOptionsForAssets) -> &Self {
        self.m_assets.replace(options.clone());
        self
    }
}

pub struct LocaleBundleOptionsForAssets {
    m_src: RefCell<String>,
    m_base_file_names: RefCell<Vec<String>>,
    m_clean_unused: Cell<bool>,
    m_load_method: Cell<LocaleBundleLoadMethod>,
}

impl Clone for LocaleBundleOptionsForAssets {
    fn clone(&self) -> Self {
        Self {
            m_src: self.m_src.clone(),
            m_base_file_names: self.m_base_file_names.clone(),
            m_clean_unused: self.m_clean_unused.clone(),
            m_load_method: self.m_load_method.clone(),
        }
    }
}

impl LocaleBundleOptionsForAssets {
    pub fn new() -> Self {
        LocaleBundleOptionsForAssets {
            m_src: RefCell::new("res/lang".to_string()),
            m_base_file_names: RefCell::new(vec![]),
            m_clean_unused: Cell::new(true),
            m_load_method: Cell::new(LocaleBundleLoadMethod::Http),
        }
    }
    
    pub fn src<S: ToString>(&self, src: S) -> &Self {
        self.m_src.replace(src.to_string());
        self
    } 

    pub fn base_file_names<S: ToString>(&self, list: Vec<S>) -> &Self {
        self.m_base_file_names.replace(list.iter().map(|name| name.to_string()).collect());
        self
    }

    pub fn clean_unused(&self, value: bool) -> &Self {
        self.m_clean_unused.set(value);
        self
    }

    pub fn load_method(&self, value: LocaleBundleLoadMethod) -> &Self {
        self.m_load_method.set(value);
        self
    }
}

#[derive(Copy, Clone)]
pub enum LocaleBundleLoadMethod {
    FileSystem,
    Http,
}