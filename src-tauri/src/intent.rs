use regex::Regex;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

/// An interpreted user intent.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Intent {
    pub kind: String,
    pub action: String,
    pub payload: serde_json::Value,
}

struct IntentPattern {
    pattern: Regex,
    kind: &'static str,
    action: &'static str,
    extract: fn(&regex::Captures) -> serde_json::Value,
}

static PATTERNS: Lazy<Vec<IntentPattern>> = Lazy::new(|| {
    vec![
        // "email <name>" or "send email to <name>"
        IntentPattern {
            pattern: Regex::new(r"(?i)^(?:email|send\s+(?:an?\s+)?email\s+to)\s+(.+)$").unwrap(),
            kind: "email",
            action: "open_mail",
            extract: |caps| {
                serde_json::json!({ "recipient": caps.get(1).map(|m| m.as_str()).unwrap_or("") })
            },
        },
        // "call <name>"
        IntentPattern {
            pattern: Regex::new(r"(?i)^(?:call|phone|ring)\s+(.+)$").unwrap(),
            kind: "phone",
            action: "open_facetime",
            extract: |caps| {
                serde_json::json!({ "contact": caps.get(1).map(|m| m.as_str()).unwrap_or("") })
            },
        },
        // "timer <n>" / "pomodoro <n>" / "start timer <n> min"
        IntentPattern {
            pattern: Regex::new(r"(?i)^(?:start\s+)?(?:timer|pomodoro|alarm)\s*(\d+)(?:\s*min(?:utes?)?)?$").unwrap(),
            kind: "timer",
            action: "start_timer",
            extract: |caps| {
                let minutes: u64 = caps
                    .get(1)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(25);
                serde_json::json!({ "minutes": minutes })
            },
        },
        // "open <app>"
        IntentPattern {
            pattern: Regex::new(r"(?i)^open\s+(.+)$").unwrap(),
            kind: "open",
            action: "open_app",
            extract: |caps| {
                serde_json::json!({ "name": caps.get(1).map(|m| m.as_str()).unwrap_or("") })
            },
        },
        // "search <query>" / "google <query>"
        IntentPattern {
            pattern: Regex::new(r"(?i)^(?:search(?:\s+for)?|google)\s+(.+)$").unwrap(),
            kind: "web_search",
            action: "open_browser",
            extract: |caps| {
                let q = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                let url = format!("https://www.google.com/search?q={}", urlencoding(q));
                serde_json::json!({ "query": q, "url": url })
            },
        },
        // "volume <0-100>"
        IntentPattern {
            pattern: Regex::new(r"(?i)^(?:set\s+)?volume\s+(\d+)$").unwrap(),
            kind: "system",
            action: "set_volume",
            extract: |caps| {
                // Parse as u64 first to avoid u8 overflow on values > 255, then clamp to 100.
                let v: u64 = caps
                    .get(1)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(50)
                    .min(100);
                serde_json::json!({ "volume": v })
            },
        },
        // "sleep" / "lock screen"
        IntentPattern {
            pattern: Regex::new(r"(?i)^(?:sleep|lock\s+(?:screen|computer|mac))$").unwrap(),
            kind: "system",
            action: "sleep",
            extract: |_| serde_json::json!({}),
        },
        // "empty trash"
        IntentPattern {
            pattern: Regex::new(r"(?i)^empty\s+trash$").unwrap(),
            kind: "system",
            action: "empty_trash",
            extract: |_| serde_json::json!({}),
        },
        // "brightness <0-100>"
        IntentPattern {
            pattern: Regex::new(r"(?i)^(?:set\s+)?brightness\s+(\d+)$").unwrap(),
            kind: "system",
            action: "set_brightness",
            extract: |caps| {
                // Parse as u64 first to avoid u8 overflow on values > 255, then clamp to 100.
                let v: u64 = caps
                    .get(1)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(80)
                    .min(100);
                serde_json::json!({ "brightness": v })
            },
        },
    ]
});

/// Percent-encodes a string for use in a URL query parameter.
///
/// Iterates over the *UTF-8 bytes* of the string so that multi-byte Unicode
/// characters are encoded as `%XX%XX…` sequences rather than a single
/// `%XXXXXX` code-point escape, producing valid RFC-3986 percent-encoding.
fn urlencoding(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 3);
    for byte in s.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(*byte as char);
            }
            b' ' => result.push('+'),
            b => result.push_str(&format!("%{b:02X}")),
        }
    }
    result
}

/// Attempt to parse the user query as an intent.
/// Returns `None` if no pattern matches.
pub fn parse_intent(query: &str) -> Option<Intent> {
    let query = query.trim();
    for p in PATTERNS.iter() {
        if let Some(caps) = p.pattern.captures(query) {
            let payload = (p.extract)(&caps);
            return Some(Intent {
                kind: p.kind.to_string(),
                action: p.action.to_string(),
                payload,
            });
        }
    }
    None
}
