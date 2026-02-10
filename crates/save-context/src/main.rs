//! `save-context` - Claude Code context manager
//!
//! A CLI tool to reduce Claude Code context overhead by enabling/disabling
//! skills and MCP servers per-project.

use std::{
    collections::HashSet,
    env, fmt,
    path::{Path, PathBuf},
    process::exit,
};

use serde::{Deserialize, Serialize};
use tokio::fs;

const USAGE: &str = "
save-context - Claude Code context manager

Manages both project-scope and user-scope items. Each item is tagged
[project] or [user] in list output. Scope is auto-detected by name.

USAGE:
    save-context [OPTIONS] <COMMAND>

OPTIONS:
    -p, --project <PATH>    Project directory (default: current directory)
    -q, --quiet             Suppress non-essential output

COMMANDS:
    status                  Show current context configuration summary
    skill                   Manage skills (project and user)
    mcp                     Manage MCP servers (project and user)

SKILL SUBCOMMANDS:
    save-context skill list                   List all skills with scope and status
    save-context skill disable <NAME>...      Disable one or more skills
    save-context skill enable <NAME>...       Re-enable one or more skills
    save-context skill disable-all            Disable all project skills
    save-context skill enable-all             Re-enable all disabled project skills

MCP SUBCOMMANDS:
    save-context mcp list                     List all MCP servers with scope and status
    save-context mcp disable <NAME>...        Disable one or more MCP servers
    save-context mcp enable <NAME>...         Re-enable one or more MCP servers";

// --- Error handling ---

#[derive(Debug)]
enum Error {
    NotClaudeProject,
    Io(PathBuf, std::io::Error),
    Json(PathBuf, serde_json::Error),
    SkillNotFound(String),
    McpNotFound(String),
    NoSkillsToDisable,
    NoSkillsToEnable,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotClaudeProject => {
                write!(f, "Not a Claude Code project (no .claude/ directory found)")
            }
            Self::Io(path, e) => write!(f, "{}: {e}", path.display()),
            Self::Json(path, e) => write!(f, "{}: {e}", path.display()),
            Self::SkillNotFound(name) => write!(f, "Skill not found: {name}"),
            Self::McpNotFound(name) => write!(f, "MCP server not found: {name}"),
            Self::NoSkillsToDisable => write!(f, "No enabled skills to disable"),
            Self::NoSkillsToEnable => write!(f, "No disabled skills to enable"),
        }
    }
}

type Result<T> = std::result::Result<T, Error>;

// --- Scope ---

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Scope {
    User,
    Project,
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::User => write!(f, "user"),
            Self::Project => write!(f, "project"),
        }
    }
}

// --- JSON structures for settings ---

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Settings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    mcp_servers: Option<serde_json::Map<String, serde_json::Value>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    disabled_mcpjson_servers: Vec<String>,
    #[serde(flatten)]
    other: serde_json::Map<String, serde_json::Value>,
}

// --- Skill info ---

#[derive(Debug)]
struct Skill {
    name: String,
    enabled: bool,
    tokens: usize,
    scope: Scope,
}

// --- MCP server info ---

#[derive(Debug)]
struct McpServer {
    name: String,
    enabled: bool,
    scope: Scope,
}

// --- Token estimation ---

fn estimate_tokens(content: &str) -> usize {
    let words = content.split_whitespace().count();
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    let tokens = (words as f64 * 1.3).ceil() as usize;
    tokens
}

#[allow(clippy::cast_precision_loss)]
fn format_tokens(tokens: usize) -> String {
    if tokens >= 1000 {
        format!("{:.1}k", tokens as f64 / 1000.0)
    } else {
        format!("{tokens}")
    }
}

fn strip_frontmatter(content: &str) -> &str {
    if let Some(rest) = content.strip_prefix("---")
        && let Some(end) = rest.find("\n---")
    {
        let after = &rest[end + 4..];
        return after.strip_prefix('\n').unwrap_or(after);
    }
    content
}

// --- Project detection ---

fn user_claude_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude"))
}

fn find_claude_dir(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        let claude_dir = current.join(".claude");
        if claude_dir.is_dir() {
            return Some(claude_dir);
        }
        if !current.pop() {
            return None;
        }
    }
}

// --- Skill operations ---

async fn list_project_skills(claude_dir: &Path) -> Result<Vec<Skill>> {
    let commands_dir = claude_dir.join("commands");
    let disabled_dir = claude_dir.join("commands.disabled");
    let mut skills =
        read_skill_dir(&commands_dir, true, Scope::Project).await?;
    skills.extend(
        read_skill_dir(&disabled_dir, false, Scope::Project).await?,
    );
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}

async fn disable_project_skill(claude_dir: &Path, name: &str) -> Result<()> {
    let src = claude_dir.join("commands").join(format!("{name}.md"));
    let dst_dir = claude_dir.join("commands.disabled");
    let dst = dst_dir.join(format!("{name}.md"));

    if !src.exists() {
        return Err(Error::SkillNotFound(name.to_string()));
    }

    // Create disabled dir if needed
    if !dst_dir.exists() {
        fs::create_dir_all(&dst_dir)
            .await
            .map_err(|e| Error::Io(dst_dir.clone(), e))?;
    }

    fs::rename(&src, &dst)
        .await
        .map_err(|e| Error::Io(src, e))?;
    Ok(())
}

async fn enable_project_skill(claude_dir: &Path, name: &str) -> Result<()> {
    let src = claude_dir.join("commands.disabled").join(format!("{name}.md"));
    let dst = claude_dir.join("commands").join(format!("{name}.md"));

    if !src.exists() {
        return Err(Error::SkillNotFound(name.to_string()));
    }

    fs::rename(&src, &dst)
        .await
        .map_err(|e| Error::Io(src, e))?;
    Ok(())
}

async fn read_skill_dir(
    dir: &Path,
    enabled: bool,
    scope: Scope,
) -> Result<Vec<Skill>> {
    let mut skills = Vec::new();
    if !dir.is_dir() {
        return Ok(skills);
    }
    let mut entries = fs::read_dir(dir)
        .await
        .map_err(|e| Error::Io(dir.to_path_buf(), e))?;
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| Error::Io(dir.to_path_buf(), e))?
    {
        let path = entry.path();
        // User skills: directory with SKILL.md inside
        let skill_file = if path.is_dir() {
            let f = path.join("SKILL.md");
            if f.exists() { Some(f) } else { None }
        } else if path.extension().is_some_and(|e| e == "md") {
            // Project skills: flat .md files
            Some(path.clone())
        } else {
            None
        };
        let Some(skill_file) = skill_file else {
            continue;
        };
        let name = match scope {
            Scope::User => path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string(),
            Scope::Project => path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string(),
        };
        let content = fs::read_to_string(&skill_file)
            .await
            .map_err(|e| Error::Io(skill_file.clone(), e))?;
        let body = strip_frontmatter(&content);
        let tokens = estimate_tokens(body);
        skills.push(Skill {
            name,
            enabled,
            tokens,
            scope,
        });
    }
    Ok(skills)
}

async fn list_user_skills() -> Result<Vec<Skill>> {
    let Some(claude_dir) = user_claude_dir() else {
        return Ok(Vec::new());
    };
    let enabled_dir = claude_dir.join("skills");
    let disabled_dir = claude_dir.join("skills.disabled");
    let mut skills = read_skill_dir(&enabled_dir, true, Scope::User).await?;
    skills.extend(
        read_skill_dir(&disabled_dir, false, Scope::User).await?,
    );
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}

async fn list_all_skills(claude_dir: &Path) -> Result<Vec<Skill>> {
    let mut skills = list_project_skills(claude_dir).await?;
    skills.extend(list_user_skills().await?);
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}

async fn disable_user_skill(name: &str) -> Result<()> {
    let Some(claude_dir) = user_claude_dir() else {
        return Err(Error::SkillNotFound(name.to_string()));
    };
    let src = claude_dir.join("skills").join(name);
    let dst_dir = claude_dir.join("skills.disabled");
    let dst = dst_dir.join(name);

    if !src.exists() {
        return Err(Error::SkillNotFound(name.to_string()));
    }
    if !dst_dir.exists() {
        fs::create_dir_all(&dst_dir)
            .await
            .map_err(|e| Error::Io(dst_dir.clone(), e))?;
    }
    fs::rename(&src, &dst)
        .await
        .map_err(|e| Error::Io(src, e))?;
    Ok(())
}

async fn enable_user_skill(name: &str) -> Result<()> {
    let Some(claude_dir) = user_claude_dir() else {
        return Err(Error::SkillNotFound(name.to_string()));
    };
    let src = claude_dir.join("skills.disabled").join(name);
    let dst = claude_dir.join("skills").join(name);

    if !src.exists() {
        return Err(Error::SkillNotFound(name.to_string()));
    }
    fs::rename(&src, &dst)
        .await
        .map_err(|e| Error::Io(src, e))?;
    Ok(())
}

// --- MCP operations ---

async fn read_settings(path: &Path) -> Result<Option<Settings>> {
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(path)
        .await
        .map_err(|e| Error::Io(path.to_path_buf(), e))?;
    let settings: Settings =
        serde_json::from_str(&content).map_err(|e| Error::Json(path.to_path_buf(), e))?;
    Ok(Some(settings))
}

async fn write_settings(path: &Path, settings: &Settings) -> Result<()> {
    let content =
        serde_json::to_string_pretty(settings).map_err(|e| Error::Json(path.to_path_buf(), e))?;
    fs::write(path, content)
        .await
        .map_err(|e| Error::Io(path.to_path_buf(), e))?;
    Ok(())
}

async fn list_project_mcp_servers(claude_dir: &Path) -> Result<Vec<McpServer>> {
    let project_root = claude_dir.parent().unwrap_or(claude_dir);

    // Collect all MCP server names from various sources
    let mut all_servers: HashSet<String> = HashSet::new();
    let mut disabled: HashSet<String> = HashSet::new();

    // Read .claude/settings.local.json
    let local_settings_path = claude_dir.join("settings.local.json");
    if let Some(settings) = read_settings(&local_settings_path).await? {
        if let Some(servers) = &settings.mcp_servers {
            all_servers.extend(servers.keys().cloned());
        }
        disabled.extend(settings.disabled_mcpjson_servers);
    }

    // Read .claude/settings.json
    let settings_path = claude_dir.join("settings.json");
    if let Some(settings) = read_settings(&settings_path).await? {
        if let Some(servers) = &settings.mcp_servers {
            all_servers.extend(servers.keys().cloned());
        }
        disabled.extend(settings.disabled_mcpjson_servers);
    }

    // Read .mcp.json at project root
    let mcp_json_path = project_root.join(".mcp.json");
    if mcp_json_path.exists() {
        let content = fs::read_to_string(&mcp_json_path)
            .await
            .map_err(|e| Error::Io(mcp_json_path.clone(), e))?;
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&content)
            && let Some(servers) = value.get("mcpServers").and_then(|v| v.as_object())
        {
            all_servers.extend(servers.keys().cloned());
        }
    }

    // Read ~/.claude.json (user-level config with per-project MCP servers)
    if let Some(home) = dirs::home_dir() {
        let user_config_path = home.join(".claude.json");
        if user_config_path.exists() {
            let content = fs::read_to_string(&user_config_path)
                .await
                .map_err(|e| Error::Io(user_config_path.clone(), e))?;
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) {
                // Look up the project by its root path
                let project_key = project_root.to_string_lossy();
                if let Some(project) = value
                    .get("projects")
                    .and_then(|p| p.get(project_key.as_ref()))
                {
                    if let Some(servers) = project.get("mcpServers").and_then(|v| v.as_object()) {
                        all_servers.extend(servers.keys().cloned());
                    }
                    // Also check for disabled servers in user config
                    if let Some(disabled_list) = project
                        .get("disabledMcpjsonServers")
                        .and_then(|v| v.as_array())
                    {
                        for item in disabled_list {
                            if let Some(name) = item.as_str() {
                                disabled.insert(name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    let mut servers: Vec<McpServer> = all_servers
        .into_iter()
        .map(|name| {
            let enabled = !disabled.contains(&name);
            McpServer {
                name,
                enabled,
                scope: Scope::Project,
            }
        })
        .collect();

    servers.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(servers)
}

async fn disable_project_mcp_server(claude_dir: &Path, name: &str) -> Result<()> {
    let settings_path = claude_dir.join("settings.local.json");
    let mut settings = read_settings(&settings_path).await?.unwrap_or_default();

    // Verify the server exists
    let servers = list_project_mcp_servers(claude_dir).await?;
    if !servers.iter().any(|s| s.name == name) {
        return Err(Error::McpNotFound(name.to_string()));
    }

    if !settings.disabled_mcpjson_servers.contains(&name.to_string()) {
        settings.disabled_mcpjson_servers.push(name.to_string());
        settings.disabled_mcpjson_servers.sort();
    }

    write_settings(&settings_path, &settings).await?;
    Ok(())
}

async fn enable_project_mcp_server(claude_dir: &Path, name: &str) -> Result<()> {
    let settings_path = claude_dir.join("settings.local.json");
    let mut settings = read_settings(&settings_path).await?.unwrap_or_default();

    // Verify the server exists
    let servers = list_project_mcp_servers(claude_dir).await?;
    if !servers.iter().any(|s| s.name == name) {
        return Err(Error::McpNotFound(name.to_string()));
    }

    settings.disabled_mcpjson_servers.retain(|s| s != name);

    write_settings(&settings_path, &settings).await?;
    Ok(())
}

async fn list_user_mcp_servers() -> Result<Vec<McpServer>> {
    let Some(home) = dirs::home_dir() else {
        return Ok(Vec::new());
    };
    let config_path = home.join(".claude.json");
    if !config_path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&config_path)
        .await
        .map_err(|e| Error::Io(config_path.clone(), e))?;
    let value: serde_json::Value =
        serde_json::from_str(&content)
            .map_err(|e| Error::Json(config_path.clone(), e))?;

    let mut all_servers: HashSet<String> = HashSet::new();
    let mut disabled: HashSet<String> = HashSet::new();

    if let Some(servers) =
        value.get("mcpServers").and_then(|v| v.as_object())
    {
        all_servers.extend(servers.keys().cloned());
    }
    if let Some(disabled_list) = value
        .get("disabledMcpjsonServers")
        .and_then(|v| v.as_array())
    {
        for item in disabled_list {
            if let Some(name) = item.as_str() {
                disabled.insert(name.to_string());
            }
        }
    }

    let mut servers: Vec<McpServer> = all_servers
        .into_iter()
        .map(|name| {
            let enabled = !disabled.contains(&name);
            McpServer {
                name,
                enabled,
                scope: Scope::User,
            }
        })
        .collect();
    servers.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(servers)
}

async fn list_all_mcp_servers(
    claude_dir: &Path,
) -> Result<Vec<McpServer>> {
    let mut servers = list_project_mcp_servers(claude_dir).await?;
    servers.extend(list_user_mcp_servers().await?);
    servers.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(servers)
}

async fn disable_user_mcp_server(name: &str) -> Result<()> {
    let Some(home) = dirs::home_dir() else {
        return Err(Error::McpNotFound(name.to_string()));
    };
    let config_path = home.join(".claude.json");

    let servers = list_user_mcp_servers().await?;
    if !servers.iter().any(|s| s.name == name) {
        return Err(Error::McpNotFound(name.to_string()));
    }

    let content = fs::read_to_string(&config_path)
        .await
        .map_err(|e| Error::Io(config_path.clone(), e))?;
    let mut value: serde_json::Value =
        serde_json::from_str(&content)
            .map_err(|e| Error::Json(config_path.clone(), e))?;

    let disabled = value
        .as_object_mut()
        .unwrap()
        .entry("disabledMcpjsonServers")
        .or_insert_with(|| serde_json::Value::Array(Vec::new()));
    if let Some(arr) = disabled.as_array_mut() {
        let name_val = serde_json::Value::String(name.to_string());
        if !arr.contains(&name_val) {
            arr.push(name_val);
            arr.sort_by(|a, b| a.as_str().cmp(&b.as_str()));
        }
    }

    let output = serde_json::to_string_pretty(&value)
        .map_err(|e| Error::Json(config_path.clone(), e))?;
    fs::write(&config_path, output)
        .await
        .map_err(|e| Error::Io(config_path, e))?;
    Ok(())
}

async fn enable_user_mcp_server(name: &str) -> Result<()> {
    let Some(home) = dirs::home_dir() else {
        return Err(Error::McpNotFound(name.to_string()));
    };
    let config_path = home.join(".claude.json");

    let servers = list_user_mcp_servers().await?;
    if !servers.iter().any(|s| s.name == name) {
        return Err(Error::McpNotFound(name.to_string()));
    }

    let content = fs::read_to_string(&config_path)
        .await
        .map_err(|e| Error::Io(config_path.clone(), e))?;
    let mut value: serde_json::Value =
        serde_json::from_str(&content)
            .map_err(|e| Error::Json(config_path.clone(), e))?;

    if let Some(arr) = value
        .get_mut("disabledMcpjsonServers")
        .and_then(|v| v.as_array_mut())
    {
        arr.retain(|v| v.as_str() != Some(name));
    }

    let output = serde_json::to_string_pretty(&value)
        .map_err(|e| Error::Json(config_path.clone(), e))?;
    fs::write(&config_path, output)
        .await
        .map_err(|e| Error::Io(config_path, e))?;
    Ok(())
}

// --- Scope resolution ---

fn resolve_skill_scope(
    name: &str,
    all_skills: &[Skill],
) -> Result<Scope> {
    let matches: Vec<_> = all_skills
        .iter()
        .filter(|s| s.name == name)
        .collect();
    match matches.len() {
        0 => Err(Error::SkillNotFound(name.to_string())),
        1 => Ok(matches[0].scope),
        _ => {
            let scopes: Vec<_> =
                matches.iter().map(|s| s.scope).collect();
            if scopes.iter().all(|s| *s == scopes[0]) {
                Ok(scopes[0])
            } else {
                eprintln!(
                    "Ambiguous skill '{name}' exists in both \
                     user and project scope"
                );
                exit(1)
            }
        }
    }
}

fn resolve_mcp_scope(
    name: &str,
    all_servers: &[McpServer],
) -> Result<Scope> {
    let matches: Vec<_> = all_servers
        .iter()
        .filter(|s| s.name == name)
        .collect();
    match matches.len() {
        0 => Err(Error::McpNotFound(name.to_string())),
        1 => Ok(matches[0].scope),
        _ => {
            let scopes: Vec<_> =
                matches.iter().map(|s| s.scope).collect();
            if scopes.iter().all(|s| *s == scopes[0]) {
                Ok(scopes[0])
            } else {
                eprintln!(
                    "Ambiguous MCP server '{name}' exists in both \
                     user and project scope"
                );
                exit(1)
            }
        }
    }
}

// --- Command handlers ---

fn print_scope_section(
    scope: Scope,
    enabled: &[&Skill],
    disabled: &[&Skill],
    quiet: bool,
) {
    let enabled_tokens: usize =
        enabled.iter().map(|s| s.tokens).sum();
    println!(
        "[{scope}] Skills: {} enabled (~{} tokens)",
        enabled.len(),
        format_tokens(enabled_tokens)
    );
    if !quiet {
        for skill in enabled {
            println!(
                "  {} {} tokens",
                skill.name,
                format_tokens(skill.tokens)
            );
        }
        if !disabled.is_empty() {
            println!(
                "[{scope}] Disabled skills: {}",
                disabled.len()
            );
            for skill in disabled {
                println!("  {} (disabled)", skill.name);
            }
        }
    }
}

fn print_mcp_scope_section(
    scope: Scope,
    enabled: &[&McpServer],
    disabled: &[&McpServer],
    quiet: bool,
) {
    println!("[{scope}] MCP servers: {} enabled", enabled.len());
    if !quiet {
        for server in enabled {
            println!("  {}", server.name);
        }
        if !disabled.is_empty() {
            println!(
                "[{scope}] Disabled MCP servers: {}",
                disabled.len()
            );
            for server in disabled {
                println!("  {} (disabled)", server.name);
            }
        }
    }
}

async fn cmd_status(claude_dir: &Path, quiet: bool) -> Result<()> {
    let project_root = claude_dir.parent().unwrap_or(claude_dir);

    if !quiet {
        println!("Project: {}", project_root.display());
        println!();
    }

    for (scope, skills) in [
        (Scope::Project, list_project_skills(claude_dir).await?),
        (Scope::User, list_user_skills().await?),
    ] {
        if scope == Scope::User && skills.is_empty() {
            continue;
        }
        let enabled: Vec<_> =
            skills.iter().filter(|s| s.enabled).collect();
        let disabled: Vec<_> =
            skills.iter().filter(|s| !s.enabled).collect();
        print_scope_section(scope, &enabled, &disabled, quiet);
    }

    println!();

    for (scope, servers) in [
        (
            Scope::Project,
            list_project_mcp_servers(claude_dir).await?,
        ),
        (Scope::User, list_user_mcp_servers().await?),
    ] {
        if scope == Scope::User && servers.is_empty() {
            continue;
        }
        let enabled: Vec<_> =
            servers.iter().filter(|s| s.enabled).collect();
        let disabled: Vec<_> =
            servers.iter().filter(|s| !s.enabled).collect();
        print_mcp_scope_section(
            scope, &enabled, &disabled, quiet,
        );
    }

    Ok(())
}

async fn cmd_skill_list(claude_dir: &Path) -> Result<()> {
    let skills = list_all_skills(claude_dir).await?;

    if skills.is_empty() {
        println!("No skills found");
        return Ok(());
    }

    for skill in &skills {
        let status = if skill.enabled { "" } else { " (disabled)" };
        println!(
            "[{}] {}{} - {} tokens",
            skill.scope,
            skill.name,
            status,
            format_tokens(skill.tokens)
        );
    }

    Ok(())
}

async fn cmd_skill_disable(
    claude_dir: &Path,
    names: &[String],
) -> Result<()> {
    let all = list_all_skills(claude_dir).await?;
    for name in names {
        match resolve_skill_scope(name, &all)? {
            Scope::Project => {
                disable_project_skill(claude_dir, name).await?;
            }
            Scope::User => disable_user_skill(name).await?,
        }
        println!("Disabled: {name}");
    }
    Ok(())
}

async fn cmd_skill_enable(
    claude_dir: &Path,
    names: &[String],
) -> Result<()> {
    let all = list_all_skills(claude_dir).await?;
    for name in names {
        match resolve_skill_scope(name, &all)? {
            Scope::Project => {
                enable_project_skill(claude_dir, name).await?;
            }
            Scope::User => enable_user_skill(name).await?,
        }
        println!("Enabled: {name}");
    }
    Ok(())
}

async fn cmd_skill_disable_all(claude_dir: &Path) -> Result<()> {
    let skills = list_project_skills(claude_dir).await?;
    let enabled: Vec<_> = skills.iter().filter(|s| s.enabled).collect();

    if enabled.is_empty() {
        return Err(Error::NoSkillsToDisable);
    }

    let total_tokens: usize = enabled.iter().map(|s| s.tokens).sum();

    for skill in &enabled {
        disable_project_skill(claude_dir, &skill.name).await?;
    }

    println!("Disabled {} skills", enabled.len());
    println!(
        "Estimated savings: ~{} tokens",
        format_tokens(total_tokens)
    );
    Ok(())
}

async fn cmd_skill_enable_all(claude_dir: &Path) -> Result<()> {
    let skills = list_project_skills(claude_dir).await?;
    let disabled: Vec<_> =
        skills.iter().filter(|s| !s.enabled).collect();

    if disabled.is_empty() {
        return Err(Error::NoSkillsToEnable);
    }

    for skill in &disabled {
        enable_project_skill(claude_dir, &skill.name).await?;
    }

    println!("Enabled {} skills", disabled.len());
    Ok(())
}

async fn cmd_mcp_list(claude_dir: &Path) -> Result<()> {
    let servers = list_all_mcp_servers(claude_dir).await?;

    if servers.is_empty() {
        println!("No MCP servers found");
        return Ok(());
    }

    for server in &servers {
        let status = if server.enabled { "" } else { " (disabled)" };
        println!("[{}] {}{status}", server.scope, server.name);
    }

    Ok(())
}

async fn cmd_mcp_disable(
    claude_dir: &Path,
    names: &[String],
) -> Result<()> {
    let all = list_all_mcp_servers(claude_dir).await?;
    for name in names {
        match resolve_mcp_scope(name, &all)? {
            Scope::Project => {
                disable_project_mcp_server(claude_dir, name).await?;
            }
            Scope::User => disable_user_mcp_server(name).await?,
        }
        println!("Disabled: {name}");
    }
    Ok(())
}

async fn cmd_mcp_enable(
    claude_dir: &Path,
    names: &[String],
) -> Result<()> {
    let all = list_all_mcp_servers(claude_dir).await?;
    for name in names {
        match resolve_mcp_scope(name, &all)? {
            Scope::Project => {
                enable_project_mcp_server(claude_dir, name).await?;
            }
            Scope::User => enable_user_mcp_server(name).await?,
        }
        println!("Enabled: {name}");
    }
    Ok(())
}

// --- Main ---

fn die(message: impl fmt::Display) -> ! {
    eprintln!("Error: {message}");
    exit(1)
}

fn usage() -> ! {
    eprintln!("{USAGE}");
    exit(2)
}

#[tokio::main]
async fn main() {
    let mut args: Vec<String> = env::args().skip(1).collect();

    // Parse global options
    let mut project_path: Option<PathBuf> = None;
    let mut quiet = false;

    while !args.is_empty() {
        match args[0].as_str() {
            "-p" | "--project" => {
                args.remove(0);
                if args.is_empty() {
                    usage();
                }
                project_path = Some(PathBuf::from(args.remove(0)));
            }
            "-q" | "--quiet" => {
                args.remove(0);
                quiet = true;
            }
            _ => break,
        }
    }

    // Find Claude directory
    let start_dir = project_path.unwrap_or_else(|| env::current_dir().unwrap_or_default());
    let Some(claude_dir) = find_claude_dir(&start_dir) else {
        die(Error::NotClaudeProject);
    };

    // Parse command
    if args.is_empty() {
        usage();
    }

    let command = args.remove(0);
    let result = match command.as_str() {
        "status" => cmd_status(&claude_dir, quiet).await,
        "skill" => {
            if args.is_empty() {
                usage();
            }
            let subcommand = args.remove(0);
            match subcommand.as_str() {
                "list" => cmd_skill_list(&claude_dir).await,
                "disable" | "enable" if args.is_empty() => usage(),
                "disable" => cmd_skill_disable(&claude_dir, &args).await,
                "enable" => cmd_skill_enable(&claude_dir, &args).await,
                "disable-all" => cmd_skill_disable_all(&claude_dir).await,
                "enable-all" => cmd_skill_enable_all(&claude_dir).await,
                _ => usage(),
            }
        }
        "mcp" => {
            if args.is_empty() {
                usage();
            }
            let subcommand = args.remove(0);
            match subcommand.as_str() {
                "list" => cmd_mcp_list(&claude_dir).await,
                "disable" | "enable" if args.is_empty() => usage(),
                "disable" => cmd_mcp_disable(&claude_dir, &args).await,
                "enable" => cmd_mcp_enable(&claude_dir, &args).await,
                _ => usage(),
            }
        }
        _ => usage(),
    };

    if let Err(e) = result {
        die(e);
    }
}
