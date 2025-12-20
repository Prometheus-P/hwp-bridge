// crates/hwp-core/src/parser/summary.rs

//! HwpSummaryInformation stream parser
//!
//! Parses document metadata from the OLE Summary Information stream.
//! Contains title, author, subject, keywords, and timestamps.

use hwp_types::HwpError;

/// Parsed document summary information
#[derive(Debug, Clone, Default)]
pub struct HwpSummaryInfo {
    /// Document title
    pub title: Option<String>,
    /// Document subject
    pub subject: Option<String>,
    /// Document author
    pub author: Option<String>,
    /// Keywords
    pub keywords: Option<String>,
    /// Comments
    pub comments: Option<String>,
    /// Last saved by
    pub last_saved_by: Option<String>,
    /// Revision number
    pub revision_number: Option<String>,
    /// Created timestamp (ISO 8601)
    pub created_at: Option<String>,
    /// Last modified timestamp (ISO 8601)
    pub modified_at: Option<String>,
    /// Last printed timestamp (ISO 8601)
    pub printed_at: Option<String>,
}

/// Property IDs from OLE Summary Information specification
#[allow(dead_code)]
mod property_id {
    pub const CODEPAGE: u32 = 0x01;
    pub const TITLE: u32 = 0x02;
    pub const SUBJECT: u32 = 0x03;
    pub const AUTHOR: u32 = 0x04;
    pub const KEYWORDS: u32 = 0x05;
    pub const COMMENTS: u32 = 0x06;
    pub const TEMPLATE: u32 = 0x07;
    pub const LAST_SAVED_BY: u32 = 0x08;
    pub const REVISION_NUMBER: u32 = 0x09;
    pub const EDIT_TIME: u32 = 0x0A;
    pub const LAST_PRINTED: u32 = 0x0B;
    pub const CREATE_DATE: u32 = 0x0C;
    pub const MODIFY_DATE: u32 = 0x0D;
    pub const PAGE_COUNT: u32 = 0x0E;
    pub const WORD_COUNT: u32 = 0x0F;
    pub const CHAR_COUNT: u32 = 0x10;
    pub const THUMBNAIL: u32 = 0x11;
    pub const APP_NAME: u32 = 0x12;
    pub const DOC_SECURITY: u32 = 0x13;
}

/// Type markers for property values
mod value_type {
    pub const VT_LPWSTR: u32 = 0x1F; // UTF-16LE string
    pub const VT_FILETIME: u32 = 0x40; // Windows FILETIME
}

/// Parse HwpSummaryInformation stream
///
/// The stream follows OLE Property Set format:
/// - Header at offset 0x00
/// - Property count at offset 0x2C
/// - Property entries (ID + offset pairs) follow
pub fn parse_summary_info(data: &[u8]) -> Result<HwpSummaryInfo, HwpError> {
    if data.len() < 0x30 {
        return Ok(HwpSummaryInfo::default());
    }

    let mut result = HwpSummaryInfo::default();

    // Property count is at offset 0x2C (little-endian u32)
    let property_count = read_u32(data, 0x2C)? as usize;

    // Property entries start at 0x30
    // Each entry is 8 bytes: property_id (u32) + offset (u32)
    let entries_start = 0x30;
    let entries_end = entries_start + property_count * 8;

    if entries_end > data.len() {
        return Ok(result); // Not enough data
    }

    for i in 0..property_count {
        let entry_offset = entries_start + i * 8;
        let property_id = read_u32(data, entry_offset)?;
        let value_offset = read_u32(data, entry_offset + 4)? as usize;

        // Value offset is relative to section start (typically 0x30)
        // But in practice, it seems to be an absolute offset in the data
        let abs_offset = if value_offset < 0x30 {
            entries_start + value_offset
        } else {
            value_offset
        };

        if abs_offset + 4 > data.len() {
            continue;
        }

        match property_id {
            property_id::TITLE => {
                result.title = parse_string_property(data, abs_offset);
            }
            property_id::SUBJECT => {
                result.subject = parse_string_property(data, abs_offset);
            }
            property_id::AUTHOR => {
                result.author = parse_string_property(data, abs_offset);
            }
            property_id::KEYWORDS => {
                result.keywords = parse_string_property(data, abs_offset);
            }
            property_id::COMMENTS => {
                result.comments = parse_string_property(data, abs_offset);
            }
            property_id::LAST_SAVED_BY => {
                result.last_saved_by = parse_string_property(data, abs_offset);
            }
            property_id::REVISION_NUMBER => {
                result.revision_number = parse_string_property(data, abs_offset);
            }
            property_id::CREATE_DATE => {
                result.created_at = parse_filetime_property(data, abs_offset);
            }
            property_id::MODIFY_DATE => {
                result.modified_at = parse_filetime_property(data, abs_offset);
            }
            property_id::LAST_PRINTED => {
                result.printed_at = parse_filetime_property(data, abs_offset);
            }
            _ => {}
        }
    }

    Ok(result)
}

/// Parse a string property (VT_LPWSTR: type 0x1F)
fn parse_string_property(data: &[u8], offset: usize) -> Option<String> {
    if offset + 8 > data.len() {
        return None;
    }

    let value_type = read_u32(data, offset).ok()?;
    if value_type != value_type::VT_LPWSTR {
        return None;
    }

    // Size is in UTF-16 code units (including null terminator)
    let size = read_u32(data, offset + 4).ok()? as usize;
    let byte_size = size * 2;

    if offset + 8 + byte_size > data.len() || byte_size == 0 {
        return None;
    }

    let string_bytes = &data[offset + 8..offset + 8 + byte_size];

    // Decode UTF-16LE, strip null terminator
    let u16_chars: Vec<u16> = string_bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .take_while(|&c| c != 0)
        .collect();

    let s = String::from_utf16(&u16_chars).ok()?;
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Parse a FILETIME property (type 0x40)
fn parse_filetime_property(data: &[u8], offset: usize) -> Option<String> {
    if offset + 12 > data.len() {
        return None;
    }

    let value_type = read_u32(data, offset).ok()?;
    if value_type != value_type::VT_FILETIME {
        return None;
    }

    // FILETIME is 64-bit value representing 100-nanosecond intervals since Jan 1, 1601
    let low = read_u32(data, offset + 4).ok()? as u64;
    let high = read_u32(data, offset + 8).ok()? as u64;
    let filetime = low | (high << 32);

    if filetime == 0 {
        return None;
    }

    filetime_to_iso8601(filetime)
}

/// Convert Windows FILETIME to ISO 8601 string
fn filetime_to_iso8601(filetime: u64) -> Option<String> {
    // FILETIME is 100-nanosecond intervals since Jan 1, 1601
    // Unix epoch is Jan 1, 1970
    // Difference: 11644473600 seconds = 116444736000000000 100-ns intervals

    const EPOCH_DIFF: u64 = 116_444_736_000_000_000;

    if filetime < EPOCH_DIFF {
        return None;
    }

    let unix_100ns = filetime - EPOCH_DIFF;
    let unix_secs = unix_100ns / 10_000_000;

    // Use chrono if available, otherwise format manually
    // For now, format as UTC timestamp
    let days_since_epoch = unix_secs / 86400;
    let time_of_day = unix_secs % 86400;

    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Calculate year, month, day from days since 1970-01-01
    let (year, month, day) = days_to_ymd(days_since_epoch);

    Some(format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    ))
}

/// Convert days since Unix epoch to (year, month, day)
fn days_to_ymd(days: u64) -> (i32, u32, u32) {
    // Simplified algorithm for dates after 1970
    let mut remaining = days as i64;
    let mut year = 1970i32;

    // Find year
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        year += 1;
    }

    // Find month
    let leap = is_leap_year(year);
    let days_in_months: [i64; 12] = if leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1u32;
    for days_in_month in days_in_months {
        if remaining < days_in_month {
            break;
        }
        remaining -= days_in_month;
        month += 1;
    }

    (year, month, remaining as u32 + 1)
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn read_u32(data: &[u8], offset: usize) -> Result<u32, HwpError> {
    if offset + 4 > data.len() {
        return Err(HwpError::ParseError(format!(
            "Offset {} out of bounds for u32 read",
            offset
        )));
    }
    Ok(u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_return_default_for_empty_data() {
        let result = parse_summary_info(&[]).unwrap();
        assert!(result.title.is_none());
        assert!(result.author.is_none());
    }

    #[test]
    fn test_should_return_default_for_insufficient_data() {
        let data = vec![0u8; 0x20];
        let result = parse_summary_info(&data).unwrap();
        assert!(result.title.is_none());
    }

    #[test]
    fn test_should_convert_filetime_to_iso8601() {
        // 2020-01-01 00:00:00 UTC
        // Unix timestamp: 1577836800
        // FILETIME = (unix_secs * 10_000_000) + EPOCH_DIFF
        // EPOCH_DIFF = 116_444_736_000_000_000
        // FILETIME = (1577836800 * 10_000_000) + 116_444_736_000_000_000
        //          = 15_778_368_000_000_000 + 116_444_736_000_000_000
        //          = 132_223_104_000_000_000
        let filetime: u64 = 132_223_104_000_000_000;
        let result = filetime_to_iso8601(filetime);
        assert!(result.is_some());
        let date_str = result.unwrap();
        assert!(
            date_str.starts_with("2020-01-01"),
            "Expected 2020-01-01, got {}",
            date_str
        );
    }

    #[test]
    fn test_should_parse_string_property() {
        // Build a VT_LPWSTR property: type (0x1F), size (5), "Test\0" in UTF-16LE
        let mut data = vec![0u8; 100];
        // Type = 0x1F
        data[0] = 0x1F;
        data[1] = 0x00;
        data[2] = 0x00;
        data[3] = 0x00;
        // Size = 5 (including null)
        data[4] = 0x05;
        data[5] = 0x00;
        data[6] = 0x00;
        data[7] = 0x00;
        // "Test" in UTF-16LE: T=0x54, e=0x65, s=0x73, t=0x74, null=0x00
        data[8] = 0x54;
        data[9] = 0x00;
        data[10] = 0x65;
        data[11] = 0x00;
        data[12] = 0x73;
        data[13] = 0x00;
        data[14] = 0x74;
        data[15] = 0x00;
        data[16] = 0x00;
        data[17] = 0x00;

        let result = parse_string_property(&data, 0);
        assert_eq!(result, Some("Test".to_string()));
    }

    #[test]
    fn test_days_to_ymd() {
        // 2020-01-01 is day 18262 since 1970-01-01
        let (y, m, d) = days_to_ymd(18262);
        assert_eq!(y, 2020);
        assert_eq!(m, 1);
        assert_eq!(d, 1);
    }

    #[test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2000));
        assert!(is_leap_year(2020));
        assert!(!is_leap_year(1900));
        assert!(!is_leap_year(2019));
    }
}
