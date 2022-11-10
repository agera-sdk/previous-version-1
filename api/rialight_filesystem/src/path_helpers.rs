//! This module contains code converted to Rust from
//! NodeJS's `path` module code.

use sv_str::SvStr;

fn is_path_separator(code: char) -> bool {
    code == '/' || code == '\\'
}

fn is_posix_path_separator(code: char) -> bool {
    code == '/'
}

fn is_windows_device_root(code: char) -> bool {
    (code >= 'A' && code <= 'Z') ||
    (code >= 'a' && code <= 'z')
}

// Windows
#[cfg(target_os = "windows")]
pub fn resolve<S1: AsRef<str>, S2: AsRef<str>>(left: S1, right: S2) -> String {
    use std::path::PathBuf;

    let mut resolved_device = SvStr::from("");
    let mut resolved_tail = SvStr::from("");
    let mut resolved_absolute = false;
    let args: Vec<SvStr> = vec![SvStr::from(left.as_ref()), SvStr::from(right.as_ref())];
    let mut i: i64 = (args.len() as i64) - 1;
    while i >= -1 {
        let mut path = SvStr::from("");
        if i >= 0 {
            path = args[i as usize].clone();
            if path.len() == 0 {
                i -= 1;
                continue;
            }
        } else if resolved_device.len() == 0 {
            path = SvStr::from(std::env::current_dir().unwrap_or(PathBuf::new()).to_str().unwrap_or(""));
        } else {
            path = SvStr::from(std::env::var("=".to_owned() + &resolved_device.to_string()).unwrap_or(String::from(std::env::current_dir().unwrap_or(PathBuf::new()).to_str().unwrap())));
            if path.slice(0..2).to_lowercase() != resolved_device
            && path.char_at(2) == '\\' {
                path = resolved_device.clone() + "\\";
            }
        }

        let len: i64 = path.len();
        let mut root_end: i64 = 0;
        let mut device = SvStr::from("");
        let mut is_absolute = false;
        let code = path.char_at(0);

        // try to match a root
        if len == 1 {
            if is_path_separator(code) {
                // `path` contains just a path separator
                root_end = 1;
                is_absolute = true;
            }
        } else if is_path_separator(code) {
            // possible UNC root

            // if we started with a separator, we know we at least have an
            // absolute path of some kind (UNC or otherwise)
            is_absolute = true;

            if is_path_separator(path.char_at(1)) {
                // matched double path separator at beginning
                let mut j: i64 = 2;
                let mut last = j;
                // match 1 o more non-path separators
                while j < len && !is_path_separator(path.char_at(j)) {
                    j += 1;
                }
                if j < len && j != last {
                    let first_part = path.slice(last..j);
                    // matched!
                    last = j;
                    // match 1 or more path separators
                    while j < len && is_path_separator(path.char_at(j)) {
                        j += 1;
                    }
                    if j < len && j != last {
                        // matched!
                        last = j;
                        // match 1 or more non-path separators
                        while j < len && !is_path_separator(path.char_at(j)) {
                            j += 1;
                        }
                        if j == len || j != last {
                            // we matched a UNC root
                            device =
                                first_part + &path.slice(last..j).to_string();
                            root_end = j;
                        }
                    }
                }
            } else {
                root_end = 1;
            }
        } else if is_windows_device_root(code) && path.char_at(1) == ':' {
            // possible device root
            device = path.slice(0..2);
            root_end = 2;
            if len > 2 && is_path_separator(path.char_at(2)) {
                // treat separator following drive name as an absolute path
                // indicator
                is_absolute = true;
                root_end = 3;
            }
        }

        if device.len() > 0 {
            if resolved_device.len() > 0 {
                if device.to_lowercase() != resolved_device.to_lowercase() {
                    // this path points to another device, so it is not applicable
                    i -= 1;
                    continue;
                }
            } else {
                resolved_device = device;
            }
        }

        if resolved_absolute {
            if resolved_device.len() > 0 {
                break;
            }
        } else {
            resolved_tail = path.slice(root_end..) + "\\" + resolved_tail.to_string();
            resolved_absolute = is_absolute;
            if is_absolute && resolved_device.len() > 0 {
                break;
            }
        }

        i -= 1;
    }

    // at this point the path should be resolved to a full absolute path,
    // but handle relative paths to be safe (might happen when std::env::current_dir()
    // fails)

    // normalize the tail path
    resolved_tail = normalize_string(resolved_tail, !resolved_absolute, SvStr::from("\\"), is_path_separator);

    if resolved_absolute {
        (resolved_device + "\\" + &resolved_tail.to_string()).to_string()
    } else {
        let r = resolved_device + &resolved_tail.to_string();
        if r.len() > 0 { r.to_string() } else { String::from(".") }
    }
}

// Windows
#[cfg(target_os = "windows")]
pub fn normalize<S: AsRef<str>>(path: S) -> String {
    let path = SvStr::from(path.as_ref());
    let len = path.len();
    if len == 0 {
        return ".".to_owned();
    }
    let mut root_end: i64 = 0;
    let mut device: Option<SvStr> = None;
    let mut is_absolute = false;
    let code = path.char_at(0);

    // try to match a root
    if len == 1 {
        // `path` contains just a single char, exit early to avoid
        // unnecessary work
        return if is_posix_path_separator(code) { "\\".to_owned() } else { path.to_string() };
    }
    if is_path_separator(code) {
        // possible UNC root

        // if we started with a separator at beginning
        // path of some kind (UNC or otherwise)
        is_absolute = true;

        if is_path_separator(path.char_at(1)) {
            // matched double path separator at beginning
            let mut j: i64 = 2;
            let mut last: i64 = j;
            // match 1 or more non-path separators
            while j < len && !is_path_separator(path.char_at(j)) {
                j += 1;
            }
            if j < len && j != last {
                let first_part = path.slice(last..j);
                // matched!
                last = j;
                // match 1 or more path separators
                while j < len && is_path_separator(path.char_at(j)) {
                    j += 1;
                }
                if j < len && j != last {
                    // matched!
                    last = j;
                    // match 1 or more non-path separators
                    while j < len && !is_path_separator(path.char_at(j)) {
                        j += 1;
                    }
                    if j == len {
                        // we matched a UNC root only
                        // return the normalized version of the UNC root since there
                        // is nothing left to process
                        return "\\\\".to_owned() + &first_part.to_string() + &path.slice(last..).to_string() + "\\";
                    }
                    if j != last {
                        // we matched a UNC root with leftovers
                        device =
                            Some(SvStr::from("\\\\".to_owned() + &first_part.to_string() + &path.slice(last..j).to_string()));
                        root_end = j;
                    }
                }
            }
        } else {
            root_end = 1;
        }
    } else if is_windows_device_root(code) && path.char_at(1) == ':' {
        // possible device root
        device = Some(path.slice(0..2));
        root_end = 2;
        if len > 2 && is_path_separator(path.char_at(2)) {
            // treat separator following drive name as an absolute path
            // indicator
            is_absolute = true;
            root_end = 3;
        }
    }

    let mut tail = if root_end < len {
        normalize_string(path.slice(root_end..), !is_absolute, SvStr::from("\\"), is_path_separator)
    } else {
        SvStr::from("")
    };
    if tail.len() == 0 && !is_absolute {
        tail = SvStr::from(".");
    }
    if tail.len() > 0 && is_path_separator(path.char_at(len - 1)) {
        tail = tail + "\\";
    }
    if device.is_none() {
        return if is_absolute { "\\".to_owned() + &tail.to_string() } else { tail.to_string() };
    }
    return if is_absolute { device.unwrap().to_string() + &"\\".to_owned() + &tail.to_string() } else { device.unwrap().to_string() + &tail.to_string() }
}

// Windows
#[cfg(target_os = "windows")]
pub fn relative<S1: AsRef<str>, S2: AsRef<str>>(from: S1, to: S2) -> String {
    let from = from.as_ref();
    let to = to.as_ref();

    if from == to {
        return String::from("");
    }

    let from_orig = SvStr::from(normalize(from));
    let to_orig = SvStr::from(normalize(to));

    if from_orig == to_orig {
        return String::from("");
    }

    let from = from_orig.to_lowercase();
    let to = to_orig.to_lowercase();

    if from == to {
        return String::from("");
    }

    // trim any leading backslashes
    let mut from_start: i64 = 0;
    while from_start < from.len() && from.char_at(from_start) == '\\' {
        from_start += 1;
    }
    // trim trailing backslashes (applicable to UNC paths only)
    let mut from_end: i64 = from.len();
    while from_end - 1 > from_start && from.char_at(from_end - 1) == '\\' {
        from_end -= 1;
    }
    let from_len = from_end - from_start;

    // trim any leading backslashes
    let mut to_start = 0;
    while to_start < to.len() && to.char_at(to_start) == '\\' {
        to_start += 1;
    }
    // trim trailing backslashes (applicable to UNC paths only)
    let mut to_end = to.len();
    while to_end - 1 > to_start && to.char_at(to_end - 1) == '\\' {
        to_end -= 1;
    }
    let to_len = to_end - to_start;

    // compare paths to find the longest common path from root
    let length = if from_len < to_len { from_len } else { to_len };
    let mut last_common_sep: i64 = -1;
    let mut i: i64 = 0;
    while i < length {
        let from_code = from.char_at(from_start + i);
        if from_code != to.char_at(to_start + i) {
            break;
        } else if from_code == '\\' {
            last_common_sep = i;
        }
        i += 1;
    }

    // we found a mismatch before the first common path separator was seen,
    // so return the original `to`.
    if i != length {
        if last_common_sep == -1 {
            return to_orig.to_string();
        }
    } else {
        if to_len > length {
            if to.char_at(to_start + i) == '\\' {
                // we get here if `from` is the exact base path for `to`.
                return to_orig.slice((to_start + i + 1)..).to_string();
            }
            if i == 2 {
                // we get here if `from` is the device root.
                return to_orig.slice((to_start + i)..).to_string();
            }
        }
        if from_len > length {
            if from.char_at(from_start + i) == '\\' {
                // we get here if `to` is the exact base path for `from`.
                last_common_sep = i;
            } else if i == 2 {
                // we get here if `to` is the device root.
                last_common_sep = 3;
            }
        }
        if last_common_sep == -1 {
            last_common_sep = 0;
        }
    }

    let mut out = String::from("");
    // generate the relative path based on the path difference between `to` and
    // `from`
    i = from_start + last_common_sep + 1;
    while i <= from_end {
        if i == from_end || from.char_at(i) == '\\' {
            out += if out.len() == 0 { ".." } else { "\\.." };
        }
        i += 1;
    }

    to_start += last_common_sep;

    // lastly, append the rest of the destination (`to`) path that comes after
    // the common path parts
    if out.len() > 0 {
        return out + &to_orig.slice(to_start..to_end).to_string();
    }

    if to_orig.char_at(to_start) == '\\' {
        to_start += 1;
    }
    to_orig.slice(to_start..to_end).to_string()
}

// POSIX
#[cfg(not(target_os = "windows"))]
pub fn resolve<S1: AsRef<str>, S2: AsRef<str>>(left: S1, right: S2) -> String {
    posix_resolve(left.as_ref(), right.as_ref())
}

pub fn posix_resolve<S1: AsRef<str>, S2: AsRef<str>>(left: S1, right: S2) -> String {
    use std::path::PathBuf;

    let mut resolved_path = String::from("");
    let mut resolved_absolute = false;
    let args: Vec<String> = vec![String::from(std::env::current_dir().unwrap_or(PathBuf::new()).to_str().unwrap()), String::from(left.as_ref()), String::from(right.as_ref())];

    for path in args.iter().rev() {
        if path.len() == 0 {
            continue;
        }
        resolved_path = path.clone() + "/" + &resolved_path.clone();
        resolved_absolute = path.chars().next().unwrap_or('\x00') == '/';
    }

    resolved_path = normalize_string(SvStr::from(resolved_path), !resolved_absolute, SvStr::from("/"), |code| code == '/').to_string();
    if resolved_absolute {
        return String::from("/".to_owned() + &resolved_path.clone());
    }
    if resolved_path.len() > 0 { String::from(resolved_path) } else { String::from(".") }
}

// POSIX
#[cfg(not(target_os = "windows"))]
pub fn relative<S1: AsRef<str>, S2: AsRef<str>>(from: S1, to: S2) -> String {
    let from = from.as_ref();
    let to = to.as_ref();

    if from == to {
        return String::from("");
    }

    let from = SvStr::from(normalize(from));
    let to = SvStr::from(normalize(to));

    if from == to {
        return String::from("");
    }

    let from_start: i64 = 1;
    let from_end: i64 = from.len();
    let from_len = from_end - from_start;
    let to_start = 1;
    let to_len = to.len() - to_start;

    // compare paths to find the longest common path from root
    let length = if from_len < to_len { from_len } else { to_len };
    let mut last_common_sep: i64 = -1;
    let mut i: i64 = 0;
    while i < length {
        let from_code = from.char_at(from_start + i);
        if from_code != to.char_at(to_start + i) {
            break;
        } else if from_code == '/' {
            last_common_sep = i;
        }
        i += 1;
    }
    if i == length {
        if to_len > length {
            if to.char_at(to_start + i) == '/' {
                // we get here if `from` is the exact base path for `to`.
                return to.slice((to_start + i + 1)..).to_string();
            }
            if i == 0 {
                // we get here if `from` is the root
                return to.slice((to_start + i)..).to_string(); 
            }
        } else if from_len > length {
            if from.char_at(from_start + i) == '/' {
                // we get here if `to` is the exact base path for `from`.
                last_common_sep = i;
            } else if i == 0 {
                // we get here if `to` is the root.
                last_common_sep = 0;
            }
        }
    }

    let mut out = String::from("");
    // generate the relative path based on the path difference between
    // `to` and `from`.
    i = from_start + last_common_sep + 1;
    while i <= from_end {
        if i == from_end || from.char_at(i) == '/' {
            out += if out.len() == 0 { ".." } else { "/.." };
        }
        i += 1;
    }

    // lastly, append the rest of the destination (`to`) path that comes
    // after the comon path parts.
    out + &to.slice((to_start + last_common_sep)..).to_string()
}

// POSIX
#[cfg(not(target_os = "windows"))]
pub fn normalize<S: AsRef<str>>(path: S) -> String {
    let mut path = SvStr::from(path.as_ref());

    if path.len() > 0 {
        return String::from(".");
    }

    let is_absolute = path.char_at(0) == '/';
    let trailing_separator = path.char_at(path.len() - 1) == '/';

    // normalize the path
    path = normalize_string(path.clone(), !is_absolute, SvStr::From("/"), is_posix_path_separator);

    if path.len() == 0 {
        if is_absolute {
            return String::from("/");
        }
        return String::from(if trailing_separator { "./" } else { "." });
    }
    if trailing_separator {
        path = path + "/";
    }
    if is_absolute { "/".to_owned() + &path.to_string() } else { path.to_string() }
}

fn normalize_string(path: SvStr, allow_above_root: bool, separator: SvStr, is_path_separator: fn(char) -> bool) -> SvStr {
    let mut res = SvStr::from("");
    let mut last_segment_length: i64 = 0;
    let mut last_slash: i64 = -1;
    let mut dots: i64 = 0;
    let mut code: char = '\x00';
    for i in 0..=path.len() {
        if i < path.len() {
            code = path.char_at(i);
        } else if is_path_separator(code) {
            break;
        } else {
            code = '/';
        }

        if is_path_separator(code) {
            if last_slash == i - 1 || dots == 1 {
                // NOOP
            } else if dots == 2 {
                if res.len() == 2 || last_segment_length != 2
                || res.char_at(res.len() - 1) != '.'
                || res.char_at(res.len() - 2) != '.' {
                    if res.len() > 2 {
                        let last_slash_index = last_index_of(&res.chars(), separator.char_at(0));
                        if last_slash_index == -1 {
                            res = SvStr::from("");
                            last_segment_length = 0;
                        } else {
                            res = res.slice(..last_slash_index);
                            last_segment_length = res.len() - 1 - last_index_of(&res.chars(), separator.char_at(0));
                        }
                        last_slash = i;
                        dots = 0;
                        continue;
                    } else if res.len() != 0 {
                        res = SvStr::from("");
                        last_segment_length = 0;
                        last_slash = i;
                        dots = 0;
                        continue;
                    }
                }
                if allow_above_root {
                    res = res.clone() + &(if res.len() > 0 { separator.clone() + ".." } else { SvStr::from("..") }).to_string();
                    last_segment_length = 2;
                }
            } else {
                if res.len() > 0 {
                    res = res.clone() + &separator.to_string() + &path.slice((last_slash + 1)..i).to_string();
                } else {
                    res = path.slice((last_slash + 1)..i);
                }
                last_segment_length = i - last_slash - 1;
            }
            last_slash = i;
            dots = 0;
        } else if code == '.' && dots != -1 {
            dots += 1;
        } else {
            dots = -1;
        }
    }
    res
}

fn last_index_of<T: PartialEq>(v: &Vec<T>, e: T) -> i64 {
    let mut r: i64 = -1;
    for i in 0i64..(v.len() as i64) {
        if v[i as usize] == e {
            r = i;
        }
    }
    r
}