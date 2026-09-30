//! Talking to the GitHub API.
//!
//! Two things need the network: listing the tags of a repository so the UI can
//! offer them, and turning the `latest` selector into a concrete release tag
//! before a download URL can be built.

pub fn fetch_repo_tags(repo: &str) -> Result<Vec<String>, String> {
    let url = format!("https://api.github.com/repos/{}/tags", repo);
    let req = ureq::get(&url)
        .set("User-Agent", "zapret-rust-tui")
        .call()
        .map_err(|e| format!("{}{}: {}", rust_i18n::t!("err_fetch_tags"), repo, e))?;

    let json_str = req
        .into_string()
        .map_err(|e| format!("{}{}", rust_i18n::t!("err_read_tags"), e))?;
    let tags_json: serde_json::Value =
        serde_json::from_str(&json_str).map_err(|e| format!("{}{}", rust_i18n::t!("err_parse_tags"), e))?;
    let mut tags = Vec::new();

    if let Some(arr) = tags_json.as_array() {
        for item in arr {
            if let Some(name) = item.get("name").and_then(|v| v.as_str()) {
                tags.push(name.to_string());
            }
        }
    }

    Ok(tags)
}

/// Resolve a version selector to a concrete release tag.
/// `latest` queries the GitHub releases API and falls back to `ZAPRET_REC_VER`.
pub(crate) fn resolve_tag(version: &str) -> Result<String, String> {
    if version == "latest" {
        println!("{}", rust_i18n::t!("msg_fetch_rel"));
        let latest_url = format!(
            "https://api.github.com/repos/{}/releases/latest",
            crate::download::ZAPRET_REPO
        );
        let req = ureq::get(&latest_url)
            .set("User-Agent", "zapret-rust")
            .call()
            .map_err(|e| format!("{}{}", rust_i18n::t!("err_fetch_rel"), e))?;

        let json_str = req.into_string().unwrap_or_else(|_| "{}".to_string());
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap_or_default();
        return Ok(parsed
            .get("tag_name")
            .and_then(|t| t.as_str())
            .unwrap_or(crate::download::ZAPRET_REC_VER)
            .to_string());
    }
    Ok(version.to_string())
}
