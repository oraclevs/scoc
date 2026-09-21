use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};

/// Parse the ISO timestamp forms supported by the first SCOC `ls` port.
/// Returns `(naive_local_epoch, utc_epoch)`; the UTC value is present only
/// when the input explicitly identifies UTC.
pub fn parse_ls_timestamp(text: &str) -> (Option<i64>, Option<i64>) {
    let text = text.trim();
    let formats = [
        "%Y-%m-%d %H:%M",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M:%S%.f",
    ];
    for format in formats {
        if let Ok(naive) = NaiveDateTime::parse_from_str(text, format) {
            let local = Local
                .from_local_datetime(&naive)
                .single()
                .map(|dt| dt.timestamp());
            return (local, None);
        }
    }
    if let Ok(dt) = DateTime::parse_from_str(text, "%Y-%m-%d %H:%M:%S%.f %z") {
        let utc = (dt.offset().local_minus_utc() == 0).then(|| dt.with_timezone(&Utc).timestamp());
        let naive = Local
            .from_local_datetime(&dt.naive_local())
            .single()
            .map(|value| value.timestamp());
        return (naive, utc);
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(text) {
        let utc = (dt.offset().local_minus_utc() == 0).then(|| dt.with_timezone(&Utc).timestamp());
        let naive = Local
            .from_local_datetime(&dt.naive_local())
            .single()
            .map(|value| value.timestamp());
        return (naive, utc);
    }
    (None, None)
}
