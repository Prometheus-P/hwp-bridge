// crates/hwp-core/src/parser/bodytext/table.rs

//! Table (표) 파서
//!
//! HWP BodyText 스트림의 Table 레코드를 파싱합니다.
//! 레코드 태그: 0x4D (TABLE)

use hwp_types::{Table, TableCell};
use nom::{
    IResult,
    number::complete::{le_i32, le_u16, le_u32},
};

use crate::parser::primitives::parse_utf16le_string;

/// Table 레코드 최소 크기 (바이트)
/// properties(4) + rows(2) + cols(2) + cell_spacing(2) + margins(16) = 26
pub const TABLE_MIN_SIZE: usize = 26;

/// TableCell 최소 크기 (바이트)
/// list_header_id(4) + col_span(2) + row_span(2) + width(4) + height(4) +
/// margins(8) + border_fill_id(2) + text_width(4) = 30
pub const TABLE_CELL_MIN_SIZE: usize = 30;

/// Table 레코드 파싱
///
/// # Format (가변 길이)
/// - properties: u32 = 4 bytes
/// - rows: u16 = 2 bytes
/// - cols: u16 = 2 bytes
/// - cell_spacing: u16 = 2 bytes
/// - left_margin: i32 = 4 bytes
/// - right_margin: i32 = 4 bytes
/// - top_margin: i32 = 4 bytes
/// - bottom_margin: i32 = 4 bytes
/// - [cells...] (가변)
pub fn parse_table(input: &[u8]) -> IResult<&[u8], Table> {
    // 기본 속성
    let (input, properties) = le_u32(input)?;
    let (input, rows) = le_u16(input)?;
    let (input, cols) = le_u16(input)?;
    let (input, cell_spacing) = le_u16(input)?;
    let (input, left_margin) = le_i32(input)?;
    let (input, right_margin) = le_i32(input)?;
    let (input, top_margin) = le_i32(input)?;
    let (input, bottom_margin) = le_i32(input)?;

    // 셀 파싱 (rows * cols 개수만큼)
    let total_cells = (rows as usize) * (cols as usize);
    let mut cells = Vec::with_capacity(total_cells);
    let mut remaining = input;

    for i in 0..total_cells {
        // 데이터가 부족하면 중단
        if remaining.len() < TABLE_CELL_MIN_SIZE {
            break;
        }

        let row = (i / cols as usize) as u16;
        let col = (i % cols as usize) as u16;

        match parse_table_cell_internal(remaining, row, col) {
            Ok((rest, cell)) => {
                cells.push(cell);
                remaining = rest;
            }
            Err(_) => break,
        }
    }

    Ok((
        remaining,
        Table {
            properties,
            rows,
            cols,
            cell_spacing,
            left_margin,
            right_margin,
            top_margin,
            bottom_margin,
            cells,
            border_fill_id: 0,
        },
    ))
}

/// TableCell 레코드 파싱 (row, col 주소 포함)
fn parse_table_cell_internal(input: &[u8], row: u16, col: u16) -> IResult<&[u8], TableCell> {
    let (input, list_header_id) = le_u32(input)?;
    let (input, col_span) = le_u16(input)?;
    let (input, row_span) = le_u16(input)?;
    let (input, width) = le_u32(input)?;
    let (input, height) = le_u32(input)?;
    let (input, left_margin) = le_u16(input)?;
    let (input, right_margin) = le_u16(input)?;
    let (input, top_margin) = le_u16(input)?;
    let (input, bottom_margin) = le_u16(input)?;
    let (input, border_fill_id) = le_u16(input)?;
    let (input, text_width) = le_u32(input)?;

    // 필드 이름 (UTF-16LE string, 길이 prefix)
    let (input, field_name) = if input.len() >= 2 {
        parse_utf16le_string(input)?
    } else {
        (input, String::new())
    };

    Ok((
        input,
        TableCell {
            list_header_id,
            row,
            col,
            col_span,
            row_span,
            width,
            height,
            left_margin,
            right_margin,
            top_margin,
            bottom_margin,
            border_fill_id,
            text_width,
            field_name,
            text: String::new(),
        },
    ))
}

/// TableCell 레코드 파싱 (독립 함수)
pub fn parse_table_cell(input: &[u8]) -> IResult<&[u8], TableCell> {
    parse_table_cell_internal(input, 0, 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ═══════════════════════════════════════════════════════════════
    // Test Helpers
    // ═══════════════════════════════════════════════════════════════

    /// 기본 Table 데이터 생성 (셀 없음)
    fn create_table_header_data(rows: u16, cols: u16) -> Vec<u8> {
        let mut data = Vec::new();

        // properties: 0x0001 (테두리 활성화)
        data.extend_from_slice(&0x0001u32.to_le_bytes());
        // rows
        data.extend_from_slice(&rows.to_le_bytes());
        // cols
        data.extend_from_slice(&cols.to_le_bytes());
        // cell_spacing: 142 (0.5mm)
        data.extend_from_slice(&142u16.to_le_bytes());
        // margins: 567 (2mm) each
        data.extend_from_slice(&567i32.to_le_bytes()); // left
        data.extend_from_slice(&567i32.to_le_bytes()); // right
        data.extend_from_slice(&567i32.to_le_bytes()); // top
        data.extend_from_slice(&567i32.to_le_bytes()); // bottom

        data
    }

    /// TableCell 데이터 생성
    fn create_cell_data(width: u32, height: u32) -> Vec<u8> {
        let mut data = Vec::new();

        // list_header_id: 0
        data.extend_from_slice(&0u32.to_le_bytes());
        // col_span: 1
        data.extend_from_slice(&1u16.to_le_bytes());
        // row_span: 1
        data.extend_from_slice(&1u16.to_le_bytes());
        // width
        data.extend_from_slice(&width.to_le_bytes());
        // height
        data.extend_from_slice(&height.to_le_bytes());
        // margins: 100 each
        data.extend_from_slice(&100u16.to_le_bytes()); // left
        data.extend_from_slice(&100u16.to_le_bytes()); // right
        data.extend_from_slice(&100u16.to_le_bytes()); // top
        data.extend_from_slice(&100u16.to_le_bytes()); // bottom
        // border_fill_id: 0
        data.extend_from_slice(&0u16.to_le_bytes());
        // text_width
        data.extend_from_slice(&(width - 200).to_le_bytes());
        // field_name: empty string (length = 0)
        data.extend_from_slice(&0u16.to_le_bytes());

        data
    }

    /// 병합된 셀 데이터 생성
    fn create_merged_cell_data(col_span: u16, row_span: u16) -> Vec<u8> {
        let mut data = Vec::new();

        // list_header_id: 1
        data.extend_from_slice(&1u32.to_le_bytes());
        // col_span
        data.extend_from_slice(&col_span.to_le_bytes());
        // row_span
        data.extend_from_slice(&row_span.to_le_bytes());
        // width: 14400 (2 inch)
        data.extend_from_slice(&14400u32.to_le_bytes());
        // height: 7200 (1 inch)
        data.extend_from_slice(&7200u32.to_le_bytes());
        // margins
        data.extend_from_slice(&100u16.to_le_bytes());
        data.extend_from_slice(&100u16.to_le_bytes());
        data.extend_from_slice(&100u16.to_le_bytes());
        data.extend_from_slice(&100u16.to_le_bytes());
        // border_fill_id: 1
        data.extend_from_slice(&1u16.to_le_bytes());
        // text_width
        data.extend_from_slice(&14200u32.to_le_bytes());
        // field_name: "Merged"
        let name = "Merged";
        let name_utf16: Vec<u16> = name.encode_utf16().collect();
        data.extend_from_slice(&(name_utf16.len() as u16).to_le_bytes());
        for ch in name_utf16 {
            data.extend_from_slice(&ch.to_le_bytes());
        }

        data
    }

    // ═══════════════════════════════════════════════════════════════
    // Table Parser Tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_should_parse_table_header_when_valid_data() {
        // Arrange
        let data = create_table_header_data(3, 4);

        // Act
        let result = parse_table(&data);

        // Assert
        assert!(result.is_ok());
        let (remaining, table) = result.unwrap();
        assert!(remaining.is_empty());
        assert_eq!(table.rows, 3);
        assert_eq!(table.cols, 4);
        assert_eq!(table.properties, 0x0001);
        assert_eq!(table.cell_spacing, 142);
        assert_eq!(table.left_margin, 567);
        assert!(table.cells.is_empty()); // 셀 데이터 없음
    }

    #[test]
    fn test_should_parse_table_with_cells_when_data_provided() {
        // Arrange: 2x2 표
        let mut data = create_table_header_data(2, 2);

        // 4개의 셀 추가
        for _ in 0..4 {
            data.extend(create_cell_data(7200, 3600));
        }

        // Act
        let result = parse_table(&data);

        // Assert
        assert!(result.is_ok());
        let (_, table) = result.unwrap();
        assert_eq!(table.rows, 2);
        assert_eq!(table.cols, 2);
        assert_eq!(table.cells.len(), 4);

        // 셀 위치 확인
        assert_eq!(table.cells[0].row, 0);
        assert_eq!(table.cells[0].col, 0);
        assert_eq!(table.cells[1].row, 0);
        assert_eq!(table.cells[1].col, 1);
        assert_eq!(table.cells[2].row, 1);
        assert_eq!(table.cells[2].col, 0);
        assert_eq!(table.cells[3].row, 1);
        assert_eq!(table.cells[3].col, 1);
    }

    #[test]
    fn test_should_parse_partial_cells_when_data_incomplete() {
        // Arrange: 2x2 표지만 셀 데이터는 2개만
        let mut data = create_table_header_data(2, 2);
        data.extend(create_cell_data(7200, 3600));
        data.extend(create_cell_data(7200, 3600));

        // Act
        let result = parse_table(&data);

        // Assert
        assert!(result.is_ok());
        let (_, table) = result.unwrap();
        assert_eq!(table.rows, 2);
        assert_eq!(table.cols, 2);
        assert_eq!(table.cells.len(), 2); // 데이터가 있는 만큼만 파싱
    }

    // ═══════════════════════════════════════════════════════════════
    // TableCell Parser Tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_should_parse_cell_when_valid_data() {
        // Arrange
        let data = create_cell_data(7200, 3600);

        // Act
        let result = parse_table_cell(&data);

        // Assert
        assert!(result.is_ok());
        let (remaining, cell) = result.unwrap();
        assert!(remaining.is_empty());
        assert_eq!(cell.width, 7200);
        assert_eq!(cell.height, 3600);
        assert_eq!(cell.col_span, 1);
        assert_eq!(cell.row_span, 1);
        assert_eq!(cell.left_margin, 100);
        assert_eq!(cell.text_width, 7000);
    }

    #[test]
    fn test_should_parse_merged_cell_when_spans_greater_than_one() {
        // Arrange
        let data = create_merged_cell_data(2, 3);

        // Act
        let result = parse_table_cell(&data);

        // Assert
        assert!(result.is_ok());
        let (_, cell) = result.unwrap();
        assert_eq!(cell.col_span, 2);
        assert_eq!(cell.row_span, 3);
        assert!(cell.is_merged());
        assert_eq!(cell.field_name, "Merged");
        assert_eq!(cell.border_fill_id, 1);
    }

    #[test]
    fn test_should_return_cell_address_when_table_parsed() {
        // Arrange: 3x3 표
        let mut data = create_table_header_data(3, 3);
        for _ in 0..9 {
            data.extend(create_cell_data(4800, 2400));
        }

        // Act
        let (_, table) = parse_table(&data).unwrap();

        // Assert: 중앙 셀 (1,1) 확인
        let center_cell = table.get_cell(1, 1);
        assert!(center_cell.is_some());
        assert_eq!(center_cell.unwrap().row, 1);
        assert_eq!(center_cell.unwrap().col, 1);
    }

    // ═══════════════════════════════════════════════════════════════
    // Edge Cases
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_should_handle_empty_table_when_zero_rows_cols() {
        // Arrange
        let data = create_table_header_data(0, 0);

        // Act
        let result = parse_table(&data);

        // Assert
        assert!(result.is_ok());
        let (_, table) = result.unwrap();
        assert_eq!(table.rows, 0);
        assert_eq!(table.cols, 0);
        assert!(table.cells.is_empty());
    }

    #[test]
    fn test_should_parse_single_cell_table() {
        // Arrange: 1x1 표
        let mut data = create_table_header_data(1, 1);
        data.extend(create_cell_data(7200, 7200));

        // Act
        let result = parse_table(&data);

        // Assert
        assert!(result.is_ok());
        let (_, table) = result.unwrap();
        assert_eq!(table.rows, 1);
        assert_eq!(table.cols, 1);
        assert_eq!(table.cells.len(), 1);
        assert_eq!(table.cells[0].width, 7200);
    }
}
