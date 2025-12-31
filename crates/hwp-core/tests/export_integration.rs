use hwp_core::export::parse_structured_document;
use hwp_core::parser::SectionLimits;
use std::fs::File;
use std::io::BufReader;

#[test]
fn test_should_parse_sample_hwp_file() {
    // Arrange
    let file_path = "tests/fixtures/sample2.hwp";
    let file = File::open(file_path).expect("Could not open sample HWP file");
    let reader = BufReader::new(file);

    let limits = SectionLimits::default();

    // Act
    let doc = parse_structured_document(reader, None, limits).expect("Failed to parse document");

    // Assert
    assert!(!doc.sections.is_empty(), "Document should contain sections");
    assert!(
        doc.paragraph_count() > 0,
        "Document should contain paragraphs"
    );
    // Add more specific assertions based on expected content of sample.hwp
}
