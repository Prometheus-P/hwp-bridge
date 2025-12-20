// crates/hwp-core/src/parser/ole.rs

use cfb::CompoundFile;
use hwp_types::{FileHeader, HwpError};
use std::io::{Read, Seek};

use super::header::parse_file_header;
use super::summary::{HwpSummaryInfo, parse_summary_info};

/// HWP OLE 컨테이너 래퍼
pub struct HwpOleFile<F: Read + Seek> {
    cfb: CompoundFile<F>,
    header: FileHeader,
}

impl<F: Read + Seek> HwpOleFile<F> {
    /// OLE 파일을 열고 FileHeader를 파싱합니다.
    /// Fail-Fast: 암호화/배포용 문서는 즉시 에러를 반환합니다.
    pub fn open(inner: F) -> Result<Self, HwpError> {
        let mut cfb = CompoundFile::open(inner).map_err(|e| HwpError::OleError(e.to_string()))?;

        // FileHeader 스트림 읽기
        let header_data = Self::read_stream(&mut cfb, "/FileHeader")?;
        let header = parse_file_header(&header_data)?;

        // Fail-Fast 검증
        header.validate()?;

        Ok(Self { cfb, header })
    }

    /// FileHeader 반환
    pub fn header(&self) -> &FileHeader {
        &self.header
    }

    /// 스트림 데이터 읽기
    fn read_stream(cfb: &mut CompoundFile<F>, path: &str) -> Result<Vec<u8>, HwpError> {
        let mut stream = cfb
            .open_stream(path)
            .map_err(|e| HwpError::OleError(format!("Failed to open stream '{}': {}", path, e)))?;

        let mut data = Vec::new();
        stream.read_to_end(&mut data).map_err(HwpError::Io)?;

        Ok(data)
    }

    /// 지정된 경로의 스트림 존재 여부 확인
    pub fn has_stream(&self, path: &str) -> bool {
        self.cfb.is_stream(path)
    }

    /// 스트림 데이터를 읽어 반환
    pub fn read(&mut self, path: &str) -> Result<Vec<u8>, HwpError> {
        Self::read_stream(&mut self.cfb, path)
    }

    /// DocInfo 스트림 읽기
    pub fn read_doc_info(&mut self) -> Result<Vec<u8>, HwpError> {
        self.read("/DocInfo")
    }

    /// BodyText Section 스트림 목록 조회
    pub fn list_sections(&self) -> Vec<String> {
        let mut sections = Vec::new();
        let mut idx = 0;
        loop {
            let path = format!("/BodyText/Section{}", idx);
            if self.has_stream(&path) {
                sections.push(path);
                idx += 1;
            } else {
                break;
            }
        }
        sections
    }

    /// Section 스트림 읽기
    ///
    /// 섹션이 존재하지 않으면 NotFound 에러를 반환합니다.
    pub fn read_section(&mut self, index: usize) -> Result<Vec<u8>, HwpError> {
        let path = format!("/BodyText/Section{}", index);
        if !self.has_stream(&path) {
            return Err(HwpError::NotFound(format!("Section {} not found", index)));
        }
        self.read(&path)
    }

    /// HwpSummaryInformation 스트림 읽기 및 파싱
    ///
    /// HWP 문서의 메타데이터(제목, 저자, 생성일 등)를 반환합니다.
    /// 스트림이 없거나 파싱 실패 시 기본값을 반환합니다.
    pub fn read_summary_info(&mut self) -> HwpSummaryInfo {
        // HWP uses "\x05HwpSummaryInformation" stream name
        const SUMMARY_STREAM: &str = "\x05HwpSummaryInformation";

        if !self.has_stream(SUMMARY_STREAM) {
            return HwpSummaryInfo::default();
        }

        match self.read(SUMMARY_STREAM) {
            Ok(data) => parse_summary_info(&data).unwrap_or_default(),
            Err(_) => HwpSummaryInfo::default(),
        }
    }
}
