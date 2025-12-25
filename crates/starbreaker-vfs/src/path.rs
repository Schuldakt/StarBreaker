//! VFS path utilities


/// Normalize a VFS path
/// - Converts backslashes to forward slashes
/// - Removes redundant separators
/// - Resolves . and .. components
/// - Ensures absolute paths start with /
pub fn normalize_path(path: &str) -> String {
    let path = path.replace('\\', "/");
    let path = path.trim();
    
    let mut components = Vec::new();
    
    for component in path.split('/') {
        match component {
            "" | "." => continue,
            ".." => {
                components.pop();
            }
            _ => components.push(component),
        }
    }
    
    if components.is_empty() {
        "/".to_string()
    } else {
        format!("/{}", components.join("/"))
    }
}

/// Split path into directory and filename
pub fn split_path(path: &str) -> (&str, &str) {
    let path = path.trim_start_matches('/');
    
    if let Some(pos) = path.rfind('/') {
        (&path[..pos], &path[pos + 1..])
    } else {
        ("", path)
    }
}

/// Get parent directory of a path
pub fn parent_path(path: &str) -> Option<String> {
    let normalized = normalize_path(path);
    
    if normalized == "/" {
        return None;
    }
    
    if let Some(pos) = normalized.rfind('/') {
        if pos == 0 {
            Some("/".to_string())
        } else {
            Some(normalized[..pos].to_string())
        }
    } else {
        Some("/".to_string())
    }
}

/// Get filename from path
pub fn filename(path: &str) -> &str {
    let path = path.trim_end_matches('/');
    
    if let Some(pos) = path.rfind('/') {
        &path[pos + 1..]
    } else {
        path
    }
}

/// Join path components
pub fn join_paths(base: &str, relative: &str) -> String {
    if relative.starts_with('/') {
        return normalize_path(relative);
    }
    
    let base = base.trim_end_matches('/');
    normalize_path(&format!("{}/{}", base, relative))
}

/// Check if path matches a glob pattern
/// Supports * (any chars) and ? (single char)
pub fn glob_match(pattern: &str, path: &str) -> bool {
    glob_match_impl(pattern.as_bytes(), path.as_bytes())
}

fn glob_match_impl(pattern: &[u8], text: &[u8]) -> bool {
    let mut p = 0;
    let mut t = 0;
    let mut star_p = None;
    let mut star_t = None;
    
    while t < text.len() {
        if p < pattern.len() {
            match pattern[p] {
                b'*' => {
                    star_p = Some(p);
                    star_t = Some(t);
                    p += 1;
                    continue;
                }
                b'?' => {
                    p += 1;
                    t += 1;
                    continue;
                }
                c if c == text[t] => {
                    p += 1;
                    t += 1;
                    continue;
                }
                _ => {}
            }
        }
        
        // Mismatch - backtrack to last star if any
        if let Some(sp) = star_p {
            p = sp + 1;
            if let Some(st) = star_t {
                star_t = Some(st + 1);
                t = st + 1;
                continue;
            }
        }
        
        return false;
    }
    
    // Match remaining stars
    while p < pattern.len() && pattern[p] == b'*' {
        p += 1;
    }
    
    p == pattern.len()
}

/// Get file extension from path
pub fn get_extension(path: &str) -> Option<&str> {
    let filename = filename(path);
    
    if let Some(pos) = filename.rfind('.') {
        if pos > 0 && pos < filename.len() - 1 {
            return Some(&filename[pos + 1..]);
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("foo/bar"), "/foo/bar");
        assert_eq!(normalize_path("/foo/bar"), "/foo/bar");
        assert_eq!(normalize_path("foo\\bar"), "/foo/bar");
        assert_eq!(normalize_path("foo//bar"), "/foo/bar");
        assert_eq!(normalize_path("foo/./bar"), "/foo/bar");
        assert_eq!(normalize_path("foo/baz/../bar"), "/foo/bar");
        assert_eq!(normalize_path("/"), "/");
        assert_eq!(normalize_path(""), "/");
    }

    #[test]
    fn test_split_path() {
        assert_eq!(split_path("/foo/bar/file.txt"), ("foo/bar", "file.txt"));
        assert_eq!(split_path("file.txt"), ("", "file.txt"));
        assert_eq!(split_path("/folder/"), ("folder", ""));
    }

    #[test]
    fn test_parent_path() {
        assert_eq!(parent_path("/foo/bar/file.txt"), Some("/foo/bar".to_string()));
        assert_eq!(parent_path("/foo"), Some("/".to_string()));
        assert_eq!(parent_path("/"), None);
    }

    #[test]
    fn test_filename() {
        assert_eq!(filename("/foo/bar/file.txt"), "file.txt");
        assert_eq!(filename("file.txt"), "file.txt");
        assert_eq!(filename("/folder/"), "folder");
    }

    #[test]
    fn test_join_paths() {
        assert_eq!(join_paths("/foo", "bar"), "/foo/bar");
        assert_eq!(join_paths("/foo/", "bar"), "/foo/bar");
        assert_eq!(join_paths("/foo", "/bar"), "/bar");
    }

    #[test]
    fn test_glob_match() {
        assert!(glob_match("*.txt", "file.txt"));
        assert!(glob_match("file.???", "file.txt"));
        assert!(glob_match("**/file.txt", "foo/bar/file.txt"));
        assert!(!glob_match("*.txt", "file.doc"));
        assert!(glob_match("data/*.cgf", "data/model.cgf"));
    }

    #[test]
    fn test_get_extension() {
        assert_eq!(get_extension("/path/file.txt"), Some("txt"));
        assert_eq!(get_extension("file.CGF"), Some("CGF"));
        assert_eq!(get_extension("no_extension"), None);
        assert_eq!(get_extension(".hidden"), None);
    }
}
