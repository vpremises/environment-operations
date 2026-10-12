//! Shared lexical validators keep identifiers and report time fields predictable.

use std::collections::BTreeSet;

pub(crate) fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
}

pub(crate) fn valid_date(value: &str) -> bool {
    if !value.is_ascii() {
        return false;
    }
    let bytes = value.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes
            .iter()
            .enumerate()
            .any(|(index, byte)| !matches!(index, 4 | 7) && !byte.is_ascii_digit())
    {
        return false;
    }
    let year = decimal(&bytes[..4]);
    let month = decimal(&bytes[5..7]);
    let day = decimal(&bytes[8..10]);
    days_in_month(year, month).is_some_and(|limit| day > 0 && day <= limit)
}

pub(crate) fn valid_timestamp(value: &str) -> bool {
    if !value.is_ascii()
        || value.len() < 20
        || value.len() > 64
        || value.as_bytes().get(10) != Some(&b'T')
        || !valid_date(&value[..10])
    {
        return false;
    }
    let suffix = &value[11..];
    let (time, zone) = if let Some(time) = suffix.strip_suffix('Z') {
        (time, "Z")
    } else if let Some(index) = suffix.rfind(['+', '-']) {
        (&suffix[..index], &suffix[index..])
    } else {
        return false;
    };
    valid_time(time) && (zone == "Z" || valid_zone(zone))
}

pub(crate) fn unique(values: &[String]) -> bool {
    values
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>()
        .len()
        == values.len()
}

fn valid_time(value: &str) -> bool {
    let (whole, fraction) = value
        .split_once('.')
        .map_or((value, None), |(whole, fraction)| (whole, Some(fraction)));
    let bytes = whole.as_bytes();
    bytes.len() == 8
        && bytes[2] == b':'
        && bytes[5] == b':'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 2 | 5) || byte.is_ascii_digit())
        && decimal(&bytes[..2]) <= 23
        && decimal(&bytes[3..5]) <= 59
        && decimal(&bytes[6..8]) <= 59
        && fraction
            .is_none_or(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
}

fn valid_zone(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 6
        && matches!(bytes[0], b'+' | b'-')
        && bytes[3] == b':'
        && bytes[1..3].iter().all(u8::is_ascii_digit)
        && bytes[4..6].iter().all(u8::is_ascii_digit)
        && decimal(&bytes[1..3]) <= 23
        && decimal(&bytes[4..6]) <= 59
}

fn decimal(bytes: &[u8]) -> u32 {
    bytes
        .iter()
        .fold(0, |value, byte| value * 10 + u32::from(byte - b'0'))
}

const fn days_in_month(year: u32, month: u32) -> Option<u32> {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => Some(31),
        4 | 6 | 9 | 11 => Some(30),
        2 if year.is_multiple_of(400) || (year.is_multiple_of(4) && !year.is_multiple_of(100)) => {
            Some(29)
        }
        2 => Some(28),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
