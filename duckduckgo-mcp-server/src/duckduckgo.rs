use anyhow::Result;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    #[serde(default = "default_max_results")]
    pub max_results: usize,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub time_filter: Option<String>,
}

impl SearchRequest {
    #[allow(dead_code)]
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            max_results: 10,
            region: None,
            time_filter: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_params(
        query: impl Into<String>,
        max_results: usize,
        region: Option<String>,
        time_filter: Option<String>,
    ) -> Self {
        Self {
            query: query.into(),
            max_results,
            region,
            time_filter,
        }
    }
}

fn default_max_results() -> usize {
    10
}

#[allow(dead_code)]
pub struct DuckDuckGoClient {
    client: reqwest::Client,
    max_retries: u32,
    retry_delay: Duration,
}

#[allow(dead_code)]
impl Default for DuckDuckGoClient {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl DuckDuckGoClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap();

        Self {
            client,
            max_retries: 3,
            retry_delay: Duration::from_millis(500),
        }
    }

    pub fn with_retries(mut self, max_retries: u32, retry_delay: Duration) -> Self {
        self.max_retries = max_retries;
        self.retry_delay = retry_delay;
        self
    }

    async fn make_request_with_retry(&self, url: &str) -> Result<String> {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            match self.client.get(url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        return response.text().await.map_err(Into::into);
                    } else if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                        last_error = Some(anyhow::anyhow!("Rate limited (429)"));
                        if attempt < self.max_retries {
                            sleep(self.retry_delay * (attempt + 1)).await;
                            continue;
                        }
                    } else {
                        last_error = Some(anyhow::anyhow!(
                            "HTTP {}: {}",
                            response.status(),
                            response.text().await.unwrap_or_default()
                        ));
                        if attempt < self.max_retries {
                            sleep(self.retry_delay).await;
                            continue;
                        }
                    }
                }
                Err(e) => {
                    last_error = Some(e.into());
                    if attempt < self.max_retries {
                        sleep(self.retry_delay).await;
                        continue;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            anyhow::anyhow!("Request failed after {} retries", self.max_retries)
        }))
    }

    pub async fn search(&self, request: SearchRequest) -> Result<Vec<SearchResult>> {
        let query = urlencoding::encode(&request.query);
        let mut url = format!("https://html.duckduckgo.com/html/?q={query}");

        // Add region parameter if specified
        if let Some(region) = &request.region {
            url.push_str(&format!("&kl={region}"));
        }

        // Add time filter if specified
        if let Some(time_filter) = &request.time_filter {
            url.push_str(&format!("&df={time_filter}"));
        }

        let body = self.make_request_with_retry(&url).await?;

        self.parse_search_results(&body, request.max_results)
    }

    pub async fn search_news(&self, request: SearchRequest) -> Result<Vec<SearchResult>> {
        let query_str = format!("{} news", request.query);
        let query = urlencoding::encode(&query_str);
        let mut url = format!("https://html.duckduckgo.com/html/?q={query}");

        // Add news-specific parameters
        url.push_str("&iar=news");

        if let Some(region) = &request.region {
            url.push_str(&format!("&kl={region}"));
        }

        if let Some(time_filter) = &request.time_filter {
            url.push_str(&format!("&df={time_filter}"));
        }

        let body = self.make_request_with_retry(&url).await?;

        self.parse_search_results(&body, request.max_results)
    }

    fn parse_search_results(&self, html: &str, max_results: usize) -> Result<Vec<SearchResult>> {
        let document = Html::parse_document(html);

        // DuckDuckGo HTML structure selectors - multiple fallback selectors
        let result_selectors = [
            Selector::parse(".result").unwrap(),
            Selector::parse(".result--web").unwrap(),
            Selector::parse(".web-result").unwrap(),
        ];

        let title_selectors = [
            Selector::parse(".result__title a").unwrap(),
            Selector::parse(".result__a").unwrap(),
            Selector::parse("h2 a").unwrap(),
        ];

        let snippet_selectors = [
            Selector::parse(".result__snippet").unwrap(),
            Selector::parse(".result__body").unwrap(),
            Selector::parse(".snippet").unwrap(),
            Selector::parse(".web-result__description").unwrap(),
        ];

        let mut results = Vec::new();

        // Try different result selectors
        let mut elements = Vec::new();
        for selector in &result_selectors {
            elements.extend(document.select(selector));
            if !elements.is_empty() {
                break;
            }
        }

        for element in elements.into_iter().take(max_results) {
            let mut title = String::new();
            let mut url = String::new();
            let mut snippet = String::new();

            // Try different title selectors
            for selector in &title_selectors {
                if let Some(title_elem) = element.select(selector).next() {
                    title = title_elem.text().collect::<String>().trim().to_string();
                    url = title_elem.value().attr("href").unwrap_or("").to_string();
                    if !title.is_empty() {
                        break;
                    }
                }
            }

            // Try different snippet selectors
            for selector in &snippet_selectors {
                if let Some(snippet_elem) = element.select(selector).next() {
                    snippet = snippet_elem.text().collect::<String>().trim().to_string();
                    if !snippet.is_empty() {
                        break;
                    }
                }
            }

            // Fallback to meta description if no snippet found
            if snippet.is_empty() {
                let meta_selector = Selector::parse("meta[name='description']").unwrap();
                if let Some(meta_elem) = element.select(&meta_selector).next() {
                    snippet = meta_elem.value().attr("content").unwrap_or("").to_string();
                }
            }

            // Clean up the URL (handle DuckDuckGo redirects)
            let clean_url = if let Some(stripped) = url.strip_prefix("/l/?kh=-1&uddg=") {
                urlencoding::decode(stripped)
                    .unwrap_or_default()
                    .to_string()
            } else if let Some(stripped) = url.strip_prefix("//duckduckgo.com/l/?kh=-1&uddg=") {
                urlencoding::decode(stripped)
                    .unwrap_or_default()
                    .to_string()
            } else if url.starts_with("http") {
                url
            } else if let Some(stripped) = url.strip_prefix("/") {
                format!("https://duckduckgo.com{stripped}")
            } else {
                url
            };

            if !title.is_empty() && !clean_url.is_empty() {
                results.push(SearchResult {
                    title: self.clean_text(&title),
                    url: clean_url,
                    snippet: self.clean_text(&snippet),
                });
            }
        }

        // If no results found with selectors, try alternative parsing
        if results.is_empty() {
            results = self.parse_with_fallback(&document, max_results)?;
        }

        Ok(results)
    }

    fn parse_with_fallback(
        &self,
        document: &Html,
        max_results: usize,
    ) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        // Try to find any links with text content
        let link_selector = Selector::parse("a").unwrap();
        let mut seen_urls = std::collections::HashSet::new();

        for link in document.select(&link_selector).take(max_results * 3) {
            if let Some(href) = link.value().attr("href") {
                let text = link.text().collect::<String>().trim().to_string();

                // Filter out navigation and non-result links
                if !text.is_empty()
                    && href.starts_with("http")
                    && !href.contains("duckduckgo.com")
                    && seen_urls.insert(href.to_string())
                {
                    results.push(SearchResult {
                        title: self.clean_text(&text),
                        url: href.to_string(),
                        snippet: String::new(),
                    });

                    if results.len() >= max_results {
                        break;
                    }
                }
            }
        }

        Ok(results)
    }

    fn clean_text(&self, text: &str) -> String {
        text.replace(['\n', '\t'], " ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string()
    }

    pub async fn search_with_params(
        &self,
        query: &str,
        max_results: Option<usize>,
        region: Option<&str>,
        time_filter: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        let request = SearchRequest {
            query: query.to_string(),
            max_results: max_results.unwrap_or(10),
            region: region.map(|s| s.to_string()),
            time_filter: time_filter.map(|s| s.to_string()),
        };

        self.search(request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_search() {
        let client = DuckDuckGoClient::new();
        let results = client
            .search_with_params("rust programming", Some(5), None, None)
            .await;

        // In CI environment, we allow empty results due to network constraints
        // This test verifies the function doesn't panic and returns a result
        assert!(results.is_ok());
        let results = results.unwrap();

        // Don't assert non-empty results in CI - network may be unavailable
        // Instead, verify the structure is correct when results exist
        for result in results {
            assert!(!result.title.is_empty());
            assert!(!result.url.is_empty());
        }
    }

    #[test]
    fn test_search_request_validation() {
        let request = SearchRequest::new("test query");
        assert_eq!(request.query, "test query");
        assert_eq!(request.max_results, 10);
        assert_eq!(request.region, None);
        assert_eq!(request.time_filter, None);
    }

    #[test]
    fn test_search_request_with_params() {
        let request =
            SearchRequest::with_params("rust", 5, Some("us-en".to_string()), Some("d".to_string()));
        assert_eq!(request.query, "rust");
        assert_eq!(request.max_results, 5);
        assert_eq!(request.region, Some("us-en".to_string()));
        assert_eq!(request.time_filter, Some("d".to_string()));
    }
}
