// crates/hwp-core/tests/docinfo_integration.rs

//! DocInfo 파싱 통합 테스트
//!
//! 여러 레코드 타입을 포함한 DocInfo 스트림 파싱을 종합적으로 테스트합니다.

use hwp_core::parser::{parse_docinfo, parse_table};
use hwp_types::{Alignment, BinDataType};

/// 레코드 헤더 생성 헬퍼
fn create_record(tag_id: u16, level: u16, data: &[u8]) -> Vec<u8> {
    let size = data.len() as u32;
    assert!(size < 4095, "Size must be < 4095 for this test helper");

    let dword: u32 = (tag_id as u32) | ((level as u32) << 10) | (size << 20);

    let mut result = Vec::new();
    result.extend_from_slice(&dword.to_le_bytes());
    result.extend_from_slice(data);
    result
}

/// UTF-16LE 문자열 데이터 생성
fn utf16le_string(s: &str) -> Vec<u8> {
    let utf16: Vec<u16> = s.encode_utf16().collect();
    let mut data = Vec::new();
    data.extend_from_slice(&(utf16.len() as u16).to_le_bytes());
    for ch in utf16 {
        data.extend_from_slice(&ch.to_le_bytes());
    }
    data
}

/// FaceName 레코드 데이터 생성
fn create_face_name_data(name: &str) -> Vec<u8> {
    let mut data = vec![0x00]; // properties
    data.extend(utf16le_string(name));
    data
}

/// CharShape 레코드 데이터 생성
fn create_char_shape_data(base_size: i32, bold: bool, italic: bool) -> Vec<u8> {
    let mut data = Vec::new();

    // font_ids: 7 * u16
    for i in 0u16..7 {
        data.extend_from_slice(&i.to_le_bytes());
    }
    // font_scales, char_spacing, relative_sizes, char_offsets: 7 * u8 each
    data.extend_from_slice(&[100u8; 7]); // scales
    data.extend_from_slice(&[0u8; 7]); // spacing
    data.extend_from_slice(&[100u8; 7]); // sizes
    data.extend_from_slice(&[0u8; 7]); // offsets

    // base_size
    data.extend_from_slice(&base_size.to_le_bytes());

    // attr (bold=bit0, italic=bit1)
    let mut attr: u32 = 0;
    if bold {
        attr |= 0x01;
    }
    if italic {
        attr |= 0x02;
    }
    data.extend_from_slice(&attr.to_le_bytes());

    // shadow gaps
    data.push(0);
    data.push(0);

    // colors
    data.extend_from_slice(&0x000000u32.to_le_bytes()); // text
    data.extend_from_slice(&0x000000u32.to_le_bytes()); // underline
    data.extend_from_slice(&0xFFFFFFu32.to_le_bytes()); // shade
    data.extend_from_slice(&0x808080u32.to_le_bytes()); // shadow

    data
}

/// ParaShape 레코드 데이터 생성
fn create_para_shape_data(alignment: u8, line_spacing: i32) -> Vec<u8> {
    let mut data = Vec::new();

    // attr (alignment in bits 2-4)
    let attr: u32 = (alignment as u32) << 2;
    data.extend_from_slice(&attr.to_le_bytes());

    // margins
    data.extend_from_slice(&567i32.to_le_bytes()); // left
    data.extend_from_slice(&567i32.to_le_bytes()); // right
    data.extend_from_slice(&0i32.to_le_bytes()); // indent
    data.extend_from_slice(&283i32.to_le_bytes()); // top_space
    data.extend_from_slice(&283i32.to_le_bytes()); // bottom_space
    data.extend_from_slice(&line_spacing.to_le_bytes()); // line_space

    // IDs
    data.extend_from_slice(&0u16.to_le_bytes()); // tab_def_id
    data.extend_from_slice(&0u16.to_le_bytes()); // numbering_id
    data.extend_from_slice(&0u16.to_le_bytes()); // border_fill_id

    // border spaces
    data.extend_from_slice(&0i16.to_le_bytes());
    data.extend_from_slice(&0i16.to_le_bytes());
    data.extend_from_slice(&0i16.to_le_bytes());
    data.extend_from_slice(&0i16.to_le_bytes());

    // attr2, attr3
    data.extend_from_slice(&0u32.to_le_bytes());
    data.extend_from_slice(&0u32.to_le_bytes());

    // line_space_type
    data.extend_from_slice(&0u32.to_le_bytes());

    data
}

/// BinData 레코드 데이터 생성
fn create_bin_data_data(storage_type: u16, extension: &str) -> Vec<u8> {
    let mut data = Vec::new();

    // properties
    data.extend_from_slice(&storage_type.to_le_bytes());

    // bin_id
    data.extend_from_slice(&1u16.to_le_bytes());

    // extension
    data.extend(utf16le_string(extension));

    data
}

// ═══════════════════════════════════════════════════════════════
// 통합 테스트
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_should_parse_complete_docinfo_stream() {
    // Arrange: 여러 타입의 레코드를 포함한 DocInfo 스트림
    let mut stream = Vec::new();

    // BinData 레코드 (tag 0x12)
    stream.extend(create_record(0x12, 0, &create_bin_data_data(0x01, "png")));

    // FaceName 레코드들 (tag 0x13)
    stream.extend(create_record(0x13, 0, &create_face_name_data("맑은 고딕")));
    stream.extend(create_record(0x13, 0, &create_face_name_data("Arial")));

    // CharShape 레코드들 (tag 0x15)
    stream.extend(create_record(
        0x15,
        0,
        &create_char_shape_data(1000, false, false),
    )); // 10pt, normal
    stream.extend(create_record(
        0x15,
        0,
        &create_char_shape_data(1200, true, false),
    )); // 12pt, bold
    stream.extend(create_record(
        0x15,
        0,
        &create_char_shape_data(1000, false, true),
    )); // 10pt, italic

    // ParaShape 레코드 (tag 0x19)
    stream.extend(create_record(
        0x19,
        0,
        &create_para_shape_data(1, 160), // Left, 160%
    ));

    // Act
    let docinfo = parse_docinfo(&stream).expect("Failed to parse DocInfo");

    // Assert
    // BinData 확인
    assert_eq!(docinfo.bin_data.len(), 1);
    assert_eq!(docinfo.bin_data[0].storage_type, BinDataType::Embedding);
    assert_eq!(docinfo.bin_data[0].extension, "png");

    // FaceName 확인
    assert_eq!(docinfo.face_names.len(), 2);
    assert_eq!(docinfo.face_names[0].name, "맑은 고딕");
    assert_eq!(docinfo.face_names[1].name, "Arial");

    // CharShape 확인
    assert_eq!(docinfo.char_shapes.len(), 3);
    assert_eq!(docinfo.char_shapes[0].base_size, 1000);
    assert!(!docinfo.char_shapes[0].attr.is_bold());
    assert!(docinfo.char_shapes[1].attr.is_bold());
    assert!(docinfo.char_shapes[2].attr.is_italic());

    // ParaShape 확인
    assert_eq!(docinfo.para_shapes.len(), 1);
    assert_eq!(docinfo.para_shapes[0].alignment(), Alignment::Left);
}

#[test]
fn test_should_access_items_by_index() {
    // Arrange
    let mut stream = Vec::new();
    for i in 0..5 {
        stream.extend(create_record(
            0x13,
            0,
            &create_face_name_data(&format!("Font{}", i)),
        ));
    }

    // Act
    let docinfo = parse_docinfo(&stream).unwrap();

    // Assert
    assert_eq!(docinfo.get_face_name(0).unwrap().name, "Font0");
    assert_eq!(docinfo.get_face_name(2).unwrap().name, "Font2");
    assert_eq!(docinfo.get_face_name(4).unwrap().name, "Font4");
    assert!(docinfo.get_face_name(5).is_none());
}

#[test]
fn test_should_handle_empty_stream_gracefully() {
    // Arrange
    let stream: &[u8] = &[];

    // Act
    let docinfo = parse_docinfo(stream).unwrap();

    // Assert
    assert!(docinfo.bin_data.is_empty());
    assert!(docinfo.face_names.is_empty());
    assert!(docinfo.char_shapes.is_empty());
    assert!(docinfo.para_shapes.is_empty());
    assert!(docinfo.border_fills.is_empty());
}

#[test]
fn test_should_skip_unsupported_tags() {
    // Arrange: 알 수 없는 태그 포함
    let mut stream = Vec::new();
    stream.extend(create_record(0x13, 0, &create_face_name_data("TestFont")));
    stream.extend(create_record(0xFF, 0, &[1, 2, 3, 4])); // 알 수 없는 태그
    stream.extend(create_record(
        0x13,
        0,
        &create_face_name_data("AnotherFont"),
    ));

    // Act
    let docinfo = parse_docinfo(&stream).unwrap();

    // Assert: 알 수 없는 태그는 무시됨
    assert_eq!(docinfo.face_names.len(), 2);
}

// ═══════════════════════════════════════════════════════════════
// Table 통합 테스트
// ═══════════════════════════════════════════════════════════════

/// 테이블 데이터 생성
fn create_table_data(rows: u16, cols: u16) -> Vec<u8> {
    let mut data = Vec::new();

    // 테이블 헤더
    data.extend_from_slice(&0x0001u32.to_le_bytes()); // properties
    data.extend_from_slice(&rows.to_le_bytes());
    data.extend_from_slice(&cols.to_le_bytes());
    data.extend_from_slice(&142u16.to_le_bytes()); // cell_spacing
    data.extend_from_slice(&567i32.to_le_bytes()); // margins
    data.extend_from_slice(&567i32.to_le_bytes());
    data.extend_from_slice(&567i32.to_le_bytes());
    data.extend_from_slice(&567i32.to_le_bytes());

    // 셀 데이터
    for _ in 0..(rows * cols) {
        data.extend_from_slice(&0u32.to_le_bytes()); // list_header_id
        data.extend_from_slice(&1u16.to_le_bytes()); // col_span
        data.extend_from_slice(&1u16.to_le_bytes()); // row_span
        data.extend_from_slice(&7200u32.to_le_bytes()); // width
        data.extend_from_slice(&3600u32.to_le_bytes()); // height
        data.extend_from_slice(&100u16.to_le_bytes()); // margins
        data.extend_from_slice(&100u16.to_le_bytes());
        data.extend_from_slice(&100u16.to_le_bytes());
        data.extend_from_slice(&100u16.to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes()); // border_fill_id
        data.extend_from_slice(&7000u32.to_le_bytes()); // text_width
        data.extend_from_slice(&0u16.to_le_bytes()); // empty field_name
    }

    data
}

#[test]
fn test_should_parse_table_with_complete_cells() {
    // Arrange
    let data = create_table_data(3, 3);

    // Act
    let (_, table) = parse_table(&data).expect("Failed to parse table");

    // Assert
    assert_eq!(table.rows, 3);
    assert_eq!(table.cols, 3);
    assert_eq!(table.cells.len(), 9);

    // 모든 셀의 위치 확인
    for row in 0..3 {
        for col in 0..3 {
            let cell = table.get_cell(row, col);
            assert!(cell.is_some(), "Cell at ({}, {}) should exist", row, col);
        }
    }
}

#[test]
fn test_should_calculate_cell_positions_correctly() {
    // Arrange
    let data = create_table_data(2, 4);

    // Act
    let (_, table) = parse_table(&data).unwrap();

    // Assert: 8 cells (2 rows * 4 cols)
    assert_eq!(table.cells.len(), 8);

    // 첫 번째 행
    assert_eq!(table.cells[0].row, 0);
    assert_eq!(table.cells[0].col, 0);
    assert_eq!(table.cells[1].row, 0);
    assert_eq!(table.cells[1].col, 1);
    assert_eq!(table.cells[2].row, 0);
    assert_eq!(table.cells[2].col, 2);
    assert_eq!(table.cells[3].row, 0);
    assert_eq!(table.cells[3].col, 3);

    // 두 번째 행
    assert_eq!(table.cells[4].row, 1);
    assert_eq!(table.cells[4].col, 0);
}
