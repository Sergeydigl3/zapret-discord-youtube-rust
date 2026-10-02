//! Talking to the GitHub API: listing a repository's tags, and turning the
//! `latest` selector into a concrete release tag.

/// Ask the API for a JSON body, treating any failure as "no answer".
fn get_json(url: &str, agent: &str) -> serde_json::Value {
    ureq::get(url)
        .set("User-Agent", agent)
        .call()
        .ok()
        .and_then(|req| req.into_string().ok())
        .and_then(|body| serde_json::from_str(&body).ok())
        .unwrap_or_default()
}

pub fn fetch_repo_tags(repo: &str) -> Result<Vec<String>, String> {
    let url = format!("https://api.github.com/repos/{}/tags", repo);
    let body = ureq::get(&url)
        .set("User-Agent", "zapret-rust-tui")
        .call()
        .map_err(|e| format!("{}{}: {}", rust_i18n::t!("err_fetch_tags"), repo, e))?
        .into_string()
        .map_err(|e| format!("{}{}", rust_i18n::t!("err_read_tags"), e))?;
    let parsed: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| format!("{}{}", rust_i18n::t!("err_parse_tags"), e))?;

    Ok(parsed
        .as_array()
        .map(|tags| {
            tags.iter()
                .filter_map(|tag| tag.get("name")?.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default())
}

/// Resolve a version selector to a concrete release tag, falling back to the
/// pinned recommendation when the API cannot answer.
pub(crate) fn resolve_tag(version: &str) -> Result<String, String> {
    if version != "latest" {
        return Ok(version.to_string());
    }
    println!("{}", rust_i18n::t!("msg_fetch_rel"));
    let url = format!("https://api.github.com/repos/{}/releases/latest", crate::ZAPRET_REPO);
    let parsed = get_json(&url, "zapret-rust");
    Ok(parsed
        .get("tag_name")
        .and_then(|t| t.as_str())
        .unwrap_or(crate::ZAPRET_REC_VER)
        .to_string())
}
