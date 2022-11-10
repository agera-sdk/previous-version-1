use lazy_regex::{regex_is_match};
use serde::{Serialize, Deserialize};
use std::{fs, path::Path};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ProjectSettings {
    #[serde(rename = "short-id")]
    pub short_id: String,
    #[serde(rename = "full-id")]
    pub full_id: String,
}

#[derive(Clone, Debug)]
pub enum ProjectSettingsError {
    NotFound,
    InvalidShortId,
    InvalidFullId,
}

pub fn is_id_valid<S: AsRef<str>>(name: S) -> bool {
    regex_is_match!(r"[a-z0-9][a-z\-0-9.]*", name.as_ref())
}

pub fn read_project_settings<S: AsRef<str>>(dir: S) -> Result<ProjectSettings, ProjectSettingsError> {
    let project_settings_path = Path::new(dir.as_ref()).join("Rialight.toml");
    if !(project_settings_path.exists() && project_settings_path.is_file()) {
        return Err(ProjectSettingsError::NotFound);
    }
    let project_settings: ProjectSettings = toml::from_str(std::str::from_utf8(&fs::read(project_settings_path).unwrap()).unwrap()).unwrap();
    if !is_id_valid(project_settings.short_id.clone()) {
        return Err(ProjectSettingsError::InvalidShortId);
    }
    if !is_id_valid(project_settings.full_id.clone()) {
        return Err(ProjectSettingsError::InvalidFullId);
    }
    Ok(project_settings)
}

pub fn prepare_build(out_dir: &'static str) {
    let _project_settings = read_project_settings("/").unwrap();

    // rialight_entry.rs
    let rialight_entry_contents = "\
use rialight::filesystem::{
    APPLICATION_DIRECTORY,
    APPLICATION_STORAGE_DIRECTORY,
};

#[cfg(debug_assertions)]
{
    APPLICATION_DIRECTORY = Some(String::from(std::env::current_dir().unwrap().to_str().unwrap()));
    APPLICATION_STORAGE_DIRECTORY = Some(String::from(concat!(env!(\"OUT_DIR\"), \"/rialight_debug_app_storage_dir\")));
}
#[cfg(not(debug_assertions))]
{
}
";

    fs::write(out_dir.to_owned() + "/rialight_entry.rs", rialight_entry_contents).unwrap();

    // rialight_debug_app_storage_dir
    drop(fs::remove_dir_all(out_dir.to_owned() + "/rialight_debug_app_storage_dir"));
    fs::create_dir_all(out_dir.to_owned() + "/rialight_debug_app_storage_dir").unwrap();
}