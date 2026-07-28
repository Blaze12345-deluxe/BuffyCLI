use buffy::bsl::{lexer, parser};

#[test]
fn test_parser_valid_full_script() {
    let source = include_str!("../fixtures/sample-scripts/valid_full.bsl");
    let tokens = lexer::tokenize(source).unwrap();
    let script = parser::parse(tokens).unwrap();
    assert_eq!(script.metadata.len(), 4, "Should have 4 metadata fields");
    assert!(!script.statements.is_empty(), "Should have statements");
}

#[test]
fn test_parser_valid_minimal() {
    let source = include_str!("../fixtures/sample-scripts/valid_minimal.bsl");
    let tokens = lexer::tokenize(source).unwrap();
    let script = parser::parse(tokens).unwrap();
    assert_eq!(script.statements.len(), 2);
}

#[test]
fn test_parser_unknown_instruction() {
    let source = include_str!("../fixtures/sample-scripts/invalid_unknown.bsl");
    let tokens = lexer::tokenize(source).unwrap();
    let result = parser::parse(tokens);
    assert!(result.is_err());
}
