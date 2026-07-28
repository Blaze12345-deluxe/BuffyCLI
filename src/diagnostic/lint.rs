use crate::bsl::error::BslError;

/// Lints a BSL script and returns a list of issues.
pub fn lint_script(source: &str) -> Result<Vec<LintIssue>, BslError> {
    let mut issues = Vec::new();
    let tokens = crate::bsl::lexer::tokenize(source)?;
    let script = crate::bsl::parser::parse(tokens)?;

    // Check for recommended metadata
    let has_version = script.metadata.iter().any(|m| matches!(m, crate::bsl::ast::Metadata::Version(_)));
    let has_description = script.metadata.iter().any(|m| matches!(m, crate::bsl::ast::Metadata::Description(_)));
    let has_author = script.metadata.iter().any(|m| matches!(m, crate::bsl::ast::Metadata::Author(_)));

    if !has_version {
        issues.push(LintIssue {
            severity: Severity::Warning,
            message: "Missing VERSION metadata. Consider adding: VERSION = \"YYYY.MM.DD\"".to_string(),
        });
    }
    if !has_description {
        issues.push(LintIssue {
            severity: Severity::Warning,
            message: "Missing DESCRIPTION metadata. Consider adding a brief description.".to_string(),
        });
    }
    if !has_author {
        issues.push(LintIssue {
            severity: Severity::Info,
            message: "Missing AUTHOR metadata. Good practice to include author info.".to_string(),
        });
    }

    // Check statements for potential issues
    for (i, stmt) in script.statements.iter().enumerate() {
        let line_num = i + 1 + script.metadata.len(); // approximate line number

        match stmt {
            crate::bsl::ast::Statement::Write(text) => {
                if text.trim().is_empty() {
                    issues.push(LintIssue {
                        severity: Severity::Warning,
                        message: format!("Line {}: WRITE with empty content. Did you mean CLEAR?", line_num),
                    });
                }
                // Check for potential variable issues
                if text.contains("${") && !text.contains('}') {
                    issues.push(LintIssue {
                        severity: Severity::Error,
                        message: format!("Line {}: Unclosed variable reference '{{' found in WRITE.", line_num),
                    });
                }
            }
            crate::bsl::ast::Statement::Run(cmd) => {
                if cmd.trim().is_empty() {
                    issues.push(LintIssue {
                        severity: Severity::Error,
                        message: format!("Line {}: RUN with empty command.", line_num),
                    });
                }
                // Check for potential run command issues
                if cmd.contains("rm -rf /") || cmd.contains("rm -rf /*") {
                    issues.push(LintIssue {
                        severity: Severity::Warning,
                        message: format!("Line {}: Potentially destructive command detected: {}", line_num, cmd),
                    });
                }
            }
            crate::bsl::ast::Statement::Wait(target) => {
                match target {
                    crate::bsl::ast::WaitTarget::Duration(secs) => {
                        if *secs == 0 {
                            issues.push(LintIssue {
                                severity: Severity::Warning,
                                message: format!("Line {}: WAIT with 0 duration has no effect.", line_num),
                            });
                        }
                    }
                    crate::bsl::ast::WaitTarget::Prompt(msg) => {
                        if msg.trim().is_empty() {
                            issues.push(LintIssue {
                                severity: Severity::Warning,
                                message: format!("Line {}: WAIT prompt with empty message.", line_num),
                            });
                        }
                    }
                }
            }
            crate::bsl::ast::Statement::Exit => {}
            crate::bsl::ast::Statement::Clear => {}
        }
    }

    // Check if there are too many RUN statements (potential performance issue)
    let run_count = script.statements.iter().filter(|s| matches!(s, crate::bsl::ast::Statement::Run(_))).count();
    if run_count > 20 {
        issues.push(LintIssue {
            severity: Severity::Info,
            message: format!("Script has {} RUN statements. Consider breaking into multiple packages for maintainability.", run_count),
        });
    }

    Ok(issues)
}

#[derive(Debug)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Debug)]
pub struct LintIssue {
    pub severity: Severity,
    pub message: String,
}

impl LintIssue {
    /// Returns a colored severity label for display.
    pub fn severity_label(&self) -> &str {
        match self.severity {
            Severity::Info => "info",
            Severity::Warning => "warning",
            Severity::Error => "error",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lint_valid_script() {
        let source = r#"VERSION = "2026.01.01"
AUTHOR = "Test Author"
DESCRIPTION = "A test script"

WRITE "hello"
EXIT
"#;
        let issues = lint_script(source).unwrap();
        // No issues for valid script
        let errors: Vec<_> = issues.iter().filter(|i| matches!(i.severity, Severity::Error)).collect();
        assert!(errors.is_empty(), "Expected no errors in valid script");
    }

    #[test]
    fn test_lint_empty_write_warning() {
        let source = r#"VERSION = "2026.01.01"
AUTHOR = "Test"
DESCRIPTION = "Test"
WRITE ""
EXIT
"#;
        let issues = lint_script(source).unwrap();
        let has_empty_write = issues.iter().any(|i| i.message.contains("WRITE with empty content"));
        assert!(has_empty_write, "Expected warning about empty WRITE");
    }

    #[test]
    fn test_lint_destructive_command() {
        let source = r#"VERSION = "2026.01.01"
AUTHOR = "Test"
DESCRIPTION = "Test"
RUN "rm -rf /"
EXIT
"#;
        let issues = lint_script(source).unwrap();
        let has_destructive = issues.iter().any(|i| i.message.contains("destructive command"));
        assert!(has_destructive, "Expected warning about destructive command");
    }

    #[test]
    fn test_lint_missing_metadata_warnings() {
        let source = r#"WRITE "hello"
EXIT
"#;
        let issues = lint_script(source).unwrap();
        let has_version_warning = issues.iter().any(|i| i.message.contains("Missing VERSION"));
        let has_desc_warning = issues.iter().any(|i| i.message.contains("Missing DESCRIPTION"));
        assert!(has_version_warning, "Expected VERSION warning");
        assert!(has_desc_warning, "Expected DESCRIPTION warning");
    }
}
