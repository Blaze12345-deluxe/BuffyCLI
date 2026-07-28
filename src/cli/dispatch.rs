use crate::cli::args::CliArgs;
use crate::error::Result;

pub fn execute(args: CliArgs) -> Result<()> {
    if args.version { return print_version(); }
    if args.about { return print_about(); }
    if args.list { return list_commands(); }
    if args.doctor { return run_doctor(); }
    if args.self_update { return self_update(); }
    if args.check_update { return check_update(); }
    if args.clean { return clean_cache(); }
    if args.repair { return repair(); }
    if args.reset { return reset(); }
    if args.discover { return discover(); }

    if let Some(packages) = args.install { return install_packages(packages); }
    if let Some(name) = args.uninstall { return uninstall_package(name); }
    if args.update_packages { return update_all_packages(); }
    if let Some(name) = args.update_package { return update_one_package(name); }
    if let Some(name) = args.info { return show_info(name); }
    if args.outdated { return show_outdated(); }
    if let Some(name) = args.verify { return verify_package(name); }
    if let Some(alias_args) = args.alias { return manage_aliases(alias_args); }
    if let Some(repo_args) = args.repo { return manage_repositories(repo_args); }
    if let Some(log_args) = args.logs { return manage_logs(log_args); }
    if let Some(path) = args.check { return check_syntax(path); }
    if let Some(path) = args.validate { return validate_metadata(path); }
    if let Some(path) = args.run { return run_script(path, &args.command); }
    if let Some(path) = args.benchmark { return benchmark_script(path); }
    if let Some(shell) = args.completion { return generate_completion(shell); }

    if !args.command.is_empty() { return execute_bsl_command(args.command); }
    show_welcome()
}

fn print_version() -> Result<()> {
    println!("buffy v{}", env!("CARGO_PKG_VERSION"));
    Ok(())
}

fn print_about() -> Result<()> {
    println!("Buffy CLI Automation Framework v{}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("A lightweight, cross-platform CLI automation framework.");
    println!("Commands are written in BSL (Buffy Script Language) and distributed");
    println!("via GitHub package repositories.");
    println!();
    println!("Key features:");
    println!("  . Simple script language - no Python, no Bash needed");
    println!("  . Built-in package manager with GitHub repositories");
    println!("  . SHA-256 package verification");
    println!("  . System discovery: suggests packages for detected tools");
    println!();
    println!("Repository: https://github.com/Blaze12345-deluxe/BuffyCLI");
    Ok(())
}

fn show_welcome() -> Result<()> {
    use crate::logger::formatter;
    formatter::write("Buffy CLI Automation Framework");
    formatter::write("");
    formatter::write("A lightweight framework for creating and sharing terminal commands.");
    formatter::write("Commands are written in BSL (Buffy Script Language) - a simple text format.");
    formatter::write("");
    formatter::info("Usage:");
    formatter::write("  buffy <command>          Run an installed BSL command");
    formatter::write("  buffy --run <file.bsl>   Run a .bsl file without installing");
    formatter::success("  buffy --install <pkg>    Install a package");
    formatter::write("  buffy --list             List installed commands");
    formatter::info("  buffy --help             Show all available flags");
    formatter::write("");
    formatter::info("Example: buffy pip-env");
    formatter::write("  Creates a Python virtual environment in the current directory.");
    Ok(())
}

fn list_commands() -> Result<()> {
    let commands_dir = crate::config::buffy_home::commands_dir();
    if !commands_dir.exists() { println!("No commands installed."); return Ok(()); }

    let entries = std::fs::read_dir(&commands_dir)?;
    let mut found = false;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if path.is_dir() {
            let sub_entries = std::fs::read_dir(&path)?;
            let subcommands: Vec<String> = sub_entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "bsl"))
                .map(|e| e.file_name().to_string_lossy().trim_end_matches(".bsl").to_string())
                .collect();

            if !subcommands.is_empty() {
                found = true;
                if subcommands.len() == 1 { println!("  {}", name); }
                else { println!("  {} ({})", name, subcommands.join(", ")); }
            }
        } else if path.extension().map_or(false, |ext| ext == "bsl") {
            found = true;
            println!("  {}", name.trim_end_matches(".bsl"));
        }
    }

    if !found { println!("No commands installed."); }
    Ok(())
}

fn run_doctor() -> Result<()> {
    println!("Running system diagnostics...\n");
    let report = crate::diagnostic::doctor::run_doctor()?;
    println!();
    if report.all_passed() { println!("All checks passed."); }
    else { println!("{} check(s) failed.", report.checks.iter().filter(|(_, p)| !p).count()); }
    Ok(())
}

fn self_update() -> Result<()> {
    println!("Checking for updates...");
    match check_for_update() {
        Ok(Some((latest, url))) => {
            println!("  Current version: v{}", env!("CARGO_PKG_VERSION"));
            println!("  Latest version:  {}", latest);
            println!();
            println!("Download the latest release from:");
            println!("  {}", url);
            println!();
            println!("Or install via cargo:");
            println!("  cargo install buffy");
        }
        Ok(None) => {
            println!("  You're running the latest version (v{}).", env!("CARGO_PKG_VERSION"));
        }
        Err(_) => {
            eprintln!("Could not check for updates. Check manually at:");
            eprintln!("  https://github.com/Blaze12345-deluxe/BuffyCLI/releases");
        }
    }
    Ok(())
}

fn check_update() -> Result<()> {
    println!("Checking for updates...");
    match check_for_update() {
        Ok(Some((latest, _))) => {
            let current = env!("CARGO_PKG_VERSION");
            if latest.trim_start_matches('v') != current {
                println!("  A new version is available: {} (current: v{})", latest, current);
                println!("  Run 'buffy --update' for download instructions.");
            } else {
                println!("  You're up to date (v{}).", current);
            }
        }
        Ok(None) => {
            println!("  You're up to date (v{}).", env!("CARGO_PKG_VERSION"));
        }
        Err(_) => {
            eprintln!("  Could not check for updates.");
            eprintln!("  Check manually: https://github.com/Blaze12345-deluxe/BuffyCLI/releases");
        }
    }
    Ok(())
}

/// Checks the GitHub API for the latest release.
/// Returns (tag_name, html_url) if a newer version exists, or None if up to date.
fn check_for_update() -> std::result::Result<Option<(String, String)>, String> {
    let url = "https://api.github.com/repos/Blaze12345-deluxe/BuffyCLI/releases/latest";
    match ureq::get(url)
        .set("User-Agent", "buffy-cli")
        .set("Accept", "application/json")
        .call()
    {
        Ok(response) => {
            match serde_json::from_reader::<_, serde_json::Value>(response.into_reader()) {
                Ok(json) => {
                    let tag = json.get("tag_name").and_then(|v| v.as_str()).unwrap_or("");
                    let html_url = json.get("html_url").and_then(|v| v.as_str()).unwrap_or("");

                    if !tag.is_empty() {
                        let current = env!("CARGO_PKG_VERSION");
                        let latest = tag.trim_start_matches('v');
                        if latest != current {
                            Ok(Some((tag.to_string(), html_url.to_string())))
                        } else {
                            Ok(None)
                        }
                    } else {
                        Err("Could not parse release info".to_string())
                    }
                }
                Err(e) => Err(format!("JSON parse error: {}", e)),
            }
        }
        Err(e) => Err(format!("HTTP error: {}", e)),
    }
}

fn clean_cache() -> Result<()> {
    let cache_dir = crate::config::buffy_home::cache_dir();
    if !cache_dir.exists() { println!("Cache directory is empty."); return Ok(()); }

    let mut deleted = 0u64;
    let mut freed = 0u64;
    if let Ok(entries) = std::fs::read_dir(&cache_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(meta) = path.metadata() { freed += meta.len(); }
                let _ = std::fs::remove_file(&path);
                deleted += 1;
            }
        }
    }

    if deleted > 0 {
        println!("Cleaning cache...");
        println!("  Deleted {} cached files.", deleted);
        println!("  Freed {:.1} MB of disk space.", freed as f64 / 1_048_576.0);
    } else { println!("Cache directory is empty."); }
    Ok(())
}

fn repair() -> Result<()> {
    println!("Running repair...");
    let mut fixed = 0u64;

    // 1. Ensure directories exist
    let _ = crate::config::buffy_home::ensure_directories();
    println!("  Directories OK.");

    // 2. Check and repair config file
    match crate::config::settings::read_config() {
        Ok(_) => println!("  Config file OK."),
        Err(_) => {
            crate::config::settings::write_config(&crate::config::settings::Config::default())?;
            println!("  Config file regenerated with defaults.");
            fixed += 1;
        }
    }

    // 3. Check and repair repositories file
    let known_stale_urls = ["https://github.com/BuffyCLI/packages"];
    match crate::config::settings::read_repositories() {
        Ok(repos) => {
            let has_stale = repos.iter().any(|r| known_stale_urls.contains(&r.as_str()));
            if has_stale {
                let updated: Vec<String> = repos.iter().map(|r| {
                    if known_stale_urls.contains(&r.as_str()) {
                        "https://github.com/Blaze12345-deluxe/Buffy-Plugins".to_string()
                    } else {
                        r.clone()
                    }
                }).collect();
                crate::config::settings::write_repositories(&updated)?;
                println!("  Updated stale repository URLs to current defaults.");
                fixed += 1;
            } else {
                println!("  Repositories file OK.");
            }
        }
        Err(_) => {
            let default_repos = vec!["https://github.com/Blaze12345-deluxe/Buffy-Plugins".to_string()];
            crate::config::settings::write_repositories(&default_repos)?;
            println!("  Repositories file regenerated with defaults.");
            fixed += 1;
        }
    }

    // 4. Check and repair aliases file
    match crate::config::aliases::read_aliases() {
        Ok(_) => println!("  Aliases file OK."),
        Err(_) => {
            // Corrupt aliases file — reset to empty
            use crate::config::aliases::Aliases;
            let empty = Aliases::default();
            if crate::config::aliases::write_aliases(&empty).is_ok() {
                println!("  Aliases file reset (was corrupt).");
                fixed += 1;
            }
        }
    }

    // 5. Check installed packages — remove orphaned registry entries
    let installed = crate::config::settings::read_installed()?;
    let commands_dir = crate::config::buffy_home::commands_dir();
    let mut cleaned_installed = installed.clone();
    let mut has_orphans = false;

    cleaned_installed.retain(|entry| {
        let exists = commands_dir.join(&entry.name).exists();
        if !exists {
            println!("  Package '{}' not found on disk. Removing from registry.", entry.name);
            has_orphans = true;
        }
        exists
    });

    if has_orphans {
        let removed = installed.len() as u64 - cleaned_installed.len() as u64;
        crate::config::settings::write_installed(&cleaned_installed)?;
        fixed += removed;
    }

    // 6. Check for orphaned directories (no registry entry)
    if commands_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&commands_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if entry.path().is_dir() && !cleaned_installed.iter().any(|e| e.name == name) {
                    println!("  Orphaned directory '{}' (no registry entry). Removing.", name);
                    let _ = std::fs::remove_dir_all(entry.path());
                    fixed += 1;
                }
            }
        }
    }

    // 7. Verify SHA integrity for installed packages (warn only)
    let mut integrity_issues = 0;
    for entry in &cleaned_installed {
        let pkg_dir = commands_dir.join(&entry.name);
        if pkg_dir.exists() {
            if let Err(e) = crate::package::verify::verify_package(&pkg_dir, &entry.name) {
                println!("  WARNING: Package '{}' integrity check failed: {}", entry.name, e);
                integrity_issues += 1;
            }
        }
    }

    if fixed == 0 && integrity_issues == 0 {
        println!("  No issues found.");
    } else {
        if fixed > 0 {
            println!("  Fixed {} issue(s).", fixed);
        }
        if integrity_issues > 0 {
            println!("  {} package(s) have integrity warnings (reinstall recommended).", integrity_issues);
        }
    }

    Ok(())
}

fn reset() -> Result<()> {
    println!("Resetting Buffy configuration...");
    let buffy_home = crate::config::buffy_home::buffy_home();
    let cache_dir = buffy_home.join("cache");
    if cache_dir.exists() { let _ = std::fs::remove_dir_all(&cache_dir); }
    crate::config::settings::write_config(&crate::config::settings::Config::default())?;
    crate::config::buffy_home::ensure_directories()?;
    println!("Done. Configuration reset to defaults.\nInstalled packages and repositories are preserved.");
    Ok(())
}

fn discover() -> Result<()> {
    crate::package::discover::discover()
}
fn install_packages(packages: Vec<String>) -> Result<()> {
    crate::package::install::install(&packages)
}

fn uninstall_package(name: String) -> Result<()> {
    crate::package::uninstall::uninstall(&name)
}

fn update_all_packages() -> Result<()> {
    crate::package::update::update_all()
}

fn update_one_package(name: String) -> Result<()> {
    crate::package::update::update_one(&name)
}

fn show_info(name: String) -> Result<()> {
    let installed = crate::config::settings::read_installed()?;
    let entry = installed.iter().find(|e| e.name == name);

    match entry {
        Some(e) => {
            println!("Package: {}", e.name);
            println!("  Version:     {}", e.version);
            println!("  Author:      {}", e.author);
            println!("  Source:      {}", e.source);
            println!("  Installed:   {}", e.installed);
            println!("  SHA-256:     {}", &e.sha256[..16.min(e.sha256.len())]);
            if !e.dependencies.system.is_empty() { println!("  System deps: {}", e.dependencies.system.join(", ")); }
            if !e.dependencies.packages.is_empty() { println!("  BSL deps:    {}", e.dependencies.packages.join(", ")); }
        }
        None => { eprintln!("Package '{}' is not installed.", name); }
    }
    Ok(())
}

fn show_outdated() -> Result<()> {
    let installed = crate::config::settings::read_installed()?;
    let repos = crate::config::settings::read_repositories()?;
    println!("Checking for updates...");
    let mut found = false;

    for entry in &installed {
        if entry.source == "local" { continue; }

        for repo_url in &repos {
            let full = if repo_url.starts_with("https://") { repo_url.clone() }
                       else { format!("https://github.com/{}", repo_url) };

            if let Some((owner, repo)) = crate::repository::source::parse_github_url(&full) {
                if let Ok(index) = crate::repository::github::fetch_index(&owner, &repo) {
                    if let Some(pkg) = index.find_package(&entry.name) {
                        use crate::repository::index::compare_versions;
                        if compare_versions(&pkg.version, &entry.version) == std::cmp::Ordering::Greater {
                            found = true;
                            println!("  {}  {}  ->  {}  ({})", entry.name, entry.version, pkg.version, entry.source);
                        }
                    }
                }
            }
        }
    }

    if !found { println!("All packages up to date."); }
    else { println!("\nRun 'buffy --update-packages' to update all."); }
    Ok(())
}

fn verify_package(name: String) -> Result<()> {
    let pkg_dir = crate::config::buffy_home::commands_dir().join(&name);
    if !pkg_dir.exists() { return Err(crate::error::BuffyError::PackageNotFound { name }); }

    match crate::package::verify::verify_package(&pkg_dir, &name) {
        Ok(()) => { println!("Package '{}' verified successfully.", name); Ok(()) }
        Err(e) => { eprintln!("Verification failed: {}", e); Err(e) }
    }
}

fn manage_repositories(args: Vec<String>) -> Result<()> {
    let cmd = args.first().map(|s| s.as_str()).unwrap_or("list");

    match cmd {
        "list" => {
            let repos = crate::config::settings::read_repositories()?;
            if repos.is_empty() { println!("No repositories configured."); }
            else { println!("Configured repositories:"); for (i, r) in repos.iter().enumerate() { println!("  {}. {}", i + 1, r); } }
        }
        "add" => {
            if args.len() < 2 { eprintln!("Usage: buffy --repo add <url>"); }
            else {
                let repos = crate::config::settings::read_repositories()?;
                if repos.contains(&args[1]) { println!("Repository already configured: {}", args[1]); return Ok(()); }

                // Validate the repository before adding
                println!("Validating repository...");
                match crate::repository::validate_repository(&args[1]) {
                    Ok(()) => {
                        let mut repos = repos;
                        repos.push(args[1].clone());
                        crate::config::settings::write_repositories(&repos)?;
                        println!("Added repository: {}", args[1]);
                    }
                    Err(e) => {
                        eprintln!("Could not add repository: {}", e);
                        eprintln!("Make sure the URL is correct and the repository has a valid index.json.");
                    }
                }
            }
        }
        "remove" => {
            if args.len() < 2 { eprintln!("Usage: buffy --repo remove <url>"); }
            else {
                let mut repos = crate::config::settings::read_repositories()?;
                let len_before = repos.len();
                repos.retain(|r| r != &args[1]);
                if repos.len() < len_before { crate::config::settings::write_repositories(&repos)?; println!("Removed repository: {}", args[1]); }
                else { eprintln!("Repository not found: {}", args[1]); }
            }
        }
        "refresh" => {
            let repos = crate::config::settings::read_repositories()?;
            if repos.is_empty() { println!("No repositories to refresh."); return Ok(()); }
            println!("Refreshing repository indexes...");
            for repo_url in &repos {
                let full = if repo_url.starts_with("https://") { repo_url.clone() }
                           else { format!("https://github.com/{}", repo_url) };
                if let Some((owner, repo)) = crate::repository::source::parse_github_url(&full) {
                    match crate::repository::refresh_index(&owner, &repo) {
                        Ok(index) => println!("  {}  ({} packages)", full, index.packages.len()),
                        Err(e) => eprintln!("  {}  error: {}", full, e),
                    }
                }
            }
            println!("Done.");
        }
        "search" => {
            if args.len() < 2 { eprintln!("Usage: buffy --repo search <query>"); }
            else {
                let query = args[1..].join(" ");
                println!("Searching for \"{}\"...", query);
                match crate::repository::search_across_repositories(&query) {
                    Ok(results) => {
                        if results.is_empty() {
                            println!("No packages found matching \"{}\".", query);
                        } else {
                            println!("Found {} package(s):\n", results.len());
                            for (repo_url, pkg) in &results {
                                let short_repo = repo_url.trim_start_matches("https://");
                                println!("  {}", pkg.name);
                                println!("    Description: {}", pkg.description);
                                println!("    Version:     {}", pkg.version);
                                println!("    Author:      {}", pkg.author);
                                println!("    Repository:  {}", short_repo);
                                println!();
                            }
                            println!("Install with: buffy --install <package-name>");
                        }
                    }
                    Err(e) => eprintln!("Search failed: {}", e),
                }
            }
        }
        _ => { eprintln!("Unknown repo command: {}\nUsage: buffy --repo [list|add|remove|refresh|search]", args[0]); }
    }
    Ok(())
}

fn manage_aliases(args: Vec<String>) -> Result<()> {
    let cmd = args.first().map(|s| s.as_str()).unwrap_or("list");

    match cmd {
        "list" => {
            let aliases = crate::config::aliases::list_aliases()?;
            let conflicts = crate::config::aliases::list_conflicts()?;

            if aliases.is_empty() && conflicts.is_empty() {
                println!("No aliases configured.");
                return Ok(());
            }

            if !aliases.is_empty() {
                println!("Aliases:");
                for (name, target) in &aliases {
                    println!("  {}  ->  {}", name, target);
                }
            }

            if !conflicts.is_empty() {
                if !aliases.is_empty() { println!(); }
                println!("Conflict Preferences:");
                for (name, sha, source) in &conflicts {
                    let short_sha = if sha.len() > 12 { &sha[..12] } else { sha.as_str() };
                    println!("  {}  ->  {}...  ({})", name, short_sha, source);
                }
            }
        }
        "set" => {
            if args.len() < 3 {
                eprintln!("Usage: buffy --alias set <name> <target>");
            } else {
                crate::config::aliases::set_alias(&args[1], &args[2])?;
                println!("Alias set: {} -> {}", args[1], args[2]);
            }
        }
        "remove" => {
            if args.len() < 2 {
                eprintln!("Usage: buffy --alias remove <name>");
            } else {
                // Remove both alias and conflict preference if they exist
                let had_alias = crate::config::aliases::resolve(&args[1])
                    .map(|r| r != args[1])
                    .unwrap_or(false);
                crate::config::aliases::remove_alias(&args[1])?;
                if had_alias {
                    println!("Removed alias: {}", args[1]);
                } else {
                    println!("Removed entry: {}", args[1]);
                }
            }
        }
        "resolve" => {
            if args.len() < 2 {
                eprintln!("Usage: buffy --alias resolve <package-name>");
            } else {
                match crate::config::aliases::resolve_conflict(&args[1])? {
                    Some((sha, source)) => {
                        let short_sha = if sha.len() > 16 { &sha[..16] } else { sha.as_str() };
                        println!("Package '{}' resolves to:", args[1]);
                        println!("  Preferred SHA: {}...", short_sha);
                        println!("  Source:        {}", source);
                    }
                    None => {
                        println!("No conflict preference set for '{}'.", args[1]);
                    }
                }
            }
        }
        _ => {
            eprintln!("Unknown alias command: {}\nUsage: buffy --alias [list|set|remove|resolve]", args[0]);
        }
    }
    Ok(())
}

fn manage_logs(log_arg: Option<String>) -> Result<()> {
    let logs_dir = crate::config::buffy_home::logs_dir();

    match log_arg.as_deref() {
        Some("clear") => {
            if logs_dir.exists() {
                let count = std::fs::read_dir(&logs_dir).map(|e| e.flatten().count()).unwrap_or(0);
                let _ = std::fs::remove_dir_all(&logs_dir);
                let _ = std::fs::create_dir_all(&logs_dir);
                println!("Deleted {} log file(s).", count);
            } else { println!("No logs to clear."); }
        }
        Some(_) => { eprintln!("Unknown logs subcommand. Use: buffy --logs [clear]"); }
        None => {
            if !logs_dir.exists() { println!("No logs found."); return Ok(()); }
            let mut log_files: Vec<_> = std::fs::read_dir(&logs_dir)
                .map(|e| e.flatten().filter(|e| e.path().is_file()).collect()).unwrap_or_default();
            log_files.sort_by_key(|e| e.file_name());
            if let Some(latest) = log_files.last() {
                let content = std::fs::read_to_string(latest.path())?;
                if content.is_empty() { println!("Latest log is empty: {}", latest.file_name().to_string_lossy()); }
                else { println!("--- {} ---\n{}", latest.file_name().to_string_lossy(), content); }
            } else { println!("No logs found."); }
        }
    }
    Ok(())
}

fn check_syntax(path: String) -> Result<()> {
    let source = std::fs::read_to_string(&path)?;
    let tokens = crate::bsl::lexer::tokenize(&source)
        .map_err(|e| crate::error::BuffyError::BslSyntax { path: path.clone(), line: 0, message: e.to_string() })?;
    match crate::bsl::parser::parse(tokens) {
        Ok(script) => { println!("Syntax OK ({} statements)", script.statements.len()); Ok(()) }
        Err(e) => Err(crate::error::BuffyError::BslSyntax { path: path.clone(), line: 0, message: e.to_string() }),
    }
}

fn validate_metadata(path: String) -> Result<()> {
    let source = std::fs::read_to_string(&path)?;
    let report = crate::diagnostic::validate::validate_script(&source)
        .map_err(|e| crate::error::BuffyError::BslSyntax { path: path.clone(), line: 0, message: e.to_string() })?;
    if report.warnings.is_empty() { println!("Metadata valid"); }
    else { for w in &report.warnings { println!("  {}", w); } }
    Ok(())
}

fn run_script(path: String, script_args: &[String]) -> Result<()> {
    let source = std::fs::read_to_string(&path)?;
    let tokens = crate::bsl::lexer::tokenize(&source)
        .map_err(|e| crate::error::BuffyError::BslSyntax { path: path.clone(), line: 0, message: e.to_string() })?;
    let script = crate::bsl::parser::parse(tokens)
        .map_err(|e| crate::error::BuffyError::BslSyntax { path: path.clone(), line: 0, message: e.to_string() })?;
    crate::bsl::interpreter::interpret(&script, script_args)
        .map_err(|e| crate::error::BuffyError::BslRuntime { command: String::new(), exit_code: 1, stderr: e.to_string() })?;
    Ok(())
}

fn benchmark_script(path: String) -> Result<()> {
    let source = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            return Err(crate::error::BuffyError::Io(e));
        }
    };

    // Parse the script first to ensure it's valid
    let tokens = match crate::bsl::lexer::tokenize(&source) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Syntax error: {}", e);
            return Ok(());
        }
    };
    let script = match crate::bsl::parser::parse(tokens) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Parse error: {}", e);
            return Ok(());
        }
    };

    let statement_count = script.statements.len();
    println!("Benchmarking \"{}\"...", path);
    println!("  Statements: {}", statement_count);
    println!();

    let runs = 3;
    let mut times: Vec<std::time::Duration> = Vec::new();

    // Warm-up run
    let _ = crate::bsl::interpreter::interpret(&script, &[]);

    // Timed runs
    for i in 1..=runs {
        let start = std::time::Instant::now();
        let result = crate::bsl::interpreter::interpret(&script, &[]);
        let elapsed = start.elapsed();

        match result {
            Ok(()) => {
                println!("  Run {}: {:.2?}", i, elapsed);
                times.push(elapsed);
            }
            Err(e) => {
                eprintln!("  Run {} failed: {}", i, e);
            }
        }
    }

    if !times.is_empty() {
        println!();
        let total: std::time::Duration = times.iter().sum();
        let avg = total / times.len() as u32;
        println!("  Average: {:.2?}", avg);
        println!("  Total:   {:.2?}", total);
    }

    Ok(())
}

fn generate_completion(shell: String) -> Result<()> {
    use clap::CommandFactory;

    let mut cmd = crate::cli::args::CliArgs::command();
    let shell_variant = match shell.as_str() {
        "bash" => clap_complete::Shell::Bash,
        "zsh" => clap_complete::Shell::Zsh,
        "fish" => clap_complete::Shell::Fish,
        "elvish" => clap_complete::Shell::Elvish,
        "powershell" => clap_complete::Shell::PowerShell,
        _ => {
            eprintln!("Unknown shell: {}", shell);
            eprintln!("Supported shells: bash, zsh, fish, elvish, powershell");
            return Ok(());
        }
    };

    clap_complete::generate(shell_variant, &mut cmd, "buffy", &mut std::io::stdout());
    Ok(())
}

fn execute_bsl_command(command: Vec<String>) -> Result<()> {
    let path = crate::resolver::tree::resolve(&command)?;
    let source = std::fs::read_to_string(&path)?;
    let tokens = crate::bsl::lexer::tokenize(&source)
        .map_err(|e| crate::error::BuffyError::BslSyntax { path: path.to_string_lossy().to_string(), line: 0, message: e.to_string() })?;
    let script = crate::bsl::parser::parse(tokens)
        .map_err(|e| crate::error::BuffyError::BslSyntax { path: path.to_string_lossy().to_string(), line: 0, message: e.to_string() })?;
    let script_args: Vec<String> = command[1..].to_vec();
    crate::bsl::interpreter::interpret(&script, &script_args)
        .map_err(|e| crate::error::BuffyError::BslRuntime { command: command.join(" "), exit_code: 1, stderr: e.to_string() })?;
    Ok(())
}
