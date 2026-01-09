use serde::Deserialize;
use serde_json::{Map, Value};
use std::process::Command;

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
    comments: Vec<serde_json::Value>,
    review_decision: Option<String>,
    status_check_rollup: Vec<StatusCheck>,
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
    comments: usize,
    review: String,
    ci: String,
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
            "title,url,comments,reviewDecision,statusCheckRollup",
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("gh pr view failed for {repo}#{number}: {stderr}").into());
    }

    let details: PrDetails = serde_json::from_slice(&output.stdout)?;
    Ok(details)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let search_results = search_open_prs()?;

    let mut prs = Vec::new();
    for result in &search_results {
        let repo = &result.repository.name_with_owner;
        let details = fetch_pr_details(repo, result.number)?;

        prs.push(OutputPr {
            title: details.title,
            index: details.url,
            comments: details.comments.len(),
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
