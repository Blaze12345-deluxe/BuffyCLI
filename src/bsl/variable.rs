/// Context available during BSL execution.
pub struct ExecutionContext {
    /// Command-line arguments passed to the BSL script.
    pub args: Vec<String>,
}

/// Resolves `${VAR}` placeholders in a string.
///
/// Uses regex `\$\{(\w+)\}` to find variable references, avoiding
/// false matches where one variable name is a prefix of another
/// (e.g., `${HOME}` won't incorrectly match inside `${HOMEDIR}` or `${HOME}_foo`).
pub fn resolve(text: &str, ctx: &ExecutionContext) -> String {
    let builtins = builtin_variables();

    // Build a combined lookup: first built-in vars, then numbered args
    let mut lookup = builtins;
    for (i, arg) in ctx.args.iter().enumerate() {
        let key = (i + 1).to_string();
        lookup.insert(key, arg.clone());
    }

    // Use regex to match ${VAR_NAME} and replace each match individually
    let re = regex_lite::Regex::new(r"\$\{(\w+)\}").unwrap();
    let result = re.replace_all(text, |caps: &regex_lite::Captures| {
        let var_name = &caps[1];
        lookup.get(var_name).cloned().unwrap_or_else(|| caps[0].to_string())
    });

    result.to_string()
}

fn builtin_variables() -> std::collections::HashMap<String, String> {
    let mut vars = std::collections::HashMap::new();

    vars.insert("HOME".to_string(), std::env::var("HOME").unwrap_or_default());
    vars.insert("USER".to_string(), std::env::var("USER").unwrap_or_default());
    vars.insert("PWD".to_string(), std::env::var("PWD").unwrap_or_default());
    vars.insert("TEMP".to_string(), std::env::temp_dir().to_string_lossy().to_string());

    let now = chrono::Local::now();
    vars.insert("DATE".to_string(), now.format("%Y-%m-%d").to_string());
    vars.insert("TIME".to_string(), now.format("%H:%M:%S").to_string());

    vars
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_home() {
        let ctx = ExecutionContext { args: vec![] };
        let result = resolve("${HOME}", &ctx);
        assert!(!result.is_empty());
        assert!(!result.contains("${HOME}"));
    }

    #[test]
    fn test_resolve_args() {
        let ctx = ExecutionContext {
            args: vec!["first".to_string(), "second".to_string()],
        };
        assert_eq!(resolve("${1}", &ctx), "first");
        assert_eq!(resolve("${2}", &ctx), "second");
    }

    #[test]
    fn test_resolve_in_args() {
        let ctx = ExecutionContext {
            args: vec!["myfile.txt".to_string()],
        };
        assert_eq!(resolve("cat ${1}", &ctx), "cat myfile.txt");
    }

    #[test]
    fn test_unknown_var_left_as_is() {
        let ctx = ExecutionContext { args: vec![] };
        let result = resolve("${UNKNOWN_VAR}", &ctx);
        assert_eq!(result, "${UNKNOWN_VAR}");
    }

    #[test]
    fn test_no_false_match_on_prefix() {
        let ctx = ExecutionContext { args: vec![] };
        // ${HOME} should NOT match inside ${HOMEDIR} or ${HOME_PATH}
        let result = resolve("${HOMEDIR} ${HOME_PATH}", &ctx);
        assert_eq!(result, "${HOMEDIR} ${HOME_PATH}");
    }

    #[test]
    fn test_multiple_vars_in_one_string() {
        let ctx = ExecutionContext { args: vec![] };
        let result = resolve("${HOME}/${USER}", &ctx);
        assert!(!result.contains("${HOME}"));
        assert!(!result.contains("${USER}"));
        assert!(result.contains('/'));
    }

    #[test]
    fn test_adjacent_vars() {
        let ctx = ExecutionContext {
            args: vec!["a".to_string(), "b".to_string()],
        };
        assert_eq!(resolve("${1}${2}", &ctx), "ab");
    }

    #[test]
    fn test_date_and_time_format() {
        let ctx = ExecutionContext { args: vec![] };
        let result = resolve("${DATE} ${TIME}", &ctx);
        // Should match date format YYYY-MM-DD and time HH:MM:SS
        assert!(result.contains('-'), "Date should contain hyphens");
        assert!(result.contains(':'), "Time should contain colons");
    }
}
