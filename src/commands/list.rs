use serde_json::Value;

use crate::error::Result;
use crate::store::{Store, iso8601};
use crate::{obj, output};

pub fn run() -> Result<()> {
    let store = Store::open()?;
    let current = store.current().ok().map(|p| p.id());
    let mut rows = Vec::new();
    for profile in store.profiles()? {
        let sessions = profile.sessions()?;
        let id = profile.id();
        rows.push(obj! {
            "profile" => id,
            "current" => current.as_deref() == Some(id.as_str()),
            "sessions" => sessions.len(),
            "archived" => sessions.iter().filter(|s| s.archived).count(),
            "lastActivity" => sessions.iter().filter_map(|s| s.last_activity).max().map(iso8601),
        });
    }
    output::write(&obj! {
        "signedInAccount" => store.signed_in_account(),
        "current" => current,
        "profiles" => Value::Array(rows),
    });
    Ok(())
}
