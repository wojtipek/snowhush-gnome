use snow_core::geometry::smoothstep;
use snow_core::halloween;

use crate::desktop::Power;

pub fn daylight() -> [f32; 3] {
    daylight_at(halloween::previewed_hour().unwrap_or_else(crate::ffi::local_hour))
}

pub fn is_halloween() -> bool {
    let (month, day, hour) = crate::ffi::local_date();
    halloween::is_halloween(month, day, hour as u32)
}

pub fn daylight_at(hour: f32) -> [f32; 3] {
    let day = smoothstep(6.0, 8.5, hour) * (1.0 - smoothstep(17.5, 20.5, hour));
    let golden = ((-((hour - 7.6) / 1.1).powi(2)).exp() + (-((hour - 18.6) / 1.2).powi(2)).exp()).min(1.0);
    let night = [0.86, 0.89, 0.97];
    let warm = [1.0, 0.965, 0.915];
    std::array::from_fn(|i| {
        let base = night[i] + (1.0 - night[i]) * day;
        base + (warm[i] - base) * golden * 0.55
    })
}

pub fn power_from_system() -> Power {
    let Ok(entries) = std::fs::read_dir("/sys/class/power_supply") else { return Power::Full };
    let mut mains_seen = false;
    for entry in entries.flatten() {
        let path = entry.path();
        let kind = std::fs::read_to_string(path.join("type")).unwrap_or_default();
        if kind.trim() == "Mains" {
            mains_seen = true;
            if std::fs::read_to_string(path.join("online")).unwrap_or_default().trim() == "1" {
                return Power::Full;
            }
        }
    }
    if mains_seen { Power::Battery } else { Power::Full }
}
