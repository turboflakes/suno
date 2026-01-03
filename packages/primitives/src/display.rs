use std::time::{SystemTime, UNIX_EPOCH};

/// Format milliseconds to human-readable string (e.g., "6.5s")
pub fn format_millis(millis: u64) -> String {
    let seconds = millis / 1000;

    match seconds {
        s if s < 10 => {
            let seconds_f = millis as f64 / 1000.0;
            format!("{:.1}s", seconds_f)
        }
        s if s < 60 => format!("{}s", s),
        s if s < 3600 => format!("{} mins", s / 60),
        s => {
            let hrs = s / 3600;
            let mins = (s % 3600) / 60;
            if mins > 0 {
                format!("{} hrs {} mins", hrs, mins)
            } else {
                format!("{} hrs", hrs)
            }
        }
    }
}

/// Format planks to human-readable string (e.g., "1.5K")
pub fn format_planks(plancks: u128, decimals: u32, display_decimals: usize) -> String {
    let value = plancks as f64 / 10_f64.powi(decimals as i32);

    match value {
        v if v >= 1_000_000_000.0 => format!("{:.prec$}B", v / 1e9, prec = display_decimals),
        v if v >= 1_000_000.0 => format!("{:.prec$}M", v / 1e6, prec = display_decimals),
        v if v >= 1_000.0 => format!("{:.prec$}K", v / 1e3, prec = display_decimals),
        v => format!("{:.prec$}", v, prec = display_decimals),
    }
}

/// Get elapsed time in milliseconds since the given timestamp
pub fn get_elapsed_millis(last_updated: u128) -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    (now - last_updated) as u64
}

/// Create a three phase progress bar based on elapsed time
pub fn create_progress_bar_by_millis(elapsed_ms: u64, bar_width: usize) -> String {
    const PHASE_1_TIMEOUT: u64 = 6_000; // 6 seconds
    const PHASE_2_TIMEOUT: u64 = 60_000; // 60 seconds

    let (ratio, empty, filled) = if elapsed_ms > PHASE_1_TIMEOUT {
        (
            (elapsed_ms.min(PHASE_2_TIMEOUT)) as f64 / PHASE_2_TIMEOUT as f64,
            "░",
            "█",
        )
    } else {
        (
            (elapsed_ms.min(PHASE_1_TIMEOUT)) as f64 / PHASE_1_TIMEOUT as f64,
            "·",
            "░",
        )
    };

    let filled_chars = (ratio * bar_width as f64) as usize;

    format!(
        "{}{}",
        filled.repeat(filled_chars),
        empty.repeat(bar_width - filled_chars),
    )
}

/// Create a progress bar based on a given progress ratio
pub fn create_progress_bar_by_blocks(progress: f64, bar_width: usize) -> String {
    let filled_chars = (progress * bar_width as f64).round() as usize;
    let empty_chars = bar_width.saturating_sub(filled_chars);

    let filled = "█".repeat(filled_chars);
    let empty = "░".repeat(empty_chars);

    format!("{}{}", filled, empty)
}
