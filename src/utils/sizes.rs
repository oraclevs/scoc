use crate::ScocError;

fn multiplier(prefix: char, binary: bool) -> Option<f64> {
    let index = "kmgtpezy".find(prefix.to_ascii_lowercase())?;
    let exp = (index + 1) as i32;
    Some(if binary {
        1024_f64.powi(exp)
    } else {
        1000_f64.powi(exp)
    })
}

pub fn convert_size_to_int(size: &str, binary: bool, posix_mode: bool) -> Option<i64> {
    let normalized = size.replace(',', "");
    let text = normalized.trim();
    if text.is_empty() {
        return None;
    }

    let number_end = text
        .char_indices()
        .take_while(|(_, ch)| ch.is_ascii_digit() || *ch == '.' || *ch == '-')
        .map(|(idx, ch)| idx + ch.len_utf8())
        .last()?;
    let number = text[..number_end].parse::<f64>().ok()?;
    let mut unit = text[number_end..].trim().to_ascii_lowercase();
    if unit.ends_with('s') {
        unit.pop();
    }
    if unit.is_empty() || unit.starts_with('b') {
        return Some(number as i64);
    }

    if unit.len() == 1 && posix_mode {
        unit.push_str("ib");
    }
    if unit.len() == 2 && unit.ends_with('i') {
        unit.push('b');
    }

    let names = [
        ("k", "kb", "kib", "kilobyte", "kibibyte"),
        ("m", "mb", "mib", "megabyte", "mebibyte"),
        ("g", "gb", "gib", "gigabyte", "gibibyte"),
        ("t", "tb", "tib", "terabyte", "tebibyte"),
        ("p", "pb", "pib", "petabyte", "pebibyte"),
        ("e", "eb", "eib", "exabyte", "exbibyte"),
        ("z", "zb", "zib", "zettabyte", "zebibyte"),
        ("y", "yb", "yib", "yottabyte", "yobibyte"),
    ];

    for (prefix, decimal_symbol, binary_symbol, decimal_name, binary_name) in names {
        if unit == binary_symbol || unit == binary_name {
            return Some((number * multiplier(prefix.chars().next()?, true)?) as i64);
        }
        if unit == decimal_symbol || unit == decimal_name || unit.starts_with(prefix) {
            return Some((number * multiplier(prefix.chars().next()?, binary)?) as i64);
        }
    }
    None
}

pub fn parse_human_size(size: &str, binary: bool) -> Result<i64, ScocError> {
    convert_size_to_int(size, binary, false).ok_or_else(|| ScocError::InvalidInput {
        parser: "size".into(),
        message: format!("could not parse size `{size}`"),
    })
}
