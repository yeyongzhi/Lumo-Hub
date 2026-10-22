use serde::Deserialize;
use serde_json::{json, Value};
use std::time::Duration;
use tauri::Manager;
use tauri_plugin_updater::UpdaterExt;

const RELEASES_FEED: &str = "https://github.com/yeyongzhi/Lumo-Hub/releases.atom";
const STABLE_MANIFEST: &str = "https://github.com/yeyongzhi/Lumo-Hub/releases/latest/download/latest.json";
const TAG_PREFIX: &str = "https://github.com/yeyongzhi/Lumo-Hub/releases/tag/";

#[derive(Deserialize)]
struct Feed {
    #[serde(rename = "entry", default)]
    entries: Vec<Entry>,
}
#[derive(Deserialize)]
struct Entry {
    #[serde(rename = "link", default)]
    links: Vec<Link>,
}
#[derive(Deserialize)]
struct Link {
    #[serde(rename = "@href")]
    href: String,
}

fn beta_manifest(feed: &str) -> Result<url::Url, String> {
    let feed: Feed = quick_xml::de::from_str(feed).map_err(|e| format!("发布订阅数据无效：{e}"))?;
    // GitHub lists published releases newest first, including prereleases.
    // Only accept this repository's semantic-version tags.
    for entry in feed.entries {
        for link in entry.links {
            if let Some(tag) = link.href.strip_prefix(TAG_PREFIX) {
                if semver::Version::parse(tag.strip_prefix('v').unwrap_or(tag)).is_ok() {
                    let mut endpoint = url::Url::parse("https://github.com/yeyongzhi/Lumo-Hub/releases/download/").unwrap();
                    endpoint.path_segments_mut().unwrap().pop_if_empty().push(tag).push("latest.json");
                    return Ok(endpoint);
                }
            }
        }
    }
    Err("发布源中尚未找到有效版本，请稍后重试".into())
}

async fn resolve_manifest(beta: bool) -> Result<url::Url, String> {
    if !beta { return url::Url::parse(STABLE_MANIFEST).map_err(|e| e.to_string()); }
    let client = reqwest::Client::builder()
        .user_agent(concat!("Lumo-Hub/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(20)).build().map_err(|e| e.to_string())?;
    let feed = client.get(RELEASES_FEED).send().await
        .map_err(|e| format!("无法连接发布源：{e}"))?
        .error_for_status().map_err(|e| format!("发布源请求失败：{e}"))?
        .text().await.map_err(|e| format!("无法读取发布订阅：{e}"))?;
    beta_manifest(&feed)
}

// Resolve prereleases without the rate-limited anonymous GitHub REST API.
// Tauri still validates the manifest and verifies the installer signature.
#[tauri::command]
pub async fn check_app_update(webview: tauri::Webview) -> Result<Option<Value>, String> {
    let endpoint = resolve_manifest(!webview.package_info().version.pre.is_empty()).await?;
    let updater = webview.updater_builder().endpoints(vec![endpoint])
        .map_err(|e| e.to_string())?.timeout(Duration::from_secs(30))
        .build().map_err(|e| e.to_string())?;
    // GitHub asset redirects can fail transiently during TLS negotiation.
    // Retry the complete updater request, retaining its manifest/signature checks.
    let mut result = updater.check().await;
    for _ in 0..2 {
        if result.is_ok() { break; }
        result = updater.check().await;
    }
    let Some(update) = result.map_err(|e| format!("无法获取有效更新清单（已尝试 3 次）：{e}。请检查 GitHub 及其下载域名的网络连接，或前往发布页面手动下载"))? else { return Ok(None) };
    let metadata = json!({
        "currentVersion": update.current_version,
        "version": update.version,
        "body": update.body,
        "rawJson": update.raw_json,
    });
    let rid = webview.resources_table().add(update);
    let mut metadata = metadata;
    metadata["rid"] = json!(rid);
    Ok(Some(metadata))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn feed_selects_newest_valid_repository_tag() {
        let feed = r#"<feed xmlns="http://www.w3.org/2005/Atom">
          <entry><link href="https://example.com/releases/tag/v9.0.0"/></entry>
          <entry><link href="https://github.com/yeyongzhi/Lumo-Hub/releases/tag/not-a-version"/></entry>
          <entry><link href="https://github.com/yeyongzhi/Lumo-Hub/releases/tag/v0.0.1-beta.2"/></entry>
          <entry><link href="https://github.com/yeyongzhi/Lumo-Hub/releases/tag/v0.0.1-beta.1"/></entry>
        </feed>"#;
        assert_eq!(beta_manifest(feed).unwrap().as_str(), "https://github.com/yeyongzhi/Lumo-Hub/releases/download/v0.0.1-beta.2/latest.json");
        assert!(beta_manifest("<feed/>").is_err());
        assert!(beta_manifest("invalid XML").is_err());
    }

    #[test]
    #[ignore = "requires GitHub network access"]
    fn live_feed_and_manifest_are_accessible_without_api() {
        tauri::async_runtime::block_on(async {
            let endpoint = resolve_manifest(true).await.unwrap();
            assert_eq!(endpoint.host_str(), Some("github.com"));
            let manifest: Value = reqwest::Client::new().get(endpoint).send().await.unwrap()
                .error_for_status().unwrap().json().await.unwrap();
            assert!(semver::Version::parse(manifest["version"].as_str().unwrap()).is_ok());
            assert!(manifest["platforms"]["windows-x86_64"]["signature"].as_str().is_some_and(|s| !s.is_empty()));
        });
    }
}
