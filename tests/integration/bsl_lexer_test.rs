use buffy::bsl::lexer;

#[test]
fn test_lexer_valid_full_script() {
    let source = include_str!("../fixtures/sample-scripts/valid_full.bsl");
    let tokens = lexer::tokenize(source).unwrap();
    assert!(!tokens.is_empty(), "Should produce tokens");
    assert!(tokens.iter().any(|(_, t)| matches!(t, buffy::bsl::ast::Token::Eof)), "Should end with EOF");
}

#[test]
fn test_lexer_valid_minimal() {
    let source = include_str!("../fixtures/sample-scripts/valid_minimal.bsl");
    let tokens = lexer::tokenize(source).unwrap();
    assert!(!tokens.is_empty());
}

#[test]
fn test_lexer_empty_script() {
    let source = "";
    let tokens = lexer::tokenize(source).unwrap();
    assert!(tokens.iter().any(|(_, t)| matches!(t, buffy::bsl::ast::Token::Eof)));
}

#[test]
fn test_lexer_comment_only() {
    let source = "// just a comment\n// another comment";
    let tokens = lexer::tokenize(source).unwrap();
    // Should just have EOF
    let non_eof: Vec<_> = tokens.iter().filter(|(_, t)| !matches!(t, buffy::bsl::ast::Token::Eof)).collect();
    assert!(non_eof.is_empty() || non_eof.iter().all(|(_, t)| matches!(t, buffy::bsl::ast::Token::Newline)));
}
