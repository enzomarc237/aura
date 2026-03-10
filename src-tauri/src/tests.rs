/// Tests for the search and intent modules.
#[cfg(test)]
mod tests {

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
    }

    mod search_tests {
        use crate::search::fuzzy_search;

        #[test]
        fn test_empty_query_returns_results() {
            // Empty query should return without error (may be empty if DB is not seeded)
            let results = fuzzy_search("", 10);
            assert!(results.is_ok());
        }

        #[test]
        fn test_search_query_returns_results() {
            let results = fuzzy_search("test", 5);
            assert!(results.is_ok());
            let r = results.unwrap();
            assert!(r.len() <= 5);
        }
    }

    mod database_tests {
        use crate::database::{get_setting, set_setting};

        #[test]
        fn test_settings_roundtrip() {
            set_setting("test_key", "test_value").expect("set setting");
            let val = get_setting("test_key").expect("get setting");
            assert_eq!(val, Some("test_value".to_string()));
        }

        #[test]
        fn test_default_settings_exist() {
            let hotkey = get_setting("hotkey").expect("get hotkey");
            assert!(hotkey.is_some());
            let theme = get_setting("theme").expect("get theme");
            assert!(theme.is_some());
        }
    }
}
