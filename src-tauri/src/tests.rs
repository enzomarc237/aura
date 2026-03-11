/// Tests for the search, intent, and database modules.
///
/// All tests that touch the database call `setup_test_db()` which redirects
/// the global `DB` singleton to a throwaway temporary directory so that unit
/// tests never write to the real Aura application database.
#[cfg(test)]
mod tests {
    /// Redirect the global DB to a temp directory for the duration of the test
    /// binary.  This must be called before the first `DB` access in each test
    /// module that uses the database.
    fn setup_test_db() {
        use std::sync::OnceLock;
        static INIT: OnceLock<()> = OnceLock::new();
        INIT.get_or_init(|| {
            let dir = std::env::temp_dir().join("aura_test_db");
            std::fs::create_dir_all(&dir).expect("create test db dir");
            // SAFETY: tests run in a single process; setting this env var before
            // any DB access is safe and ensures isolation from production data.
            unsafe { std::env::set_var("AURA_DATA_DIR", dir.to_str().unwrap()); }
        });
    }

    mod intent_tests {
        use crate::intent::parse_intent;

        #[test]
        fn test_email_intent() {
            let intent = parse_intent("email Marc Enzo").unwrap();
            assert_eq!(intent.kind, "email");
            assert_eq!(intent.action, "open_mail");
            assert_eq!(intent.payload["recipient"], "Marc Enzo");
        }

        #[test]
        fn test_email_intent_send_form() {
            let intent = parse_intent("send email to Alice").unwrap();
            assert_eq!(intent.kind, "email");
            assert_eq!(intent.payload["recipient"], "Alice");
        }

        #[test]
        fn test_timer_intent() {
            let intent = parse_intent("pomodoro 25").unwrap();
            assert_eq!(intent.kind, "timer");
            assert_eq!(intent.action, "start_timer");
            assert_eq!(intent.payload["minutes"], 25);
        }

        #[test]
        fn test_timer_intent_with_min() {
            let intent = parse_intent("start timer 10 min").unwrap();
            assert_eq!(intent.kind, "timer");
            assert_eq!(intent.payload["minutes"], 10);
        }

        #[test]
        fn test_web_search_intent() {
            let intent = parse_intent("search for Rust programming").unwrap();
            assert_eq!(intent.kind, "web_search");
            assert!(intent.payload["url"]
                .as_str()
                .unwrap()
                .contains("google.com"));
        }

        #[test]
        fn test_google_search_intent() {
            let intent = parse_intent("google Tauri framework").unwrap();
            assert_eq!(intent.kind, "web_search");
            assert_eq!(intent.payload["query"], "Tauri framework");
        }

        #[test]
        fn test_sleep_intent() {
            let intent = parse_intent("sleep").unwrap();
            assert_eq!(intent.kind, "system");
            assert_eq!(intent.action, "sleep");
        }

        #[test]
        fn test_volume_intent() {
            let intent = parse_intent("set volume 75").unwrap();
            assert_eq!(intent.kind, "system");
            assert_eq!(intent.action, "set_volume");
            assert_eq!(intent.payload["volume"], 75);
        }

        #[test]
        fn test_no_intent() {
            assert!(parse_intent("Safari").is_none());
            assert!(parse_intent("random query").is_none());
            assert!(parse_intent("").is_none());
        }

        #[test]
        fn test_case_insensitive() {
            let intent = parse_intent("EMAIL marc").unwrap();
            assert_eq!(intent.kind, "email");

            let intent2 = parse_intent("SLEEP").unwrap();
            assert_eq!(intent2.kind, "system");
        }

        #[test]
        fn test_volume_large_value_clamped() {
            // Values > 255 would overflow u8; they must be clamped to 100.
            let intent = parse_intent("set volume 300").unwrap();
            assert_eq!(intent.kind, "system");
            assert_eq!(intent.payload["volume"], 100u64);
        }

        #[test]
        fn test_brightness_large_value_clamped() {
            let intent = parse_intent("set brightness 999").unwrap();
            assert_eq!(intent.kind, "system");
            assert_eq!(intent.payload["brightness"], 100u64);
        }

        #[test]
        fn test_urlencoding_non_ascii() {
            let intent = parse_intent("google café").unwrap();
            let url = intent.payload["url"].as_str().unwrap();
            // UTF-8 bytes of 'é' (0xC3 0xA9) must be percent-encoded, not the
            // Unicode code-point (U+00E9 = 0xE9).
            assert!(url.contains("%C3%A9"), "URL: {url}");
            assert!(!url.contains("%E9"), "URL must not use code-point encoding: {url}");
        }

        #[test]
        fn test_brightness_intent() {
            let intent = parse_intent("brightness 80").unwrap();
            assert_eq!(intent.kind, "system");
            assert_eq!(intent.action, "set_brightness");
            assert_eq!(intent.payload["brightness"], 80u64);
        }
    }

    mod search_tests {
        use crate::search::fuzzy_search;
        use super::setup_test_db;

        #[test]
        fn test_empty_query_returns_results() {
            setup_test_db();
            let results = fuzzy_search("", 10);
            assert!(results.is_ok());
        }

        #[test]
        fn test_search_query_returns_results() {
            setup_test_db();
            let results = fuzzy_search("test", 5);
            assert!(results.is_ok());
            let r = results.unwrap();
            assert!(r.len() <= 5);
        }
    }

    mod database_tests {
        use crate::database::{get_setting, set_setting};
        use super::setup_test_db;

        #[test]
        fn test_settings_roundtrip() {
            setup_test_db();
            set_setting("test_key", "test_value").expect("set setting");
            let val = get_setting("test_key").expect("get setting");
            assert_eq!(val, Some("test_value".to_string()));
        }

        #[test]
        fn test_default_settings_exist() {
            setup_test_db();
            let hotkey = get_setting("hotkey").expect("get hotkey");
            assert!(hotkey.is_some());
            let theme = get_setting("theme").expect("get theme");
            assert!(theme.is_some());
        }
    }
}
