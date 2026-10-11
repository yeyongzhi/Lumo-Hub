use serde_json::Value;
use std::time::Duration;

#[tauri::command]
pub async fn fetch_home_feed(kind: String, city: Option<String>) -> Result<Value, String> {
    let address = match kind.as_str() {
        "weather" => "https://wttr.in/",
        "hitokoto" => "https://v1.hitokoto.cn/",
        "news" => "https://60s.viki.moe/v2/60s",
        "poetry" => "https://v1.jinrishici.com/all.json",
        _ => return Err("不支持的信息模块".into()),
    };
    let mut url = reqwest::Url::parse(address).map_err(|e| e.to_string())?;
    if kind == "weather" {
        let city = city.unwrap_or_default();
        let city = city.trim();
        if city.chars().count() > 80 { return Err("城市名称过长".into()); }
        if !city.is_empty() { url.path_segments_mut().map_err(|_| "天气地址无效")?.push(city); }
        url.query_pairs_mut().append_pair("format", "j1").append_pair("lang", "zh");
    }
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .connect_timeout(Duration::from_secs(8))
        .user_agent("Lumo-Hub/0.1.1")
        .build().map_err(|_| "无法创建网络连接")?;
    let response = client.get(url).send().await.map_err(|e| if e.is_timeout() { "请求超时，请稍后刷新".to_string() } else { "网络连接失败，请检查网络后刷新".to_string() })?
        .error_for_status().map_err(|e| format!("服务暂不可用：{}", e.status().map(|s| s.as_u16()).unwrap_or(0)))?;
    response.json::<Value>().await.map_err(|_| "接口返回了无效的数据，请稍后刷新".into())
}

#[cfg(test)]
mod tests {
    fn check(kind: &str, field: &str) {
        tauri::async_runtime::block_on(async {
            let value = super::fetch_home_feed(kind.into(), Some("Shanghai".into())).await.unwrap_or_else(|e| panic!("{kind}: {e}"));
            assert!(value.get(field).is_some(), "Invalid {kind} response");
        });
    }
    #[test]
    #[ignore = "requires live external service"]
    fn live_weather() { check("weather", "current_condition"); }
    #[test]
    #[ignore = "requires live external service"]
    fn live_hitokoto() { check("hitokoto", "hitokoto"); }
    #[test]
    #[ignore = "requires live external services"]
    fn live_news_and_poetry() { check("news", "data"); check("poetry", "content"); }
}
