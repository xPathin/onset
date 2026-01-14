use once_cell::sync::Lazy;
use regex::Regex;

static DELAY_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^sh -c 'sleep (\d+) && exec (.+)'$").unwrap());

pub fn wrap_with_delay(exec: &str, delay_seconds: u32) -> String {
    if delay_seconds == 0 {
        return exec.to_string();
    }

    let escaped = exec.replace("'", "'\\''");
    format!("sh -c 'sleep {} && exec {}'", delay_seconds, escaped)
}

pub fn unwrap_delay(exec: &str) -> (String, Option<u32>) {
    if let Some(caps) = DELAY_PATTERN.captures(exec) {
        let delay: u32 = caps[1].parse().unwrap_or(0);
        let command = caps[2].replace("'\\''", "'");
        (command, Some(delay))
    } else {
        (exec.to_string(), None)
    }
}

pub fn get_delay(exec: &str) -> Option<u32> {
    DELAY_PATTERN
        .captures(exec)
        .and_then(|caps| caps[1].parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_with_delay() {
        assert_eq!(
            wrap_with_delay("/usr/bin/app", 5),
            "sh -c 'sleep 5 && exec /usr/bin/app'"
        );
    }

    #[test]
    fn test_wrap_with_zero_delay() {
        assert_eq!(wrap_with_delay("/usr/bin/app", 0), "/usr/bin/app");
    }

    #[test]
    fn test_wrap_with_quotes() {
        assert_eq!(
            wrap_with_delay("/usr/bin/app --name='test'", 3),
            "sh -c 'sleep 3 && exec /usr/bin/app --name='\\''test'\\'''"
        );
    }

    #[test]
    fn test_unwrap_delay() {
        let (cmd, delay) = unwrap_delay("sh -c 'sleep 10 && exec /usr/bin/app'");
        assert_eq!(cmd, "/usr/bin/app");
        assert_eq!(delay, Some(10));
    }

    #[test]
    fn test_unwrap_no_delay() {
        let (cmd, delay) = unwrap_delay("/usr/bin/app");
        assert_eq!(cmd, "/usr/bin/app");
        assert_eq!(delay, None);
    }

    #[test]
    fn test_unwrap_with_quotes() {
        let wrapped = wrap_with_delay("/usr/bin/app --name='test'", 5);
        let (cmd, delay) = unwrap_delay(&wrapped);
        assert_eq!(cmd, "/usr/bin/app --name='test'");
        assert_eq!(delay, Some(5));
    }

    #[test]
    fn test_get_delay() {
        assert_eq!(get_delay("sh -c 'sleep 15 && exec /usr/bin/app'"), Some(15));
        assert_eq!(get_delay("/usr/bin/app"), None);
    }
}
