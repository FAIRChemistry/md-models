use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, USER_AGENT};
use reqwest::StatusCode;
use reqwest::Url;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedGithubRepo {
    pub owner: String,
    pub name: String,
    pub reference: String,
    pub commit: String,
    pub root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GithubRepoSpec {
    owner: String,
    name: String,
    reference: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GithubRepoResponse {
    default_branch: String,
}

#[derive(Debug, Deserialize)]
struct GithubCommitResponse {
    sha: String,
}

pub fn cache_github_repo(repo: &str) -> Result<CachedGithubRepo, Box<dyn Error>> {
    let spec = parse_github_repo_spec(repo)?;
    let client = github_client()?;

    let reference = match spec.reference {
        Some(reference) => reference,
        None => fetch_default_branch(&client, &spec.owner, &spec.name)?,
    };

    let commit = fetch_commit_sha(&client, &spec.owner, &spec.name, &reference)?;
    let cache_root = github_cache_root();
    let target = cache_root.join(&spec.owner).join(&spec.name).join(&commit);

    if !target.exists() {
        fs::create_dir_all(target.parent().ok_or("Invalid cache path")?)?;
        clone_into_cache(&spec.owner, &spec.name, &reference, &target)?;
    }

    Ok(CachedGithubRepo {
        owner: spec.owner,
        name: spec.name,
        reference,
        commit,
        root: target,
    })
}

pub fn parse_github_file_url(url: &str) -> Option<(String, String)> {
    if let Some(rest) = url.strip_prefix("https://raw.githubusercontent.com/") {
        let mut parts = rest.splitn(4, '/');
        let owner = parts.next()?;
        let repo = parts.next()?;
        let reference = parts.next()?;
        let path = parts.next()?;
        return Some((format!("{owner}/{repo}@{reference}"), path.to_string()));
    }

    if let Some(rest) = url.strip_prefix("https://github.com/") {
        let parts: Vec<&str> = rest.split('/').collect();
        if parts.len() >= 5 && parts[2] == "blob" {
            let owner = parts[0];
            let repo = parts[1];
            let reference = parts[3];
            let path = parts[4..].join("/");
            return Some((format!("{owner}/{repo}@{reference}"), path));
        }
    }

    None
}

fn parse_github_repo_spec(input: &str) -> Result<GithubRepoSpec, Box<dyn Error>> {
    let trimmed = input.trim().strip_prefix("git+").unwrap_or(input.trim());

    let raw = if let Some(rest) = trimmed.strip_prefix("https://github.com/") {
        rest.trim_end_matches('/')
    } else if let Some(rest) = trimmed.strip_prefix("http://github.com/") {
        rest.trim_end_matches('/')
    } else {
        trimmed
    };

    let (repo_part, reference) = if let Some((repo_part, reference)) = raw.rsplit_once('@') {
        (
            repo_part.trim_end_matches(".git"),
            Some(reference.to_string()),
        )
    } else {
        (raw.trim_end_matches(".git"), None)
    };

    let mut parts = repo_part.split('/');
    let owner = parts.next().ok_or("Missing GitHub owner")?.to_string();
    let name = parts.next().ok_or("Missing GitHub repo name")?.to_string();

    if owner.is_empty() || name.is_empty() || parts.next().is_some() {
        return Err("Expected GitHub repo in the form owner/repo[@ref]".into());
    }

    Ok(GithubRepoSpec {
        owner,
        name,
        reference,
    })
}

fn github_client() -> Result<Client, Box<dyn Error>> {
    Ok(Client::builder().build()?)
}

fn fetch_default_branch(
    client: &Client,
    owner: &str,
    repo: &str,
) -> Result<String, Box<dyn Error>> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}");
    let response = client
        .get(&url)
        .header(USER_AGENT, "mdmodels")
        .header(ACCEPT, "application/vnd.github+json")
        .send()?;

    if !response.status().is_success() {
        return Err(github_http_error(
            response.status(),
            owner,
            repo,
            None,
            "lookup repository metadata",
        )
        .into());
    }

    let payload: GithubRepoResponse = response.json()?;
    Ok(payload.default_branch)
}

fn fetch_commit_sha(
    client: &Client,
    owner: &str,
    repo: &str,
    reference: &str,
) -> Result<String, Box<dyn Error>> {
    let mut url = Url::parse("https://api.github.com")?;
    url.path_segments_mut()
        .map_err(|_| "Failed to build GitHub commit URL")?
        .extend(["repos", owner, repo, "commits", reference]);

    let response = client
        .get(url)
        .header(USER_AGENT, "mdmodels")
        .header(ACCEPT, "application/vnd.github+json")
        .send()?;

    if !response.status().is_success() {
        return Err(github_http_error(
            response.status(),
            owner,
            repo,
            Some(reference),
            "resolve commit for ref",
        )
        .into());
    }

    let payload: GithubCommitResponse = response.json()?;
    Ok(payload.sha)
}

fn github_http_error(
    status: StatusCode,
    owner: &str,
    repo: &str,
    reference: Option<&str>,
    action: &str,
) -> String {
    let target = match reference {
        Some(reference) => format!("{owner}/{repo}@{reference}"),
        None => format!("{owner}/{repo}"),
    };

    match status {
        StatusCode::NOT_FOUND => match reference {
            Some(reference) => format!(
                "GitHub {action} failed: '{target}' was not found. Check the repository name and ref '{reference}'."
            ),
            None => format!(
                "GitHub {action} failed: repository '{target}' was not found. Check owner/repo spelling."
            ),
        },
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => format!(
            "GitHub {action} failed for '{target}' (HTTP {status}). Repository may be private or rate-limited."
        ),
        _ => format!("GitHub {action} failed for '{target}' (HTTP {status})."),
    }
}

fn clone_into_cache(
    owner: &str,
    repo: &str,
    reference: &str,
    target: &Path,
) -> Result<(), Box<dyn Error>> {
    let remote_url = format!("https://github.com/{owner}/{repo}.git");

    let temp_dir = target
        .parent()
        .ok_or("Invalid target path")?
        .join(format!(".tmp-{}", cache_suffix()));

    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir)?;
    }

    let mut prepare = gix::prepare_clone(remote_url, &temp_dir)?;
    prepare = prepare.with_ref_name(Some(reference))?;

    let (mut checkout, _fetch_outcome) =
        prepare.fetch_then_checkout(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)?;
    let (_repo, _checkout_outcome) =
        checkout.main_worktree(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)?;

    if target.exists() {
        fs::remove_dir_all(&temp_dir)?;
        return Ok(());
    }

    fs::rename(&temp_dir, target)?;
    Ok(())
}

fn github_cache_root() -> PathBuf {
    cache_base_dir().join("mdmodels").join("github")
}

fn cache_base_dir() -> PathBuf {
    dirs::cache_dir().unwrap_or_else(std::env::temp_dir)
}

fn cache_suffix() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{now}-{}", std::process::id())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_repo_without_ref() {
        let spec = parse_github_repo_spec("owner/repo").expect("spec should parse");
        assert_eq!(spec.owner, "owner");
        assert_eq!(spec.name, "repo");
        assert_eq!(spec.reference, None);
    }

    #[test]
    fn parse_repo_with_ref() {
        let spec = parse_github_repo_spec("owner/repo@main").expect("spec should parse");
        assert_eq!(spec.owner, "owner");
        assert_eq!(spec.name, "repo");
        assert_eq!(spec.reference.as_deref(), Some("main"));
    }

    #[test]
    fn parse_repo_with_git_plus_url() {
        let spec = parse_github_repo_spec("git+https://github.com/owner/repo.git@v1.2.3")
            .expect("spec should parse");
        assert_eq!(spec.owner, "owner");
        assert_eq!(spec.name, "repo");
        assert_eq!(spec.reference.as_deref(), Some("v1.2.3"));
    }

    #[test]
    fn parse_blob_url() {
        let parsed = parse_github_file_url("https://github.com/a/b/blob/main/models/root.md")
            .expect("url should parse");
        assert_eq!(parsed.0, "a/b@main");
        assert_eq!(parsed.1, "models/root.md");
    }

    #[test]
    fn parse_raw_url() {
        let parsed =
            parse_github_file_url("https://raw.githubusercontent.com/a/b/main/models/root.md")
                .expect("url should parse");
        assert_eq!(parsed.0, "a/b@main");
        assert_eq!(parsed.1, "models/root.md");
    }
}
