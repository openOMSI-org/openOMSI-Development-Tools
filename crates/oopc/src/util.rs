//! Small shared helpers: where keys live, a time stamp, and the API permission map.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

/// The API manifest this build of `oopc` knows, embedded so `check` can map functions to the
/// permissions they need without a game checkout.
pub const API_JSON: &str = include_str!("../../openomsi-plugin-sdk/api.json");

/// The directory where signing keys are kept (created on demand): the user's config dir under
/// `openOMSI-devtools/keys`.
pub fn keys_dir() -> Result<PathBuf, String> {
    let base = dirs::config_dir().ok_or("cannot find a config directory for this user")?;
    Ok(base.join("openOMSI-devtools").join("keys"))
}

/// An RFC 3339 UTC timestamp for "now", computed without a date-time dependency.
pub fn now_rfc3339() -> String {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) as i64;
    // Days since the Unix epoch and the civil date (Howard Hinnant's algorithm).
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (h, mi, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
}

/// A map from a function's dotted API name (as Lua calls it, `ui.set`, `set_var`) to the
/// permission it needs, from the embedded manifest.
pub fn permission_map() -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    if let Ok(Value::Object(root)) = serde_json::from_str::<Value>(API_JSON)
        && let Some(Value::Array(funcs)) = root.get("functions")
    {
        for f in funcs {
            if let (Some(name), Some(perm)) =
                (f.get("name").and_then(Value::as_str), f.get("permission").and_then(Value::as_str))
            {
                map.insert(name.to_string(), perm.to_string());
            }
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_shape() {
        let t = now_rfc3339();
        assert_eq!(t.len(), 20);
        assert!(t.ends_with('Z'));
        assert_eq!(&t[4..5], "-");
    }

    #[test]
    fn permissions_loaded() {
        let m = permission_map();
        assert_eq!(m.get("set_var").map(String::as_str), Some("vehicle_write"));
        assert_eq!(m.get("ui.set").map(String::as_str), Some("ui"));
        assert_eq!(m.get("send").map(String::as_str), Some("network_local"));
    }
}
