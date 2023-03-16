use chrono::{DateTime, Utc};

pub fn create_tag() -> String {
    let now: DateTime<Utc> = Utc::now();
    now.format("%Y-%m-%dT%H%M").to_string()
}
