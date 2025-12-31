// crates/hwp-core/src/parser/chart.rs
//! Legacy chart (Contents) parser based on chart revision 1.2 spec.

use std::collections::HashMap;

use encoding_rs::EUC_KR;
use hwp_types::{CellBlock, StructuredParagraph, StructuredTable, StructuredTableCell};

use super::{chart_schema::chart_schema, chart_types::ChartFieldKind};

const MAX_OBJECTS: usize = 50000;
const MAX_COLLECTION_COUNT: usize = 20000;
const MAX_STRING_BYTES: usize = 4 * 1024 * 1024;

#[derive(Debug)]
pub struct ChartParseResult {
    pub title: Option<String>,
    pub chart_type: Option<String>,
    pub data_grid: Option<StructuredTable>,
}

#[derive(Default)]
struct ChartParseState {
    title: Option<String>,
    chart_type: Option<i32>,
    row_count: Option<usize>,
    col_count: Option<usize>,
    row_labels: Vec<String>,
    col_labels: Vec<String>,
    data_values: Vec<String>,
    object_count: usize,
}

impl ChartParseState {
    fn push_row_label(&mut self, value: String) {
        if value.is_empty() || self.row_labels.contains(&value) {
            return;
        }
        self.row_labels.push(value);
    }

    fn push_col_label(&mut self, value: String) {
        if value.is_empty() || self.col_labels.contains(&value) {
            return;
        }
        self.col_labels.push(value);
    }

    fn push_data_value(&mut self, value: String) {
        if value.is_empty() {
            return;
        }
        self.data_values.push(value);
    }

    fn finalize(self) -> ChartParseResult {
        let chart_type = self
            .chart_type
            .and_then(chart_type_label)
            .map(|value| value.to_string());
        let data_grid = build_data_grid(&self);

        ChartParseResult {
            title: self.title,
            chart_type,
            data_grid,
        }
    }
}

#[derive(Debug)]
struct ChartParseError;

#[derive(Clone)]
struct ChartTypeInfo {
    name: String,
    _version: i32,
}

struct ChartReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> ChartReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    fn read_u8(&mut self) -> Option<u8> {
        if self.pos >= self.data.len() {
            return None;
        }
        let value = self.data[self.pos];
        self.pos += 1;
        Some(value)
    }

    fn read_u16(&mut self) -> Option<u16> {
        if self.pos + 2 > self.data.len() {
            return None;
        }
        let value = u16::from_le_bytes([self.data[self.pos], self.data[self.pos + 1]]);
        self.pos += 2;
        Some(value)
    }

    fn read_u32(&mut self) -> Option<u32> {
        if self.pos + 4 > self.data.len() {
            return None;
        }
        let value = u32::from_le_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
        ]);
        self.pos += 4;
        Some(value)
    }

    fn read_i32(&mut self) -> Option<i32> {
        self.read_u32().map(|value| value as i32)
    }

    fn read_f32(&mut self) -> Option<f32> {
        self.read_u32()
            .map(|value| f32::from_le_bytes(value.to_le_bytes()))
    }

    fn read_f64(&mut self) -> Option<f64> {
        if self.pos + 8 > self.data.len() {
            return None;
        }
        let value = f64::from_le_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
            self.data[self.pos + 4],
            self.data[self.pos + 5],
            self.data[self.pos + 6],
            self.data[self.pos + 7],
        ]);
        self.pos += 8;
        Some(value)
    }

    fn read_bool(&mut self) -> Option<bool> {
        if self.remaining() >= 4 {
            let bytes = &self.data[self.pos..self.pos + 4];
            if bytes[1] == 0 && bytes[2] == 0 && bytes[3] == 0 {
                self.pos += 4;
                return Some(bytes[0] != 0);
            }
        }
        self.read_u8().map(|value| value != 0)
    }

    fn read_cstring(&mut self) -> Option<String> {
        let start = self.pos;
        let mut end = start;
        while end < self.data.len() && self.data[end] != 0 {
            end += 1;
            if end - start > 128 {
                return None;
            }
        }
        if end >= self.data.len() {
            return None;
        }
        let bytes = &self.data[start..end];
        self.pos = end + 1;
        let value = String::from_utf8_lossy(bytes).trim().to_string();
        if value.is_empty() { None } else { Some(value) }
    }

    fn peek_u16(&self) -> Option<u16> {
        if self.pos + 2 > self.data.len() {
            return None;
        }
        Some(u16::from_le_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
        ]))
    }

    fn read_string(&mut self) -> Option<String> {
        let start = self.pos;
        let count = self.peek_u16()?;
        let (count, header_len) = if count == 0xFFFF {
            let mut temp = self.clone();
            temp.read_u16()?;
            let len = temp.read_u32()? as usize;
            (len, 6)
        } else {
            (count as usize, 2)
        };

        let remaining = self.data.len().saturating_sub(start);
        let bytes_len = count.saturating_mul(2);
        let use_utf16 = bytes_len + header_len <= remaining
            && looks_like_utf16le(&self.data[start + header_len..start + header_len + bytes_len]);

        let string_bytes_len = if use_utf16 { bytes_len } else { count };

        if header_len + string_bytes_len <= remaining && string_bytes_len <= MAX_STRING_BYTES {
            self.pos = start + header_len;
            let bytes = &self.data[self.pos..self.pos + string_bytes_len];
            self.pos += string_bytes_len;
            let value = if use_utf16 {
                decode_utf16(bytes)
            } else {
                decode_ansi(bytes)
            };
            return Some(clean_text(&value));
        }

        self.pos = start;
        let fallback = self.read_i32()?;
        Some(fallback.to_string())
    }
}

impl<'a> Clone for ChartReader<'a> {
    fn clone(&self) -> Self {
        Self {
            data: self.data,
            pos: self.pos,
        }
    }
}

pub fn parse_chart_contents(data: &[u8]) -> Option<ChartParseResult> {
    let mut reader = ChartReader::new(data);
    let mut registry: HashMap<u32, ChartTypeInfo> = HashMap::new();
    let mut state = ChartParseState::default();

    while reader.remaining() >= 8 && state.object_count < MAX_OBJECTS {
        let result = parse_chart_object(&mut reader, &mut registry, &mut state, 0);
        if result.is_err() {
            break;
        }
    }

    let result = state.finalize();
    if result.title.is_none() && result.chart_type.is_none() && result.data_grid.is_none() {
        None
    } else {
        Some(result)
    }
}

fn parse_chart_object(
    reader: &mut ChartReader<'_>,
    registry: &mut HashMap<u32, ChartTypeInfo>,
    state: &mut ChartParseState,
    depth: usize,
) -> Result<(), ChartParseError> {
    if depth > 32 {
        return Err(ChartParseError);
    }

    let _object_id = reader.read_u32().ok_or(ChartParseError)?;
    let stored_type_id = reader.read_u32().ok_or(ChartParseError)?;

    let object_info = if let Some(info) = registry.get(&stored_type_id) {
        info.clone()
    } else {
        let name = reader.read_cstring().ok_or(ChartParseError)?;
        let version = reader.read_i32().ok_or(ChartParseError)?;
        let info = ChartTypeInfo {
            name: name.clone(),
            _version: version,
        };
        registry.insert(stored_type_id, info.clone());
        info
    };

    state.object_count += 1;
    if state.object_count > MAX_OBJECTS {
        return Err(ChartParseError);
    }

    let object_name = object_info.name.as_str();
    if let Some(_item_type) = collection_item_type(object_name) {
        let count = reader.read_i32().ok_or(ChartParseError)?.max(0) as usize;
        let count = count.min(MAX_COLLECTION_COUNT);
        for _ in 0..count {
            parse_chart_object(reader, registry, state, depth + 1)?;
        }
        return Ok(());
    }

    let schema = chart_schema(object_name).ok_or(ChartParseError)?;
    for field in schema {
        match field.kind {
            ChartFieldKind::Boolean => {
                let value = reader.read_bool().ok_or(ChartParseError)?;
                record_bool(state, object_name, field.name, value);
            }
            ChartFieldKind::Integer => {
                let value = reader.read_i32().ok_or(ChartParseError)?;
                record_int(state, object_name, field.name, value);
            }
            ChartFieldKind::Long => {
                let value = reader.read_i32().ok_or(ChartParseError)?;
                record_int(state, object_name, field.name, value);
            }
            ChartFieldKind::Single => {
                reader.read_f32().ok_or(ChartParseError)?;
            }
            ChartFieldKind::Double => {
                reader.read_f64().ok_or(ChartParseError)?;
            }
            ChartFieldKind::String => {
                let value = reader.read_string().unwrap_or_default();
                record_string(state, object_name, field.name, value);
            }
            ChartFieldKind::Object(expected) => {
                let _ = expected;
                parse_chart_object(reader, registry, state, depth + 1)?;
            }
        }
    }

    Ok(())
}

fn collection_item_type(name: &str) -> Option<&'static str> {
    match name {
        "Attributes" => Some("Attribute"),
        "DataPoints" => Some("DataPoint"),
        "Labels" => Some("Label"),
        "LightSources" => Some("LightSource"),
        "SeriesCollection" => Some("Series"),
        _ => None,
    }
}

fn record_int(state: &mut ChartParseState, object: &str, field: &str, value: i32) {
    match (object, field) {
        ("VtChart", "ChartType") => state.chart_type = Some(value),
        ("VtChart", "RowCount") | ("DataGrid", "RowCount") => {
            if value > 0 {
                state.row_count = Some(value as usize);
            }
        }
        ("VtChart", "ColumnCount") | ("DataGrid", "ColumnCount") => {
            if value > 0 {
                state.col_count = Some(value as usize);
            }
        }
        _ => {}
    }
}

fn record_bool(_state: &mut ChartParseState, _object: &str, _field: &str, _value: bool) {}

fn record_string(state: &mut ChartParseState, object: &str, field: &str, value: String) {
    let value = value.trim().to_string();
    if value.is_empty() {
        return;
    }
    match (object, field) {
        ("VtChart", "TitleText") => {
            if state.title.is_none() {
                state.title = Some(value);
            }
        }
        ("Title", "Text") => {
            if state.title.is_none() {
                state.title = Some(value);
            }
        }
        ("VtChart", "ColumnLabel")
        | ("DataGrid", "ColumnLabel")
        | ("DataGrid", "CompositeColumnLabel") => state.push_col_label(value),
        ("VtChart", "RowLabel") | ("DataGrid", "RowLabel") | ("DataGrid", "CompositeRowLabel") => {
            state.push_row_label(value)
        }
        ("VtChart", "Data") => state.push_data_value(value),
        ("Series", "LegendText") => state.push_col_label(value),
        _ => {}
    }
}

fn build_data_grid(state: &ChartParseState) -> Option<StructuredTable> {
    let mut rows = state.row_count.unwrap_or(0).max(state.row_labels.len());
    let mut cols = state.col_count.unwrap_or(0).max(state.col_labels.len());

    if rows == 0 && cols == 0 && !state.data_values.is_empty() {
        rows = state.data_values.len();
        cols = 1;
    }

    if rows == 0 || cols == 0 {
        return None;
    }

    let has_col_labels = !state.col_labels.is_empty();
    let has_row_labels = !state.row_labels.is_empty();
    let row_offset = if has_col_labels { 1 } else { 0 };
    let col_offset = if has_row_labels { 1 } else { 0 };
    let total_rows = rows + row_offset;
    let total_cols = cols + col_offset;

    let mut table = StructuredTable::new(total_rows, total_cols);
    if has_col_labels {
        table.header_rows = 1;
    }

    let mut grid = Vec::with_capacity(total_rows);
    for r in 0..total_rows {
        let mut row = Vec::with_capacity(total_cols);
        for c in 0..total_cols {
            row.push(StructuredTableCell::default().with_position(r, c));
        }
        grid.push(row);
    }

    if has_col_labels {
        for (idx, label) in state.col_labels.iter().take(cols).enumerate() {
            let cell = &mut grid[0][idx + col_offset];
            cell.is_header = true;
            cell.blocks
                .push(CellBlock::Paragraph(StructuredParagraph::from_text(
                    label.clone(),
                )));
        }
    }

    if has_row_labels {
        for (idx, label) in state.row_labels.iter().take(rows).enumerate() {
            let cell = &mut grid[idx + row_offset][0];
            cell.blocks
                .push(CellBlock::Paragraph(StructuredParagraph::from_text(
                    label.clone(),
                )));
        }
    }

    let mut value_iter = state.data_values.iter();
    for r in 0..rows {
        for c in 0..cols {
            if let Some(value) = value_iter.next() {
                let cell = &mut grid[r + row_offset][c + col_offset];
                cell.blocks
                    .push(CellBlock::Paragraph(StructuredParagraph::from_text(
                        value.clone(),
                    )));
            }
        }
    }

    for row in grid {
        table.add_row(row);
    }

    Some(table)
}

fn chart_type_label(value: i32) -> Option<&'static str> {
    match value {
        0 => Some("3D 막대"),
        1 => Some("2D 막대"),
        2 => Some("3D 선"),
        3 => Some("2D 선"),
        4 => Some("3D 영역"),
        5 => Some("2D 영역"),
        6 => Some("3D 계단"),
        7 => Some("2D 계단"),
        8 => Some("3D 조합"),
        9 => Some("2D 조합"),
        10 => Some("3D 가로 막대"),
        11 => Some("2D 가로 막대"),
        12 => Some("3D 클러스터 막대"),
        13 => Some("3D 파이"),
        14 => Some("2D 파이"),
        15 => Some("2D 도넛"),
        16 => Some("2D XY"),
        17 => Some("2D 원추"),
        18 => Some("2D 방사"),
        19 => Some("2D 풍선"),
        20 => Some("2D Hi-Lo"),
        21 => Some("2D 간트"),
        22 => Some("3D 간트"),
        23 => Some("3D 평면"),
        24 => Some("2D 등고선"),
        25 => Some("3D 산포"),
        26 => Some("3D XYZ"),
        _ => None,
    }
}

fn decode_utf16(bytes: &[u8]) -> String {
    if !bytes.len().is_multiple_of(2) {
        return String::new();
    }
    let mut units = Vec::with_capacity(bytes.len() / 2);
    for chunk in bytes.chunks_exact(2) {
        units.push(u16::from_le_bytes([chunk[0], chunk[1]]));
    }
    String::from_utf16(&units).unwrap_or_default()
}

fn decode_ansi(bytes: &[u8]) -> String {
    let (cow, _, _) = EUC_KR.decode(bytes);
    cow.to_string()
}

fn looks_like_utf16le(bytes: &[u8]) -> bool {
    let mut zero_count = 0usize;
    let mut sample_count = 0usize;
    for chunk in bytes.chunks_exact(2).take(32) {
        sample_count += 1;
        if chunk[1] == 0 {
            zero_count += 1;
        }
    }
    sample_count > 0 && zero_count * 2 >= sample_count
}

fn clean_text(input: &str) -> String {
    input
        .chars()
        .filter(|c| !c.is_control())
        .collect::<String>()
        .trim()
        .to_string()
}
