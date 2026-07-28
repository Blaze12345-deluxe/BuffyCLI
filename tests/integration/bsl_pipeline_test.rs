use buffy::bsl;

#[test]
fn test_full_pipeline_valid_script() {
    let source = r#"
VERSION = "2026.07.27"
AUTHOR = "Test"
DESCRIPTION = "Integration test"
OUTPUT = false

WRITE "Hello from BSL"
RUN "echo 'shell command'"
EXIT
"#;

    let tokens = bsl::lexer::tokenize(source).unwrap();
    let script = bsl::parser::parse(tokens).unwrap();

    assert_eq!(script.metadata.len(), 4);
    assert_eq!(script.statements.len(), 3);
    assert!(!script.get_output_mode());

    // Check the parsed statements
    match &script.statements[0] {
        bsl::ast::Statement::Write(msg) => assert_eq!(msg, "Hello from BSL"),
        _ => panic!("Expected WRITE"),
    }
    match &script.statements[1] {
        bsl::ast::Statement::Run(cmd) => assert_eq!(cmd, "echo 'shell command'"),
        _ => panic!("Expected RUN"),
    }
    match &script.statements[2] {
        bsl::ast::Statement::Exit => {}
        _ => panic!("Expected EXIT"),
    }
}

#[test]
fn test_full_pipeline_with_metadata() {
    let source = r#"
VERSION = "1.0.0"
AUTHOR = "Alice"
DESCRIPTION = "Test script"
OUTPUT = true
"#;

    let tokens = bsl::lexer::tokenize(source).unwrap();
    let script = bsl::parser::parse(tokens).unwrap();

    let mut has_version = false;
    let mut has_author = false;
    let mut has_desc = false;
    let mut has_output = false;

    for m in &script.metadata {
        match m {
            bsl::ast::Metadata::Version(v) => { has_version = true; assert_eq!(v, "1.0.0"); }
            bsl::ast::Metadata::Author(a) => { has_author = true; assert_eq!(a, "Alice"); }
            bsl::ast::Metadata::Description(d) => { has_desc = true; assert_eq!(d, "Test script"); }
            bsl::ast::Metadata::Output(o) => { has_output = true; assert!(*o); }
        }
    }

    assert!(has_version);
    assert!(has_author);
    assert!(has_desc);
    assert!(has_output);
    assert!(script.get_output_mode());  // OUTPUT = true
}

#[test]
fn test_full_pipeline_empty_script_is_valid() {
    let source = "";

    let tokens = bsl::lexer::tokenize(source).unwrap();
    let script = bsl::parser::parse(tokens).unwrap();
    assert_eq!(script.metadata.len(), 0);
    assert_eq!(script.statements.len(), 0);
}

#[test]
fn test_full_pipeline_invalid_syntax() {
    let source = r#"RUN "#;  // RUN with no argument
    let result = bsl::lexer::tokenize(source);
    assert!(result.is_ok());
    let tokens = result.unwrap();
    let parse_result = bsl::parser::parse(tokens);
    assert!(parse_result.is_err());
}

#[test]
fn test_full_pipeline_run_with_spaces() {
    let source = r#"RUN "python3 -m venv .venv""#;
    let tokens = bsl::lexer::tokenize(source).unwrap();
    let script = bsl::parser::parse(tokens).unwrap();

    match &script.statements[0] {
        bsl::ast::Statement::Run(cmd) => assert_eq!(cmd, "python3 -m venv .venv"),
        _ => panic!("Expected RUN"),
    }
}

#[test]
fn test_full_pipeline_interpret_write_and_exit() {
    let source = r#"
OUTPUT = false
WRITE "test output"
EXIT
"#;
    let tokens = bsl::lexer::tokenize(source).unwrap();
    let script = bsl::parser::parse(tokens).unwrap();
    let result = bsl::interpreter::interpret(&script, &[]);
    assert!(result.is_ok());
}

#[test]
fn test_full_pipeline_variable_substitution() {
    let source = r#"
OUTPUT = false
WRITE "${1}"
EXIT
"#;
    let tokens = bsl::lexer::tokenize(source).unwrap();
    let script = bsl::parser::parse(tokens).unwrap();
    let result = bsl::interpreter::interpret(&script, &["hello"]);
    assert!(result.is_ok());
}

#[test]
fn test_full_pipeline_wait_duration() {
    let source = "WAIT 0\nEXIT";
    let tokens = bsl::lexer::tokenize(source).unwrap();
    let script = bsl::parser::parse(tokens).unwrap();

    match &script.statements[0] {
        bsl::ast::Statement::Wait(bsl::ast::WaitTarget::Duration(n)) => assert_eq!(*n, 0),
        _ => panic!("Expected WAIT 0"),
    }
}
