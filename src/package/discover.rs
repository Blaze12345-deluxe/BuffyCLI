use crate::error::Result;

/// Scans the system for common development tools and searches repositories
/// for matching packages to suggest to the user.
pub fn discover() -> Result<()> {
    // Delegate to the dispatch-level discover which has the full implementation
    // This keeps the package module focused on package-level operations while
    // allowing the CLI to provide the full user-facing discover experience.
    println!("Running system discovery...");

    let tools_to_check = vec![
        ("python3", "Python 3", "Python development tools"),
        ("python", "Python", "Python development tools"),
        ("pip", "pip", "Python package management"),
        ("pip3", "pip3", "Python package management"),
        ("git", "Git", "Version control tools"),
        ("docker", "Docker", "Container management tools"),
        ("docker-compose", "Docker Compose", "Docker Compose tools"),
        ("node", "Node.js", "Node.js development tools"),
        ("npm", "npm", "Node.js package management"),
        ("yarn", "Yarn", "Yarn package management"),
        ("cargo", "Cargo", "Rust development tools"),
        ("rustc", "rustc", "Rust compiler tools"),
        ("go", "Go", "Go development tools"),
        ("java", "Java", "Java development tools"),
        ("mvn", "Maven", "Maven build tools"),
        ("gradle", "Gradle", "Gradle build tools"),
        ("npx", "npx", "Node.js package execution"),
        ("rails", "Rails", "Ruby on Rails tools"),
        ("bundle", "Bundler", "Ruby Bundler tools"),
        ("gem", "gem", "Ruby gem tools"),
        ("php", "PHP", "PHP development tools"),
        ("composer", "Composer", "PHP Composer tools"),
        ("dotnet", ".NET", ".NET development tools"),
        ("kubectl", "kubectl", "Kubernetes tools"),
        ("helm", "Helm", "Helm chart tools"),
        ("terraform", "Terraform", "Terraform infrastructure tools"),
        ("vagrant", "Vagrant", "Vagrant VM tools"),
        ("ansible", "Ansible", "Ansible automation tools"),
        ("make", "Make", "Make build tools"),
        ("cmake", "CMake", "CMake build tools"),
        ("psql", "PostgreSQL", "PostgreSQL database tools"),
        ("mysql", "MySQL", "MySQL database tools"),
        ("redis-cli", "Redis", "Redis database tools"),
        ("mongosh", "MongoDB", "MongoDB database tools"),
        ("sqlite3", "SQLite", "SQLite database tools"),
        ("htop", "htop", "System monitoring tools"),
        ("neofetch", "neofetch", "System information tools"),
        ("curl", "curl", "HTTP request tools"),
        ("wget", "wget", "Download tools"),
        ("jq", "jq", "JSON processing tools"),
        ("yq", "yq", "YAML processing tools"),
        ("ffmpeg", "ffmpeg", "Media processing tools"),
        ("imagemagick", "ImageMagick", "Image processing tools"),
        ("ssh", "SSH", "SSH tools"),
        ("tmux", "tmux", "Terminal multiplexer tools"),
        ("screen", "screen", "Terminal multiplexer tools"),
    ];

    let mut detected_tools: Vec<&str> = Vec::new();

    for (tool, _, _) in &tools_to_check {
        if check_path_for(tool) {
            detected_tools.push(tool);
        }
    }

    detected_tools.sort();
    detected_tools.dedup();

    if detected_tools.is_empty() {
        println!("  No known tools detected on your system.");
        println!("  Search repositories manually with: buffy --repo search <query>");
        return Ok(());
    }

    println!("  Detected tools:");
    for tool in &detected_tools {
        let display_name = tools_to_check.iter()
            .find(|&&(t, _, _)| t == *tool)
            .map(|&(_, d, _)| d)
            .unwrap_or(tool);
        println!("    ✔ {}", display_name);
    }

    // Search repos for matching packages
    println!("");
    println!("  Searching repositories for matching packages...");

    use crate::repository::search_across_repositories;
    let mut suggestions: Vec<(String, String, String)> = Vec::new();

    for tool in &detected_tools {
        match search_across_repositories(tool) {
            Ok(results) => {
                for (repo_url, pkg) in &results {
                    suggestions.push((repo_url.clone(), pkg.name.clone(), pkg.description.clone()));
                }
            }
            Err(_) => {}
        }
    }

    suggestions.sort_by(|a, b| a.1.cmp(&b.1));
    suggestions.dedup_by(|a, b| a.1 == b.1);

    if suggestions.is_empty() {
        println!("  No matching packages found in repositories.");
    } else {
        println!("");
        println!("  Suggested packages:");
        for (_, name, desc) in &suggestions {
            println!("    {}  -  {}", name, desc);
        }
        println!("");
        println!("  Install with: buffy --install <package-name>");
    }

    Ok(())
}

/// Checks if a command exists on $PATH.
fn check_path_for(tool: &str) -> bool {
    if let Ok(paths) = std::env::var("PATH") {
        for dir in paths.split(':') {
            let full_path = std::path::Path::new(dir).join(tool);
            if full_path.exists() {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_path_for_existing() {
        // "sh" should exist on any Unix system
        assert!(check_path_for("sh"));
    }

    #[test]
    fn test_check_path_for_nonexistent() {
        assert!(!check_path_for("nonexistent_tool_xyz_123"));
    }
}
