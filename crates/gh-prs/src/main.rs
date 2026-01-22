use chrono::{DateTime, Duration, Utc};
use clap::Parser;
use serde::Deserialize;
use serde_json::{Map, Value};
use std::process::Command;

#[derive(Parser)]
#[command(about = "List open GitHub PRs authored by the current user")]
struct Args {
    /// Only show PRs updated since this time (e.g., "7d", "2w", "24h", or "2024-01-15")
    #[arg(long)]
    since: Option<String>,
}

fn parse_duration(s: &str) -> Option<Duration> {
    let (num_str, unit) = s.split_at(s.len() - 1);
    let num: i64 = num_str.parse().ok()?;

    match unit {
        "d" => Some(Duration::days(num)),
        "w" => Some(Duration::weeks(num)),
        "h" => Some(Duration::hours(num)),
        "m" => Some(Duration::days(num * 30)),
        _ => None,
    }
}

fn parse_since(s: &str) -> Result<DateTime<Utc>, String> {
    // Try relative duration first: 7d, 2w, 24h, 3m (days, weeks, hours, months)
    if let Some(duration) = parse_duration(s) {
        return Ok(Utc::now() - duration);
    }

    // Try ISO date (YYYY-MM-DD)
    if let Ok(date) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Ok(date.and_hms_opt(0, 0, 0).unwrap().and_utc());
    }

    // Try full ISO datetime
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&Utc));
    }

    Err(format!(
        "Cannot parse '{s}' - expected duration (7d, 2w, 24h, 3m) or date (YYYY-MM-DD)"
    ))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SearchResult {
    number: u64,
    repository: Repository,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Repository {
    name_with_owner: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PrDetails {
    title: String,
    url: String,
    updated_at: DateTime<Utc>,
    comments: Vec<Comment>,
    reviews: Vec<Review>,
    review_decision: Option<String>,
    status_check_rollup: Vec<StatusCheck>,
}

#[derive(Debug, Deserialize)]
struct Comment {
    author: Option<Author>,
}

#[derive(Debug, Deserialize)]
struct Author {
    login: String,
}

#[derive(Debug, Deserialize)]
struct Review {
    author: Option<Author>,
    body: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StatusCheck {
    state: Option<String>,
    status: Option<String>,
    conclusion: Option<String>,
}

struct OutputPr {
    title: String,
    index: String,
    updated_at: DateTime<Utc>,
    comments: usize,
    review: String,
    ci: String,
}

fn is_bot(login: &str) -> bool {
    const BOT_NAMES: &[&str] = &[
        "github-actions",
        "dependabot",
        "renovate",
        "codecov",
        "netlify",
        "vercel",
        "semantic-release-bot",
        "snyk-bot",
        "imgbot",
        "allcontributors",
        "greenkeeper",
        "linear",
    ];

    let login_lower = login.to_lowercase();
    login_lower.ends_with("[bot]")
        || login_lower.ends_with("-bot")
        || BOT_NAMES.contains(&login_lower.as_str())
}

fn count_human_comments(comments: &[Comment]) -> usize {
    comments
        .iter()
        .filter(|c| {
            c.author
                .as_ref()
                .map(|a| !is_bot(&a.login))
                .unwrap_or(false)
        })
        .count()
}

fn count_review_body_comments(reviews: &[Review]) -> usize {
    reviews
        .iter()
        .filter(|r| {
            !r.body.trim().is_empty()
                && r.author
                    .as_ref()
                    .map(|a| !is_bot(&a.login))
                    .unwrap_or(false)
        })
        .count()
}

fn derive_review_emoji(review_decision: Option<&str>) -> String {
    match review_decision {
        Some("APPROVED") => "✅".to_string(),
        Some("CHANGES_REQUESTED") => "❌".to_string(),
        _ => String::new(),
    }
}

fn derive_ci_emoji(checks: &[StatusCheck]) -> String {
    if checks.is_empty() {
        return String::new();
    }

    let mut has_pending = false;

    for check in checks {
        // Check for actual failures first
        if let Some(state) = &check.state
            && matches!(state.as_str(), "FAILURE" | "ERROR")
        {
            return "❌".to_string();
        }
        if let Some(conclusion) = &check.conclusion
            && matches!(
                conclusion.as_str(),
                "FAILURE" | "TIMED_OUT" | "CANCELLED" | "ACTION_REQUIRED"
            )
        {
            return "❌".to_string();
        }

        // Track pending/in-progress checks
        if let Some(state) = &check.state
            && state == "PENDING"
        {
            has_pending = true;
        }
        if let Some(status) = &check.status
            && status != "COMPLETED"
        {
            has_pending = true;
        }
    }

    if has_pending {
        "⏳".to_string()
    } else {
        "✅".to_string()
    }
}

fn search_open_prs() -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
    let output = Command::new("gh")
        .args([
            "search",
            "prs",
            "--author=@me",
            "--state=open",
            "--archived=false",
            "--draft=false",
            "--json",
            "number,repository",
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("gh search failed: {stderr}").into());
    }

    let results: Vec<SearchResult> = serde_json::from_slice(&output.stdout)?;
    Ok(results)
}

fn fetch_pr_details(repo: &str, number: u64) -> Result<PrDetails, Box<dyn std::error::Error>> {
    let output = Command::new("gh")
        .args([
            "pr",
            "view",
            &number.to_string(),
            "--repo",
            repo,
            "--json",
            "title,url,updatedAt,comments,reviews,reviewDecision,statusCheckRollup",
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("gh pr view failed for {repo}#{number}: {stderr}").into());
    }

    let details: PrDetails = serde_json::from_slice(&output.stdout)?;
    Ok(details)
}

fn format_relative_time(dt: DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(dt);

    if duration.num_days() > 0 {
        format!("{}d ago", duration.num_days())
    } else if duration.num_hours() > 0 {
        format!("{}h ago", duration.num_hours())
    } else if duration.num_minutes() > 0 {
        format!("{}m ago", duration.num_minutes())
    } else {
        "just now".to_string()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let since = args
        .since
        .as_ref()
        .map(|s| parse_since(s))
        .transpose()
        .map_err(|e| format!("Invalid --since value: {e}"))?;

    let search_results = search_open_prs()?;

    let mut prs = Vec::new();
    for result in &search_results {
        let repo = &result.repository.name_with_owner;
        let details = fetch_pr_details(repo, result.number)?;

        // Filter by --since if specified
        if let Some(since_time) = since {
            if details.updated_at < since_time {
                continue;
            }
        }

        prs.push(OutputPr {
            title: details.title,
            index: details.url,
            updated_at: details.updated_at,
            comments: count_human_comments(&details.comments)
                + count_review_body_comments(&details.reviews),
            review: derive_review_emoji(details.review_decision.as_deref()),
            ci: derive_ci_emoji(&details.status_check_rollup),
        });
    }

    let has_review = prs.iter().any(|pr| !pr.review.is_empty());
    let has_ci = prs.iter().any(|pr| !pr.ci.is_empty());

    let output: Vec<Value> = prs
        .into_iter()
        .map(|pr| {
            let mut map = Map::new();
            map.insert("title".to_string(), Value::String(pr.title));
            map.insert("index".to_string(), Value::String(pr.index));
            map.insert(
                "updated".to_string(),
                Value::String(format_relative_time(pr.updated_at)),
            );
            map.insert("comments".to_string(), Value::Number(pr.comments.into()));
            if has_review {
                map.insert("review".to_string(), Value::String(pr.review));
            }
            if has_ci {
                map.insert("ci".to_string(), Value::String(pr.ci));
            }
            Value::Object(map)
        })
        .collect();

    let json = serde_json::to_string_pretty(&output)?;
    println!("{json}");

    Ok(())
}
