use crate::bsl::ast::Token;
use crate::bsl::error::BslError;

/// Tokenizes raw BSL source text into a stream of tokens.
pub fn tokenize(source: &str) -> Result<Vec<(usize, Token)>, BslError> {
    let mut tokens = Vec::new();
    for (line_num, line) in source.lines().enumerate() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with("//") {
            continue;
        }

        // Strip inline comments
        let line = if let Some(pos) = line.find("//") {
            &line[..pos]
        } else {
            line
        }
        .trim();

        if line.is_empty() {
            continue;
        }

        // Tokenize the line
        let line_tokens = tokenize_line(line, line_num + 1)?;
        tokens.extend(line_tokens);
        tokens.push((line_num + 1, Token::Newline));
    }
    tokens.push((source.lines().count().max(1), Token::Eof));
    Ok(tokens)
}

fn tokenize_line(line: &str, line_num: usize) -> Result<Vec<(usize, Token)>, BslError> {
    // First, extract a quoted string as an atomic unit
    let line = line.trim();
    if line.is_empty() {
        return Err(BslError::Syntax {
            line: line_num,
            message: "Empty instruction".to_string(),
        });
    }

    // Get the first word to determine the instruction type
    let first_word = line.split(|c: char| c == ' ' || c == '\t').next().unwrap_or("");
    let is_metadata_keyword = matches!(first_word, "VERSION" | "AUTHOR" | "DESCRIPTION" | "OUTPUT");

    // Split into identifier and the rest (handling = and space)
    // Only split on '=' for metadata keywords (to avoid matching '=' inside strings)
    let (ident, rest) = if is_metadata_keyword {
        if let Some(eq_pos) = line.find('=') {
            let id = line[..eq_pos].trim();
            let after_eq = line[eq_pos + 1..].trim();
            (id, Some((true, after_eq)))
        } else if let Some(space_pos) = line.find(|c: char| c == ' ' || c == '\t') {
            let id = line[..space_pos].trim();
            let after_space = line[space_pos + 1..].trim();
            (id, Some((false, after_space)))
        } else {
            (line, None)
        }
    } else if let Some(space_pos) = line.find(|c: char| c == ' ' || c == '\t') {
        let id = line[..space_pos].trim();
        let after_space = line[space_pos + 1..].trim();
        (id, Some((false, after_space)))
    } else {
        (line, None)
    };

    let mut tokens = Vec::new();
    tokens.push((line_num, Token::Ident(ident.to_string())));

    if let Some((has_equals, value)) = rest {
        if has_equals {
            tokens.push((line_num, Token::Equals));
        }
        if !value.is_empty() {
            tokens.push((line_num, tokenize_value(value, line_num)?));
        }
    }

    Ok(tokens)
}

fn tokenize_value(value: &str, _line_num: usize) -> Result<Token, BslError> {
    if value.starts_with('"') {
        // Find the closing quote
        let inner = if let Some(end) = value[1..].find('"') {
            &value[1..=end]
        } else {
            // No closing quote found — take everything after the opening quote
            &value[1..]
        };
        Ok(Token::StringLit(inner.to_string()))
    } else if let Ok(n) = value.parse::<u64>() {
        Ok(Token::Number(n))
    } else {
        Ok(Token::StringLit(value.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_source() {
        let tokens = tokenize("").unwrap();
        assert!(tokens.iter().any(|(_, t)| matches!(t, Token::Eof)));
    }

    #[test]
    fn test_comment_only() {
        let tokens = tokenize("// just a comment").unwrap();
        assert!(tokens.iter().any(|(_, t)| matches!(t, Token::Eof)));
    }

    #[test]
    fn test_write_statement() {
        let source = r#"WRITE "Hello World""#;
        let tokens = tokenize(source).unwrap();
        assert!(tokens.iter().any(|(_, t)| matches!(t, Token::Ident(s) if s == "WRITE")));
        assert!(tokens.iter().any(|(_, t)| matches!(t, Token::StringLit(s) if s == "Hello World")));
    }

    #[test]
    fn test_version_metadata() {
        let source = r#"VERSION = "1.0.0""#;
        let tokens = tokenize(source).unwrap();
        assert!(tokens.iter().any(|(_, t)| matches!(t, Token::Ident(s) if s == "VERSION")));
        assert!(tokens.iter().any(|(_, t)| matches!(t, Token::Equals)));
        assert!(tokens.iter().any(|(_, t)| matches!(t, Token::StringLit(s) if s == "1.0.0")));
    }

    #[test]
    fn test_output_false() {
        let source = "OUTPUT = false";
        let tokens = tokenize(source).unwrap();
        assert!(tokens.iter().any(|(_, t)| matches!(t, Token::Ident(s) if s == "OUTPUT")));
        assert!(tokens.iter().any(|(_, t)| matches!(t, Token::StringLit(s) if s == "false")));
    }

    #[test]
    fn test_run_statement() {
        let source = r#"RUN "python3 -m venv .venv""#;
        let tokens = tokenize(source).unwrap();
        assert!(tokens.iter().any(|(_, t)| matches!(t, Token::Ident(s) if s == "RUN")));
        assert!(tokens.iter().any(|(_, t)| matches!(t, Token::StringLit(s) if s == "python3 -m venv .venv")));
    }
}
