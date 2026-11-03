use std::path::PathBuf;

pub fn home_dir() -> PathBuf {
    #[cfg(windows)]
    let variable = "USERPROFILE";
    #[cfg(not(windows))]
    let variable = "HOME";
    PathBuf::from(std::env::var_os(variable).unwrap_or_default())
}

pub fn app_data_dir() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("APPDATA").map(PathBuf::from)
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME").map(|home| PathBuf::from(home).join("Library/Application Support"))
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
    }
}

#[cfg_attr(windows, allow(dead_code))]
pub fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', r"'\''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_quoting_keeps_special_characters_literal() {
        assert_eq!(shell_quote("/Users/me/Crab & Co"), "'/Users/me/Crab & Co'");
        assert_eq!(shell_quote("it's $HOME `x`"), r"'it'\''s $HOME `x`'");
        assert_eq!(shell_quote(""), "''");
    }
}
