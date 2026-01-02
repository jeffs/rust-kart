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

USAGE:
    save-context [OPTIONS] <COMMAND>

OPTIONS:
    -p, --project <PATH>    Project directory (default: current directory)
    -q, --quiet             Suppress non-essential output

COMMANDS:
    status                  Show current context configuration summary
    skill                   Manage project skills
    mcp                     Manage MCP servers

SKILL SUBCOMMANDS:
    save-context skill list                   List all skills with enabled/disabled status
    save-context skill disable <NAME>...      Disable one or more skills
    save-context skill enable <NAME>...       Re-enable one or more skills
    save-context skill disable-all            Disable all project skills
    save-context skill enable-all             Re-enable all disabled skills

MCP SUBCOMMANDS:
    save-context mcp list                     List all MCP servers with enabled/disabled status
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
}

// --- MCP server info ---

#[derive(Debug)]
struct McpServer {
    name: String,
    enabled: bool,
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

// --- Project detection ---

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

async fn list_skills(claude_dir: &Path) -> Result<Vec<Skill>> {
    let commands_dir = claude_dir.join("commands");
    let disabled_dir = claude_dir.join("commands.disabled");

    let mut skills = Vec::new();

    // Read enabled skills
    if commands_dir.is_dir() {
        let mut entries = fs::read_dir(&commands_dir)
            .await
            .map_err(|e| Error::Io(commands_dir.clone(), e))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| Error::Io(commands_dir.clone(), e))?
        {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "md") {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                let content = fs::read_to_string(&path)
                    .await
                    .map_err(|e| Error::Io(path.clone(), e))?;
                let tokens = estimate_tokens(&content);
                skills.push(Skill {
                    name,
                    enabled: true,
                    tokens,
                });
            }
        }
    }

    // Read disabled skills
    if disabled_dir.is_dir() {
        let mut entries = fs::read_dir(&disabled_dir)
            .await
            .map_err(|e| Error::Io(disabled_dir.clone(), e))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| Error::Io(disabled_dir.clone(), e))?
        {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "md") {
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                let content = fs::read_to_string(&path)
                    .await
                    .map_err(|e| Error::Io(path.clone(), e))?;
                let tokens = estimate_tokens(&content);
                skills.push(Skill {
                    name,
                    enabled: false,
                    tokens,
                });
            }
        }
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}

async fn disable_skill(claude_dir: &Path, name: &str) -> Result<()> {
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

async fn enable_skill(claude_dir: &Path, name: &str) -> Result<()> {
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

async fn list_mcp_servers(claude_dir: &Path) -> Result<Vec<McpServer>> {
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
            McpServer { name, enabled }
        })
        .collect();

    servers.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(servers)
}

async fn disable_mcp_server(claude_dir: &Path, name: &str) -> Result<()> {
    let settings_path = claude_dir.join("settings.local.json");
    let mut settings = read_settings(&settings_path).await?.unwrap_or_default();

    // Verify the server exists
    let servers = list_mcp_servers(claude_dir).await?;
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

async fn enable_mcp_server(claude_dir: &Path, name: &str) -> Result<()> {
    let settings_path = claude_dir.join("settings.local.json");
    let mut settings = read_settings(&settings_path).await?.unwrap_or_default();

    // Verify the server exists
    let servers = list_mcp_servers(claude_dir).await?;
    if !servers.iter().any(|s| s.name == name) {
        return Err(Error::McpNotFound(name.to_string()));
    }

    settings.disabled_mcpjson_servers.retain(|s| s != name);

    write_settings(&settings_path, &settings).await?;
    Ok(())
}

// --- Command handlers ---

async fn cmd_status(claude_dir: &Path, quiet: bool) -> Result<()> {
    let project_root = claude_dir.parent().unwrap_or(claude_dir);

    if !quiet {
        println!("Project: {}", project_root.display());
        println!();
    }

    // Skills summary
    let skills = list_skills(claude_dir).await?;
    let enabled_skills: Vec<_> = skills.iter().filter(|s| s.enabled).collect();
    let disabled_skills: Vec<_> = skills.iter().filter(|s| !s.enabled).collect();
    let enabled_tokens: usize = enabled_skills.iter().map(|s| s.tokens).sum();

    println!(
        "Skills: {} enabled (~{} tokens)",
        enabled_skills.len(),
        format_tokens(enabled_tokens)
    );

    if !quiet {
        for skill in &enabled_skills {
            println!("  {} {} tokens", skill.name, format_tokens(skill.tokens));
        }
        if !disabled_skills.is_empty() {
            println!();
            println!("Disabled skills: {}", disabled_skills.len());
            for skill in &disabled_skills {
                println!("  {} (disabled)", skill.name);
            }
        }
    }

    println!();

    // MCP summary
    let servers = list_mcp_servers(claude_dir).await?;
    let enabled_servers: Vec<_> = servers.iter().filter(|s| s.enabled).collect();
    let disabled_servers: Vec<_> = servers.iter().filter(|s| !s.enabled).collect();

    println!("MCP servers: {} enabled", enabled_servers.len());
    if !quiet {
        for server in &enabled_servers {
            println!("  {}", server.name);
        }
        if !disabled_servers.is_empty() {
            println!();
            println!("Disabled MCP servers: {}", disabled_servers.len());
            for server in &disabled_servers {
                println!("  {} (disabled)", server.name);
            }
        }
    }

    Ok(())
}

async fn cmd_skill_list(claude_dir: &Path) -> Result<()> {
    let skills = list_skills(claude_dir).await?;

    if skills.is_empty() {
        println!("No skills found");
        return Ok(());
    }

    for skill in &skills {
        let status = if skill.enabled { "" } else { " (disabled)" };
        println!(
            "{}{} - {} tokens",
            skill.name,
            status,
            format_tokens(skill.tokens)
        );
    }

    Ok(())
}

async fn cmd_skill_disable(claude_dir: &Path, names: &[String]) -> Result<()> {
    for name in names {
        disable_skill(claude_dir, name).await?;
        println!("Disabled: {name}");
    }
    Ok(())
}

async fn cmd_skill_enable(claude_dir: &Path, names: &[String]) -> Result<()> {
    for name in names {
        enable_skill(claude_dir, name).await?;
        println!("Enabled: {name}");
    }
    Ok(())
}

async fn cmd_skill_disable_all(claude_dir: &Path) -> Result<()> {
    let skills = list_skills(claude_dir).await?;
    let enabled: Vec<_> = skills.iter().filter(|s| s.enabled).collect();

    if enabled.is_empty() {
        return Err(Error::NoSkillsToDisable);
    }

    let total_tokens: usize = enabled.iter().map(|s| s.tokens).sum();

    for skill in &enabled {
        disable_skill(claude_dir, &skill.name).await?;
    }

    println!("Disabled {} skills", enabled.len());
    println!("Estimated savings: ~{} tokens", format_tokens(total_tokens));
    Ok(())
}

async fn cmd_skill_enable_all(claude_dir: &Path) -> Result<()> {
    let skills = list_skills(claude_dir).await?;
    let disabled: Vec<_> = skills.iter().filter(|s| !s.enabled).collect();

    if disabled.is_empty() {
        return Err(Error::NoSkillsToEnable);
    }

    for skill in &disabled {
        enable_skill(claude_dir, &skill.name).await?;
    }

    println!("Enabled {} skills", disabled.len());
    Ok(())
}

async fn cmd_mcp_list(claude_dir: &Path) -> Result<()> {
    let servers = list_mcp_servers(claude_dir).await?;

    if servers.is_empty() {
        println!("No MCP servers found");
        return Ok(());
    }

    for server in &servers {
        let status = if server.enabled { "" } else { " (disabled)" };
        println!("{}{status}", server.name);
    }

    Ok(())
}

async fn cmd_mcp_disable(claude_dir: &Path, names: &[String]) -> Result<()> {
    for name in names {
        disable_mcp_server(claude_dir, name).await?;
        println!("Disabled: {name}");
    }
    Ok(())
}

async fn cmd_mcp_enable(claude_dir: &Path, names: &[String]) -> Result<()> {
    for name in names {
        enable_mcp_server(claude_dir, name).await?;
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
