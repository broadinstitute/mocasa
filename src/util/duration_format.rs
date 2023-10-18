use std::time::Duration;

pub(crate) fn format_duration(duration: Duration) -> String {
    if duration.is_zero() {
        "0s".to_string()
    } else {
        let nanos =  duration.as_nanos();
        if nanos < 1000 {
            format!("{}ns", nanos)
        } else {
            let micros = duration.as_micros();
            if micros < 1000 {
                format!("{}.{:0>3}µs", micros, nanos % 1000)
            } else {
                let millis = duration.as_millis();
                if millis < 1000 {
                    format!("{}.{:0>3}ms", millis, nanos % 1000)
                } else {
                    let secs = duration.as_secs();
                    if secs < 60 {
                        format!("{}.{:0>3}s", secs, millis % 1000)
                    } else {
                        let mins = secs / 60;
                        if mins < 60 {
                            format!("{}m{}s", mins, secs % 60)
                        } else {
                            let hours = mins / 60;
                            if hours < 24 {
                                format!("{}h{}m{}s", hours, mins % 60, secs % 60)
                            } else {
                                let days = hours / 24;
                                if days < 7 {
                                    format!("{}d{}h{}m", days, hours % 24, mins % 60)
                                } else {
                                    let weeks = days / 7;
                                    format!("{}w{}d{}h", weeks, days % 7, hours % 24)
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
