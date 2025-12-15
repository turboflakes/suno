/// Format milliseconds to human-readable string (e.g., "6.5s")
pub fn format_millis(millis: u64) -> String {
    let seconds = millis as f64 / 1000.0;

    match seconds {
        s if s < 10.0 => format!("{:.1}s", s),
        s if s < 60.0 => format!("{:.0}s", s),
        s if s < 600.0 => format!("{:.1}m", s / 60.0),
        s if s < 3600.0 => format!("{:.0}m", s / 60.0),
        s if s < 36000.0 => format!("{:.1}h", s / 3600.0),
        _ => format!("{:.0}h", seconds / 3600.0),
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
