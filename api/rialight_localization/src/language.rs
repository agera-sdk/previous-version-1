use std::hash::{Hash, Hasher};
use std::{sync::Arc, fmt::Debug};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use rialight_util::AnyStringType;
use super::Region;

lazy_static! {
    static ref LANGUAGE_DATA: HashMap<String, LanguageData> = {
        serde_json::from_str::<HashMap<String, LanguageData>>(include_str!("../data/language.json")).unwrap()
    };
}

lazy_static! {
    static ref INTERNED_LANGUAGES: HashMap<String, Language> = {
        let mut r = HashMap::<String, Language>::new();
        for (k, v) in LANGUAGE_DATA.iter() {
            r.insert(k.clone(), Language {
                m_abbrev: k.clone(),
                m_data: Arc::new(v.clone()),
                m_region: None,
            });
        }
        r
    };
}

#[derive(Serialize, Deserialize, Clone)]
struct LanguageData {
    /// International name
    n: String,
    /// Native name
    nn: String,
    /// Direction
    d: String,
}

/// Represents a language code.
#[derive(Clone)]
pub struct Language {
    m_abbrev: String,
    m_data: Arc<LanguageData>,
    m_region: Option<Region>,
}

impl Language {
    fn parse_tag(tag: impl AnyStringType) -> Option<(String, String)> {
        let tag = tag.convert().to_lowercase().replace("_", "-");
        let tag_split: Vec<&str> = tag.split("-").collect();
        if tag_split.len() == 0 {
            return None;
        }
        let mut language_abbrev = tag_split[0];
        let mut region_abbrev = if tag_split.len() == 2 { Some(String::from(tag_split[1])) } else { None };
        if region_abbrev.is_none() {
            if language_abbrev == "en" || language_abbrev == "us" || language_abbrev == "usa" { language_abbrev = "en"; region_abbrev = Some(String::from("us")); }
            if language_abbrev == "br" { language_abbrev = "pt"; region_abbrev = Some(String::from("br")); }
            if language_abbrev == "jp" { language_abbrev = "ja"; region_abbrev = Some(String::from("jp")); }
        }
        if region_abbrev.is_none() {
            let r = Region::parse(language_abbrev);
            if let Some(a) = r {
                region_abbrev = Some(a.to_string().to_lowercase());
            }
        }
        if region_abbrev.is_none() {
            return None;
        }
        Some((String::from(language_abbrev), region_abbrev.unwrap()))
    }

    pub fn parse(tag: impl AnyStringType) -> Option<Language> {
        let tag = Language::parse_tag(tag);
        if tag.is_none() {
            return None;
        }
        let tag = tag.unwrap();
        let lng = INTERNED_LANGUAGES.get(&tag.0);
        let region = Region::parse(tag.1);
        if lng.is_none() || region.is_none() {
            return None;
        }
        Some(Language {
            m_abbrev: tag.0.clone(),
            m_data: lng.unwrap().m_data.clone(),
            m_region: region.clone(),
        })
    }

    pub fn international_name(&self) -> String {
        self.m_data.n.clone()
    }
    pub fn native_name(&self) -> String {
        self.m_data.nn.clone()
    }
    pub fn direction(&self) -> Direction {
        if self.m_data.d == "ltr" { Direction::Ltr } else { Direction::Rtl }
    }
    pub fn tag(&self) -> String {
        self.m_abbrev.clone() + "-" + self.m_region.as_ref().unwrap().to_string().to_uppercase().as_str()
    }
    pub fn region(&self) -> Region {
        self.m_region.as_ref().unwrap().clone()
    }
}

impl PartialEq for Language {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.m_data, &other.m_data) && self.m_region.as_ref().unwrap() == other.m_region.as_ref().unwrap()
    }
}

impl Eq for Language {
}

impl Hash for Language {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tag().hash(state);
    }
}

impl ToString for Language {
    fn to_string(&self) -> String {
        self.m_abbrev.clone() + "-" + self.m_region.as_ref().unwrap().to_string().to_uppercase().as_str()
    }
}

impl Debug for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[derive(PartialEq, Eq, Debug, Hash)]
pub enum Direction {
    Ltr,
    Rtl,
}