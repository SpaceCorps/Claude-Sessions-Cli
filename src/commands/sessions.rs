use serde_json::Value;

use crate::error::Result;
use crate::output;
use crate::store::Store;

pub fn run(spec: Option<&str>) -> Result<()> {
    let store = Store::open()?;
    let profiles = match spec {
        Some(spec) => store.resolve(spec)?,
        None => store.profiles()?,
    };
    let mut rows = Vec::new();
    for profile in &profiles {
        rows.extend(profile.sessions()?.iter().map(|s| s.to_json(profile)));
    }
    output::write(&Value::Array(rows));
    Ok(())
}
