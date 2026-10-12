//! Boundary observation timestamps must match the bytes that were approved.
pub(super) fn unix_ms(value: &str) -> Result<u64, &'static str> {
    if !crate::validation::valid_timestamp(value) || !value.ends_with('Z') {
        return Err("boundary-timestamp-invalid");
    }
    let number = |start, end| {
        value[start..end]
            .parse::<u64>()
            .map_err(|_| "boundary-timestamp-invalid")
    };
    let year = number(0, 4)?;
    let month = number(5, 7)?;
    let day = number(8, 10)?;
    if year < 1970 {
        return Err("boundary-timestamp-invalid");
    }
    let leap = |y: u64| y.is_multiple_of(400) || (y.is_multiple_of(4) && !y.is_multiple_of(100));
    let before = [0_u64, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let index = usize::try_from(month - 1).map_err(|_| "boundary-timestamp-invalid")?;
    let days = (year - 1970) * 365
        + u64::try_from((1970..year).filter(|y| leap(*y)).count())
            .map_err(|_| "boundary-timestamp-invalid")?
        + before[index]
        + u64::from(month > 2 && leap(year))
        + day
        - 1;
    let fraction = match value.len() {
        20 => 0,
        24 if value.as_bytes()[19] == b'.' => number(20, 23)?,
        _ => return Err("boundary-timestamp-invalid"),
    };
    Ok(
        (days * 86_400 + number(11, 13)? * 3600 + number(14, 16)? * 60 + number(17, 19)?) * 1000
            + fraction,
    )
}
#[cfg(test)]
mod tests {
    #[test]
    fn utc_milliseconds_and_calendar_bounds_match() {
        assert_eq!(super::unix_ms("1970-01-01T00:00:00Z"), Ok(0));
        assert_eq!(
            super::unix_ms("2000-01-01T00:00:00.123Z"),
            Ok(946_684_800_123)
        );
        assert!(super::unix_ms("2026-02-29T00:00:00Z").is_err());
        assert!(super::unix_ms("2000-01-01T00:00:00+01:00").is_err());
    }
}
