// crates/hwp-types/src/control.rs

//! 컨트롤 타입
//!
//! 표, 이미지 등 인라인 컨트롤을 표현합니다.

use serde::{Deserialize, Serialize};

/// 인라인 컨트롤 (표, 그림 등)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Control {
    /// 표
    Table(Table),
    /// 그림
    Picture(Picture),
    /// 알 수 없는 컨트롤
    Unknown {
        /// 컨트롤 타입 코드
        ctrl_id: u32,
    },
}

impl Default for Control {
    fn default() -> Self {
        Control::Unknown { ctrl_id: 0 }
    }
}

/// 표 컨트롤
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Table {
    /// 속성 플래그
    pub properties: u32,
    /// 행 수
    pub rows: u16,
    /// 열 수
    pub cols: u16,
    /// 셀 간격 (HWPUNIT: 1/7200 inch)
    pub cell_spacing: u16,
    /// 왼쪽 여백 (HWPUNIT)
    pub left_margin: i32,
    /// 오른쪽 여백 (HWPUNIT)
    pub right_margin: i32,
    /// 위쪽 여백 (HWPUNIT)
    pub top_margin: i32,
    /// 아래쪽 여백 (HWPUNIT)
    pub bottom_margin: i32,
    /// 셀 목록
    pub cells: Vec<TableCell>,
    /// 테두리/배경 ID
    pub border_fill_id: u16,
}

impl Table {
    /// 새 표 생성
    pub fn new(rows: u16, cols: u16) -> Self {
        Self {
            rows,
            cols,
            ..Default::default()
        }
    }

    /// 빈 표 생성
    pub fn empty() -> Self {
        Self::default()
    }

    /// 셀 추가
    pub fn add_cell(&mut self, cell: TableCell) {
        self.cells.push(cell);
    }

    /// 셀 수 반환
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    /// 테두리/배경 ID 설정
    pub fn with_border_fill_id(mut self, id: u16) -> Self {
        self.border_fill_id = id;
        self
    }

    /// 특정 위치의 셀 찾기
    pub fn get_cell(&self, row: u16, col: u16) -> Option<&TableCell> {
        self.cells.iter().find(|c| c.row == row && c.col == col)
    }
}

/// 표 셀
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TableCell {
    /// 리스트 헤더 ID
    pub list_header_id: u32,
    /// 열 주소 (0-based)
    pub col: u16,
    /// 행 주소 (0-based)
    pub row: u16,
    /// 열 병합 수 (1 = 병합 없음)
    pub col_span: u16,
    /// 행 병합 수 (1 = 병합 없음)
    pub row_span: u16,
    /// 셀 너비 (HWPUNIT: 1/7200 inch)
    pub width: u32,
    /// 셀 높이 (HWPUNIT)
    pub height: u32,
    /// 왼쪽 여백 (HWPUNIT)
    pub left_margin: u16,
    /// 오른쪽 여백 (HWPUNIT)
    pub right_margin: u16,
    /// 위쪽 여백 (HWPUNIT)
    pub top_margin: u16,
    /// 아래쪽 여백 (HWPUNIT)
    pub bottom_margin: u16,
    /// 테두리/배경 ID
    pub border_fill_id: u16,
    /// 텍스트 너비 (HWPUNIT)
    pub text_width: u32,
    /// 필드 이름
    pub field_name: String,
    /// 셀 내용 텍스트 (간소화된 버전)
    /// 실제로는 `Vec<Paragraph>`이지만 순환 참조 방지를 위해 String으로 저장
    pub text: String,
}

impl TableCell {
    /// 새 셀 생성
    pub fn new(row: u16, col: u16) -> Self {
        Self {
            row,
            col,
            col_span: 1,
            row_span: 1,
            ..Default::default()
        }
    }

    /// 빈 셀 생성
    pub fn empty() -> Self {
        Self {
            col_span: 1,
            row_span: 1,
            ..Default::default()
        }
    }

    /// 병합 설정
    pub fn with_span(mut self, col_span: u16, row_span: u16) -> Self {
        self.col_span = col_span;
        self.row_span = row_span;
        self
    }

    /// 크기 설정
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// 텍스트 설정
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// 셀이 병합되었는지 확인
    pub fn is_merged(&self) -> bool {
        self.col_span > 1 || self.row_span > 1
    }

    /// 열 병합 수 반환
    pub fn get_col_span(&self) -> u16 {
        self.col_span
    }

    /// 행 병합 수 반환
    pub fn get_row_span(&self) -> u16 {
        self.row_span
    }
}

/// 그림 컨트롤
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Picture {
    /// BinData ID 참조
    pub bin_data_id: u16,
    /// 너비 (HWPUNIT: 1/7200 inch)
    pub width: u32,
    /// 높이 (HWPUNIT)
    pub height: u32,
}

impl Picture {
    /// 새 그림 생성
    pub fn new(bin_data_id: u16) -> Self {
        Self {
            bin_data_id,
            ..Default::default()
        }
    }

    /// 크기 설정
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ═══════════════════════════════════════════════════════════════
    // Table Tests (US3)
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_should_create_table_with_dimensions_when_set() {
        // Arrange & Act
        let table = Table::new(3, 4);

        // Assert
        assert_eq!(table.rows, 3);
        assert_eq!(table.cols, 4);
        assert!(table.cells.is_empty());
    }

    #[test]
    fn test_should_add_cell_to_table_when_pushed() {
        // Arrange
        let mut table = Table::new(2, 2);
        let cell = TableCell::new(0, 0);

        // Act
        table.add_cell(cell);

        // Assert
        assert_eq!(table.cell_count(), 1);
    }

    #[test]
    fn test_should_find_cell_by_position() {
        // Arrange
        let mut table = Table::new(2, 2);
        table.add_cell(TableCell::new(0, 0).with_text("A1"));
        table.add_cell(TableCell::new(0, 1).with_text("B1"));
        table.add_cell(TableCell::new(1, 0).with_text("A2"));

        // Act & Assert
        assert_eq!(table.get_cell(0, 0).unwrap().text, "A1");
        assert_eq!(table.get_cell(0, 1).unwrap().text, "B1");
        assert_eq!(table.get_cell(1, 0).unwrap().text, "A2");
        assert!(table.get_cell(1, 1).is_none());
    }

    // ═══════════════════════════════════════════════════════════════
    // TableCell Tests (US3 - 병합 정보)
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_should_store_cell_span_when_set() {
        // Arrange & Act
        let cell = TableCell::new(0, 0).with_span(2, 3);

        // Assert
        assert_eq!(cell.col_span, 2);
        assert_eq!(cell.row_span, 3);
        assert!(cell.is_merged());
    }

    #[test]
    fn test_should_return_false_when_not_merged() {
        // Arrange & Act
        let cell = TableCell::new(0, 0);

        // Assert
        assert_eq!(cell.col_span, 1);
        assert_eq!(cell.row_span, 1);
        assert!(!cell.is_merged());
    }

    #[test]
    fn test_should_return_span_values_via_getters() {
        // Arrange
        let cell = TableCell::new(1, 2).with_span(4, 5);

        // Act & Assert
        assert_eq!(cell.get_col_span(), 4);
        assert_eq!(cell.get_row_span(), 5);
    }

    #[test]
    fn test_should_store_cell_size_when_set() {
        // Arrange & Act
        let cell = TableCell::empty().with_size(7200, 3600);

        // Assert
        assert_eq!(cell.width, 7200); // 1 inch
        assert_eq!(cell.height, 3600); // 0.5 inch
    }

    #[test]
    fn test_should_chain_cell_builder_methods() {
        // Arrange & Act
        let cell = TableCell::new(0, 0)
            .with_span(2, 1)
            .with_size(14400, 7200)
            .with_text("Merged Cell");

        // Assert
        assert_eq!(cell.row, 0);
        assert_eq!(cell.col, 0);
        assert_eq!(cell.col_span, 2);
        assert_eq!(cell.row_span, 1);
        assert_eq!(cell.width, 14400);
        assert_eq!(cell.height, 7200);
        assert_eq!(cell.text, "Merged Cell");
    }

    // ═══════════════════════════════════════════════════════════════
    // Picture Tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_should_create_picture_with_bindata_id_when_set() {
        // Arrange & Act
        let picture = Picture::new(5);

        // Assert
        assert_eq!(picture.bin_data_id, 5);
        assert_eq!(picture.width, 0);
        assert_eq!(picture.height, 0);
    }

    #[test]
    fn test_should_set_picture_size_when_with_size() {
        // Arrange & Act
        let picture = Picture::new(1).with_size(7200, 10800);

        // Assert
        assert_eq!(picture.bin_data_id, 1);
        assert_eq!(picture.width, 7200);
        assert_eq!(picture.height, 10800);
    }

    // ═══════════════════════════════════════════════════════════════
    // Control Enum Tests
    // ═══════════════════════════════════════════════════════════════

    #[test]
    fn test_should_create_control_variants() {
        // Table variant
        let table_ctrl = Control::Table(Table::new(2, 2));
        assert!(matches!(table_ctrl, Control::Table(_)));

        // Picture variant
        let pic_ctrl = Control::Picture(Picture::new(1));
        assert!(matches!(pic_ctrl, Control::Picture(_)));

        // Unknown variant
        let unknown_ctrl = Control::Unknown {
            ctrl_id: 0x12345678,
        };
        assert!(matches!(
            unknown_ctrl,
            Control::Unknown {
                ctrl_id: 0x12345678
            }
        ));
    }
}
