use chrono::Duration;
use chrono::TimeDelta;

pub fn parse_duration_str(s: &str) -> Option<Duration> {
    humantime::parse_duration(s)
        .into_iter()
        .filter_map(|x| TimeDelta::from_std(x).ok())
        .next()
}

/// Truncates any time more fine-grained than seconds
pub fn format_duration(duration: &Duration) -> String {
    let duration = duration.to_std();
    if let Ok(duration) = duration {
        let duration = std::time::Duration::from_secs(duration.as_secs());
        humantime::format_duration(duration).to_string()
    } else {
        "Invalid duration".to_string()
    }
}
