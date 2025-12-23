// crates/hwp-types/src/docinfo.rs
//! DocInfo record types.
//!
//! Structures that describe document-level metadata and style definitions.

use serde::{Deserialize, Serialize};

/// DocInfo document properties (HWPTAG_DOCUMENT_PROPERTIES).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocInfoProperties {
    pub section_count: u16,
    pub page_start_number: u16,
    pub footnote_start_number: u16,
    pub endnote_start_number: u16,
    pub figure_start_number: u16,
    pub table_start_number: u16,
    pub equation_start_number: u16,
    pub list_id: u32,
    pub paragraph_id: u32,
    pub char_pos_in_para: u32,
}

/// ID mapping counts (HWPTAG_ID_MAPPINGS).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IdMappings {
    /// Raw counts indexed by the spec table (up to 18 items).
    pub counts: [i32; 18],
}

impl IdMappings {
    pub fn get(&self, idx: usize) -> Option<i32> {
        self.counts.get(idx).copied()
    }
}

/// Tab definition (HWPTAG_TAB_DEF).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TabDef {
    pub properties: u32,
    pub tabs: Vec<TabStop>,
}

/// Single tab stop entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabStop {
    pub position: i32,
    pub kind: u8,
    pub leader: u8,
}

/// Paragraph head info shared by numbering/bullets.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParaHeadInfo {
    pub properties: u32,
    pub width_adjust: i16,
    pub distance: i16,
}

/// Numbering definition (HWPTAG_NUMBERING).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Numbering {
    pub head: ParaHeadInfo,
    pub levels: Vec<NumberingLevel>,
    pub extended_levels: Vec<NumberingLevel>,
}

/// Numbering level.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NumberingLevel {
    pub format: String,
    pub start_number: Option<u16>,
    pub start_number_extended: Option<u32>,
}

/// Bullet definition (HWPTAG_BULLET).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Bullet {
    pub head: ParaHeadInfo,
    pub bullet_char: u16,
    pub image_bullet_id: i32,
    pub image_bullet: Option<ImageBullet>,
    pub check_char: u16,
}

/// Image bullet properties.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImageBullet {
    pub contrast: i8,
    pub brightness: i8,
    pub effect: u8,
    pub image_id: u8,
}

/// Style type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StyleType {
    Paragraph,
    Character,
    Unknown(u8),
}

/// Style definition (HWPTAG_STYLE).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleRecord {
    pub name: String,
    pub english_name: String,
    pub properties: u8,
    pub next_style_id: u8,
    pub language_id: i16,
    pub para_shape_id: u16,
    pub char_shape_id: u16,
}

impl StyleRecord {
    pub fn style_type(&self) -> StyleType {
        match self.properties & 0x07 {
            0 => StyleType::Paragraph,
            1 => StyleType::Character,
            other => StyleType::Unknown(other),
        }
    }
}

/// Parameter set used in DocData and control data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParameterSet {
    pub id: u16,
    pub items: Vec<ParameterItem>,
}

/// Parameter item within a parameter set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterItem {
    pub id: u16,
    pub value: ParameterValue,
}

/// Parameter item value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterValue {
    Null(u32),
    BStr(String),
    I1(i8),
    I2(i16),
    I4(i32),
    I(i32),
    Ui1(u8),
    Ui2(u16),
    Ui4(u32),
    Ui(u32),
    Set(Box<ParameterSet>),
    Array(Vec<ParameterSet>),
    Bindata(u16),
}

/// Compatible document target program.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompatibleDocument {
    pub program: u32,
}

/// Layout compatibility flags (HWPTAG_LAYOUT_COMPATIBILITY).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LayoutCompatibility {
    pub char_level: u32,
    pub paragraph_level: u32,
    pub section_level: u32,
    pub object_level: u32,
    pub field_level: u32,
}

/// Distribute document data (HWPTAG_DISTRIBUTE_DOC_DATA).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DistributeDocData {
    pub data: Vec<u8>,
}
