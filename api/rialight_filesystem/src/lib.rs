#![allow(unused_assignments)]

use std::{path::{Path, PathBuf}, time::SystemTime};
use lazy_regex::{regex_find, regex_replace, regex_is_match};
use sv_str::SvStr;

mod path_helpers;

pub static mut APPLICATION_DIRECTORY: Option<String> = None;
pub static mut APPLICATION_STORAGE_DIRECTORY: Option<String> = None;

pub type IoError = std::io::Error;
pub type IoErrorKind = std::io::ErrorKind;

/// Represents a path to a file or directory.
///
/// # Constructing a `File` object
///
/// `File` can be constructed either with a path or a
/// URL. The following URL schemes are supported:
/// - `file:`
/// - `app:` file in the application installation directory
/// - `app-storage:` file in the application private directory
///
/// The `File` constructor performs implicit normalization of the
/// given path argument.
/// 
#[derive(Clone, Eq, PartialEq)]
pub struct File {
    m_path: String,
    m_scheme: FileScheme,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum FileScheme {
    File,
    App,
    AppStorage,
}

impl FileScheme {
    pub fn prefix(&self) -> String {
        String::from(match self {
            FileScheme::File => "file:",
            FileScheme::App => "app:",
            FileScheme::AppStorage => "app-storage:",
        })
    }
}

impl File {
    /// Constructs a new `File` object.
    pub fn new<T: AsRef<str>>(url_or_path: T) -> Self {
        let url_or_path = String::from(url_or_path.as_ref());
        let mut path = String::from("");
        let mut scheme: FileScheme = FileScheme::File;

        if url_or_path.starts_with("file:") {
            path = url_or_path[5..].to_owned();
        } else if url_or_path.starts_with("app:") {
            path = url_or_path[4..].to_owned();
            scheme = FileScheme::App;
        } else if url_or_path.starts_with("app-storage:") {
            path = url_or_path[12..].to_owned();
            scheme = FileScheme::AppStorage;
        } else {
            path = url_or_path.to_owned();
        }
        #[cfg(target_os = "windows")]
        {
            if path.starts_with("//") {
                path = SvStr::from(path).slice(2..).to_string();
            } else if path.starts_with("/") {
                path = SvStr::from(path).slice(1..).to_string();
            }
        }
        #[cfg(not(target_os = "windows"))]
        if path.starts_with("//") {
            path = SvStr::from(path).slice(1..).to_string();
        }
        if scheme == FileScheme::File {
            path = path_helpers::resolve(path, "");
        } else {
            let sv_path = SvStr::from(path.clone());
            if sv_path.char_at(0) != '/' {
                path = "/".to_owned() + &path;
            }
            path = path_helpers::posix_resolve(path, "");
        }
        File { m_scheme: scheme, m_path: path }
    }

    /// The last portion of this path.
    pub fn name(&self) -> String {
        if !regex_is_match!(r"[\\/]", self.native_path().as_ref()) {
            return self.native_path();
        }
        SvStr::from(regex_find!(r"[\\/].+$", self.native_path().as_ref()).unwrap_or("/").to_owned()).slice(1..).to_string()
    }

    /// The last portion of this path, excluding the given suffix.
    pub fn name_without_suffix<S: AsRef<str>>(&self, suffix: S) -> String {
        let s = SvStr::from(self.name());
        let suffix = SvStr::from(suffix.as_ref());
        (if s.ends_with(&suffix.to_string()) { s.slice(..(s.len() - suffix.len())) } else { s }).to_string()
    }

    /// The filename extension. This includes the dot.
    pub fn extension(&self) -> String {
        regex_find!(r"[\\.].+$", self.native_path().as_ref()).unwrap_or("").to_owned()
    }

    /// Returns a reference to the application installation directory.
    /// This is equivalent to the URL `app://`.
    pub fn application_directory() -> Self {
        File { m_scheme: FileScheme::App, m_path: String::from("") }
    }

    /// Returns a reference to the application private directory.
    /// This is equivalent to the URL `app-storage://`.
    pub fn application_storage_directory() -> Self {
        File { m_scheme: FileScheme::AppStorage, m_path: String::from("") }
    }

    /// The URL for this file path.
    pub fn url(&self) -> String {
        let path = self.m_path.replace("\\", "/");
        let mut scheme_slashes = "";
        if !path.starts_with("//") {
            if path.starts_with("/") {
                scheme_slashes = "/";
            } else {
                scheme_slashes = "//";
            }
        }
        self.m_scheme.prefix() + scheme_slashes + &path
    }

    /// The full path in the host operating system representation.
    pub fn native_path(&self) -> String {
        self.m_path.clone()
    }

    /// Resolves relative path.
    pub fn resolve_path<S: AsRef<str>>(&self, arg: S) -> Self {
        let r = if self.m_scheme == FileScheme::File {
            path_helpers::resolve(&self.m_path.clone(), arg.as_ref())
        } else {
            path_helpers::posix_resolve(&self.m_path.clone(), arg.as_ref())
        };
        File {
            m_scheme: self.m_scheme,
            m_path: r,
        }
    }

    /// Relative path from the `File` object to another `File` object.
    pub fn relative_path(&self, another: &File) -> String {
        path_helpers::relative(self.native_path(), another.native_path())
    }

    /// The user downloads directory.
    pub fn downloads_directory() -> Option<File> {
        if let Some(r) = dirs::download_dir() {
            if let Some(r) = r.to_str() {
                return Some(File::new(r));
            }
        }
        None
    }

    /// The user documents directory.
    pub fn documents_directory() -> Option<File> {
        if let Some(r) = dirs::document_dir() {
            if let Some(r) = r.to_str() {
                return Some(File::new(r));
            }
        }
        None
    }

    /// The executable directory.
    pub fn executable_directory() -> Option<File> {
        if let Some(r) = dirs::executable_dir() {
            if let Some(r) = r.to_str() {
                return Some(File::new(r));
            }
        }
        None
    }

    // The user's home directory.
    pub fn user_directory() -> Option<File> {
        if let Some(r) = dirs::home_dir() {
            if let Some(r) = r.to_str() {
                return Some(File::new(r));
            }
        }
        None
    }

    /// The user pictures directory.
    pub fn pictures_directory() -> Option<File> {
        if let Some(r) = dirs::picture_dir() {
            if let Some(r) = r.to_str() {
                return Some(File::new(r));
            }
        }
        None
    }

    /// The user videos directory.
    pub fn videos_directory() -> Option<File> {
        if let Some(r) = dirs::video_dir() {
            if let Some(r) = r.to_str() {
                return Some(File::new(r));
            }
        }
        None
    }

    /// The application's working directory (used primarily for command-line applications).
    pub fn working_directory() -> Option<File> {
        if let Ok(r) = std::env::current_dir() {
            if let Some(r) = r.to_str() {
                return Some(File::new(r));
            }
        }
        None
    }

    /// Native path of the `File` object. If the `File` object
    /// was constructed from `app:` or `app-storage:` scheme,
    /// then its path is resolved to the internal application directory.
    pub fn application_based_native_path(&self) -> PathBuf {
        PathBuf::from(self.to_path_object())
    }

    fn to_path_object(&self) -> String {
        if self.m_scheme == FileScheme::App {
            let l = File::new(unsafe {APPLICATION_DIRECTORY.clone()}.unwrap_or("".to_owned()));
            let r = regex_replace!(r"^[\\/]", self.native_path().as_ref(), |_| "").to_owned().to_string();
            l.resolve_path(&r).native_path().clone()
        } else if self.m_scheme == FileScheme::AppStorage {
            let l = File::new(unsafe {APPLICATION_STORAGE_DIRECTORY.clone()}.unwrap_or("".to_owned()));
            let r = regex_replace!(r"^[\\/]", self.native_path().as_ref(), |_| "").to_owned().to_string();
            l.resolve_path(&r).native_path().clone()
        } else {
            self.native_path().clone()
        }
    }

    /// Determines whether the referenced path exists.s
    pub fn exists(&self) -> bool {
        Path::new(&self.to_path_object()).exists()
    }

    /// Determines whether the referenced path is a directory.
    pub fn is_directory(&self) -> bool {
        Path::new(&self.to_path_object()).is_dir()
    }

    /// Determines whether the referenced path is a file.
    pub fn is_file(&self) -> bool {
        Path::new(&self.to_path_object()).is_file()
    }

    /// Determines whether the referenced path is a symbolic link.
    pub fn is_symbolic_link(&self) -> bool {
        Path::new(&self.to_path_object()).is_symlink()
    }

    /// The directory that contains the file or directory referenced by the `File` object.
    ///
    /// This property is identical to the return value of `resolve_path("..")`
    /// except that the parent of a root directory is `None`.
    pub fn parent(&self) -> Option<File> {
        let r = self.resolve_path("..");
        let p = SvStr::from(r.native_path());
        if p.len() == 0
        || p == SvStr::from(".")
        || p == SvStr::from("/")
        || p == SvStr::from("\\")
        {
            return None;
        }
        Some(r)
    }

    /// The host operating system's path component separator character.
    pub fn separator() -> String {
        #[cfg(target_os = "windows")] {
            return "\\".to_owned();
        }
        #[cfg(not(target_os = "windows"))] {
            return "/".to_owned();
        }
    }

    /// Returns a canonicalization of the `File` path.
    pub fn canonicalize(&self) -> File {
        if let Ok(r) = Path::new(&self.to_path_object()).canonicalize() {
            if let Some(r) = r.to_str() {
                return File { m_scheme: FileScheme::File, m_path: r.to_owned() };
            }
        }
        self.clone()
    }

    /// Returns a canonicalization of the `File` path.
    pub async fn canonicalize_async(&self) -> File {
        if let Ok(r) = tokio::fs::canonicalize(&self.to_path_object()).await {
            if let Some(r) = r.to_str() {
                return File { m_scheme: FileScheme::File, m_path: r.to_owned() };
            }
        }
        self.clone()
    }

    /// Copies the file at the location specified by the `File` object to the location specified by the `new_location` parameter.
    /// 
    /// This method will overwrite the contents of `new_location`.
    pub fn copy_to(&self, new_location: &File) -> Result<(), IoError> {
        std::fs::copy(&self.to_path_object(), new_location.to_path_object())?;
        Ok(())
    }

    /// Copies the file at the location specified by the `File` object to the location specified by the `new_location` parameter.
    /// 
    /// This method will overwrite the contents of `new_location`.
    pub async fn copy_to_async(&self, new_location: &File) -> Result<(), IoError> {
        tokio::fs::copy(&self.to_path_object(), new_location.to_path_object()).await?;
        Ok(())
    }

    /// Creates the specified directory and any necessary parent directories.
    /// If the directory already exists, no action is taken.
    pub fn create_directory(&self) -> Result<(), IoError> {
        std::fs::create_dir_all(&self.to_path_object())?;
        Ok(())
    }

    /// Creates the specified directory and any necessary parent directories.
    /// If the directory already exists, no action is taken.
    pub async fn create_directory_async(&self) -> Result<(), IoError> {
        tokio::fs::create_dir_all(&self.to_path_object()).await?;
        Ok(())
    }

    /// Read file contents as bytes.
    pub fn read_bytes(&self) -> Result<Vec<u8>, IoError> {
        Ok(std::fs::read(self.to_path_object())?)
    }

    /// Read file contents as bytes.
    pub async fn read_bytes_async(&self) -> Result<Vec<u8>, IoError> {
        Ok(tokio::fs::read(self.to_path_object()).await?)
    }

    /// Read file contents as UTF-8 string.
    pub fn read_utf8(&self) -> Result<String, IoError> {
        Ok(std::fs::read_to_string(self.to_path_object())?)
    }

    /// Read file contents as UTF-8 string.
    pub async fn read_utf8_async(&self) -> Result<String, IoError> {
        Ok(tokio::fs::read_to_string(self.to_path_object()).await?)
    }

    /// Returns a vector of `File` objects corresponding to files and directories
    /// in the directory represented by the `File` object.
    pub fn get_directory_listing(&self) -> Result<Vec<File>, IoError> {
        let mut r = Vec::<File>::new();
        for entry in std::fs::read_dir(self.to_path_object())? {
            let entry = entry?;
            r.push(File::new(entry.path().to_str().unwrap_or("")));
        }
        Ok(r.clone())
    }

    /// Deletes empty directory.
    pub fn delete_empty_directory(&self) -> Result<(), IoError> {
        std::fs::remove_dir(self.to_path_object())?;
        Ok(())
    }

    /// Deletes empty directory.
    pub async fn delete_empty_directory_async(&self) -> Result<(), IoError> {
        tokio::fs::remove_dir(self.to_path_object()).await?;
        Ok(())
    }

    /// Deletes directory after deleting all its contents.
    pub fn delete_all_directory(&self) -> Result<(), IoError> {
        std::fs::remove_dir_all(self.to_path_object())?;
        Ok(())
    }

    /// Deletes directory after deleting all its contents.
    pub async fn delete_all_directory_async(&self) -> Result<(), IoError> {
        tokio::fs::remove_dir_all(self.to_path_object()).await?;
        Ok(())
    }

    /// Deletes file.
    pub fn delete_file(&self) -> Result<(), IoError> {
        std::fs::remove_file(self.to_path_object())?;
        Ok(())
    }

    /// Deletes file.
    pub async fn delete_file_async(&self) -> Result<(), IoError> {
        tokio::fs::remove_file(self.to_path_object()).await?;
        Ok(())
    }

    /// Rename a file or directory to a new name specified by the `to` parameter,
    /// replacing the original file if to already exists.
    pub fn rename(&self, to: &File) -> Result<(), IoError> {
        std::fs::rename(self.to_path_object(), to.to_path_object())?;
        Ok(())
    }

    /// Rename a file or directory to a new name specified by the `to` parameter,
    /// replacing the original file if to already exists.
    pub async fn rename_async(&self, to: &File) -> Result<(), IoError> {
        tokio::fs::rename(self.to_path_object(), to.to_path_object()).await?;
        Ok(())
    }

    /// Writes bytes to a file.
    pub fn write<B: AsRef<[u8]>>(&self, b: B) -> Result<(), IoError> {
        std::fs::write(self.to_path_object(), b.as_ref())?;
        Ok(())
    }

    /// Writes bytes to a file.
    pub async fn write_async<B: AsRef<[u8]>>(&self, b: B) -> Result<(), IoError> {
        tokio::fs::write(self.to_path_object(), b.as_ref()).await?;
        Ok(())
    }

    /// Creation date.
    pub fn creation_date(&self) -> Result<SystemTime, IoError> {
        let r = std::fs::metadata(self.to_path_object())?;
        Ok(r.created()?)
    }

    /// Creation date.
    pub async fn creation_date_async(&self) -> Result<SystemTime, IoError> {
        let r = tokio::fs::metadata(self.to_path_object()).await?;
        Ok(r.created()?)
    }

    /// Modification date.
    pub fn modification_date(&self) -> Result<SystemTime, IoError> {
        let r = std::fs::metadata(self.to_path_object())?;
        Ok(r.modified()?)
    }

    /// Modification date.
    pub async fn modification_date_async(&self) -> Result<SystemTime, IoError> {
        let r = tokio::fs::metadata(self.to_path_object()).await?;
        Ok(r.modified()?)
    }

    /// Size of the file in bytes.
    pub fn size(&self) -> Result<i64, IoError> {
        let r = std::fs::metadata(self.to_path_object())?;
        Ok(r.len() as i64)
    }

    /// Size of the file in bytes.
    pub async fn size_async(&self) -> Result<i64, IoError> {
        let r = tokio::fs::metadata(self.to_path_object()).await?;
        Ok(r.len() as i64)
    }
}