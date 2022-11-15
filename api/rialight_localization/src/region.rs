use std::{collections::HashMap, fmt::Debug, hash::{Hash, Hasher}};
use rialight_util::AnyStringType;

lazy_static! {
    static ref REGION_DATA: HashMap<String, String> = {
        serde_json::from_str::<HashMap<String, String>>(include_str!("../data/region.json")).unwrap()
    };
}

#[derive(Clone)]
pub struct Region {
    m_abbrev: String,
    m_international_name: String,
}

impl Region {
    pub fn parse(abbrev: impl AnyStringType) -> Option<Region> {
        let abbrev = abbrev.convert().to_uppercase();
        let data = REGION_DATA.get(&abbrev);
        if let Some(international_name) = data { Some(Region { m_abbrev: abbrev.clone(), m_international_name: international_name.clone() }) } else { None }
    }

    pub fn international_name(&self) -> String {
        self.m_international_name.clone()
    }

    pub fn id(&self) -> String {
        self.to_string()
    }
}

impl PartialEq for Region {
    fn eq(&self, other: &Self) -> bool {
        self.m_abbrev == other.m_abbrev
    }
}

impl ToString for Region {
    fn to_string(&self) -> String {
        self.m_abbrev.to_lowercase()
    }
}

impl Debug for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Hash for Region {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id().hash(state);
    }
}