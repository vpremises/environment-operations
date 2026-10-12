//! Calendar and timezone validation regression cases.

use super::{valid_date, valid_timestamp};

#[test]
fn calendar_and_timezone_values_are_semantically_bounded() {
    assert!(valid_date("2024-02-29"));
    assert!(!valid_date("2026-02-29"));
    assert!(valid_timestamp("2026-07-24T17:00:00+09:00"));
    assert!(!valid_timestamp("2026-07-24T24:00:00Z"));
}
