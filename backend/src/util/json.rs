pub fn json_get_string(body: &str, key: &str) -> Option<String> {
    let needle = format!("\"{}\"", key);
    let i = body.find(&needle)?;
    let after_key = &body[i + needle.len()..];
    let colon = after_key.find(':')?;
    let mut s = after_key[colon + 1..].trim_start();

    if !s.starts_with('"') {
        return None;
    }
    s = &s[1..]; // skip opening quote

    let mut out = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        match c {
            '"' => return Some(out), // closing quote
            '\\' => {
                // handle a few escapes
                if let Some(e) = chars.next() {
                    match e {
                        '"' => out.push('"'),
                        '\\' => out.push('\\'),
                        'n' => out.push('\n'),
                        'r' => out.push('\r'),
                        't' => out.push('\t'),
                        _ => {
                            // unknown escape: keep it as-is
                            out.push(e);
                        }
                    }
                } else {
                    return None;
                }
            }
            _ => out.push(c),
        }
    }
    None
}

pub fn json_get_bool(body: &str, key: &str) -> Option<bool> {
    let needle = format!("\"{}\"", key);
    let i = body.find(&needle)?;
    let after_key = &body[i + needle.len()..];
    let colon = after_key.find(':')?;
    let s = after_key[colon + 1..].trim_start();
    if s.starts_with("true") {
        Some(true)
    } else if s.starts_with("false") {
        Some(false)
    } else {
        None
    }
}

pub fn json_get_string_array(body: &str, key: &str) -> Option<Vec<String>> {
    let needle = format!("\"{}\"", key);
    let i = body.find(&needle)?;
    let after_key = &body[i + needle.len()..];
    let colon = after_key.find(':')?;
    let mut s = after_key[colon + 1..].trim_start();
    if !s.starts_with('[') {
        return None;
    }
    s = &s[1..]; // skip '['

    let mut out = Vec::new();

    loop {
        s = s.trim_start();
        if s.starts_with(']') {
            return Some(out);
        }
        if !s.starts_with('"') {
            return None;
        }
        s = &s[1..]; // skip opening quote

        // parse a JSON string (same logic as json_get_string, but inline)
        let mut val = String::new();
        let mut idx = 0usize;
        let bytes = s.as_bytes();
        while idx < bytes.len() {
            let c = bytes[idx] as char;
            match c {
                '"' => {
                    // end string
                    out.push(val);
                    s = &s[idx + 1..];
                    break;
                }
                '\\' => {
                    if idx + 1 >= bytes.len() {
                        return None;
                    }
                    let e = bytes[idx + 1] as char;
                    match e {
                        '"' => val.push('"'),
                        '\\' => val.push('\\'),
                        'n' => val.push('\n'),
                        'r' => val.push('\r'),
                        't' => val.push('\t'),
                        _ => val.push(e),
                    }
                    idx += 2;
                }
                _ => {
                    val.push(c);
                    idx += 1;
                }
            }
        }

        s = s.trim_start();
        if s.starts_with(',') {
            s = &s[1..];
            continue;
        }
        if s.starts_with(']') {
            return Some(out);
        }
        // otherwise invalid
        return None;
    }
}

pub fn json_escape(s: &str) -> String {
    // Minimal JSON string escaping
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(c),
        }
    }
    out
}

fn json_get_unsigned_str<'a>(body: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!("\"{}\"", key);
    let i = body.find(&needle)?;
    let after_key = &body[i + needle.len()..];
    let colon = after_key.find(':')?;
    let s = after_key[colon + 1..].trim_start();
    let bytes = s.as_bytes();
    let mut end = 0usize;
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }
    if end == 0 {
        return None;
    }
    Some(&s[..end])
}

pub fn json_get_u64(body: &str, key: &str) -> Option<u64> {
    json_get_unsigned_str(body, key)?.parse::<u64>().ok()
}

pub fn json_get_u128(body: &str, key: &str) -> Option<u128> {
    json_get_unsigned_str(body, key)?.parse::<u128>().ok()
}
