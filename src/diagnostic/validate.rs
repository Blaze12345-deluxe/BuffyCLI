use crate::bsl::error::BslError;

/// Validates the metadata of a BSL script.
pub fn validate_script(source: &str) -> Result<ValidationReport, BslError> {
    let tokens = crate::bsl::lexer::tokenize(source)?;
    let script = crate::bsl::parser::parse(tokens)?;

    let mut warnings = Vec::new();

    let has_version = script.metadata.iter().any(|m| matches!(m, crate::bsl::ast::Metadata::Version(_)));
    let has_description = script.metadata.iter().any(|m| matches!(m, crate::bsl::ast::Metadata::Description(_)));
    let has_author = script.metadata.iter().any(|m| matches!(m, crate::bsl::ast::Metadata::Author(_)));

    if !has_version {
        warnings.push("Missing VERSION metadata".to_string());
    }
    if !has_description {
        warnings.push("Missing DESCRIPTION metadata".to_string());
    }
    if !has_author {
        warnings.push("Missing AUTHOR metadata".to_string());
    }

    Ok(ValidationReport { warnings })
}

pub struct ValidationReport {
    pub warnings: Vec<String>,
}
