use crate::config::ServerConfig;
use anyhow::Result;
use moka::future::Cache;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::Duration;
use tokio::time::sleep;
use tracing::instrument;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub source: Option<String>,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    #[serde(default = "default_max_results")]
    pub max_results: usize,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub time_filter: Option<String>,
    #[serde(default)]
    pub safe_search: Option<bool>,
}

impl Default for SearchRequest {
    fn default() -> Self {
        Self {
            query: String::new(),
            max_results: default_max_results(),
            region: None,
            time_filter: None,
            safe_search: None,
        }
    }
}

fn default_max_results() -> usize {
    10
}

#[derive(Debug, Clone)]
pub struct EnhancedDuckDuckGoClient {
    client: reqwest::Client,
    cache: Cache<String, Vec<SearchResult>>,
    config: ServerConfig,
}

impl EnhancedDuckDuckGoClient {
    pub fn new(config: ServerConfig) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(Duration::from_secs(config.request_timeout_seconds))
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");

        let cache = Cache::builder()
            .max_capacity(1000)
            .time_to_live(Duration::from_secs(config.cache_ttl_seconds))
            .build();

        Self {
            client,
            cache,
            config,
        }
    }

    #[instrument(skip(self))]
    pub async fn search(&self, request: SearchRequest) -> Result<Vec<SearchResult>> {
        // Create cache key
        let cache_key = format!(
            "search:{}:{}:{}:{}",
            request.query,
            request.max_results,
            request.region.as_deref().unwrap_or(""),
            request.time_filter.as_deref().unwrap_or("")
        );

        // Try to get from cache first
        if let Some(cached) = self.cache.get(&cache_key).await {
            tracing::debug!("Cache hit for query: {}", request.query);
            return Ok(cached);
        }

        tracing::debug!("Cache miss for query: {}", request.query);

        let results = self.perform_search(request.clone()).await?;

        // Cache the results
        self.cache.insert(cache_key, results.clone()).await;

        Ok(results)
    }

    #[instrument(skip(self))]
    pub async fn search_news(&self, request: SearchRequest) -> Result<Vec<SearchResult>> {
        let mut news_request = request.clone();
        news_request.query = format!("{} news", request.query);

        let cache_key = format!(
            "news:{}:{}:{}:{}",
            request.query,
            request.max_results,
            request.region.as_deref().unwrap_or(""),
            request.time_filter.as_deref().unwrap_or("")
        );

        if let Some(cached) = self.cache.get(&cache_key).await {
            tracing::debug!("Cache hit for news query: {}", request.query);
            return Ok(cached);
        }

        let results = self.perform_news_search(news_request).await?;
        self.cache.insert(cache_key, results.clone()).await;

        Ok(results)
    }

    async fn perform_search(&self, request: SearchRequest) -> Result<Vec<SearchResult>> {
        let query = urlencoding::encode(&request.query);
        let mut url = format!("https://html.duckduckgo.com/html/?q={query}");

        if let Some(region) = &request.region {
            url.push_str(&format!("&kl={region}"));
        }

        if let Some(time_filter) = &request.time_filter {
            url.push_str(&format!("&df={time_filter}"));
        }

        if let Some(safe) = request.safe_search {
            url.push_str(if safe { "&kp=1" } else { "&kp=-1" });
        }

        let body = self.make_request_with_retry(&url).await?;
        self.parse_search_results(&body, request.max_results)
    }

    async fn perform_news_search(&self, request: SearchRequest) -> Result<Vec<SearchResult>> {
        let query = urlencoding::encode(&request.query);
        let mut url = format!("https://html.duckduckgo.com/html/?q={query}");
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

    async fn make_request_with_retry(&self, url: &str) -> Result<String> {
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            match self.client.get(url).send().await {
                Ok(response) => {
                    let status = response.status();
                    let body = response.text().await?;

                    if status.is_success() {
                        return Ok(body);
                    } else if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                        last_error = Some(anyhow::anyhow!("Rate limited by DuckDuckGo"));
                        if attempt < self.config.max_retries {
                            let delay = Duration::from_millis(
                                self.config.retry_delay_ms * (attempt + 1) as u64,
                            );
                            sleep(delay).await;
                            continue;
                        }
                    } else {
                        last_error = Some(anyhow::anyhow!("HTTP {}: {}", status, body));
                        if attempt < self.config.max_retries {
                            sleep(Duration::from_millis(self.config.retry_delay_ms)).await;
                            continue;
                        }
                    }
                }
                Err(e) => {
                    last_error = Some(anyhow::anyhow!("Request failed: {}", e));
                    if attempt < self.config.max_retries {
                        sleep(Duration::from_millis(self.config.retry_delay_ms)).await;
                        continue;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Request failed after retries")))
    }

    fn parse_search_results(&self, html: &str, max_results: usize) -> Result<Vec<SearchResult>> {
        let document = Html::parse_document(html);
        let mut results = Vec::new();
        let mut seen_urls = HashSet::new();

        // Primary selectors for DuckDuckGo HTML structure
        let result_selectors = [
            Selector::parse(".result").unwrap(),
            Selector::parse(".result--web").unwrap(),
            Selector::parse(".web-result").unwrap(),
        ];

        let title_selectors = [
            Selector::parse(".result__title a").unwrap(),
            Selector::parse(".result__a").unwrap(),
            Selector::parse("h2.result__title a").unwrap(),
        ];

        let snippet_selectors = [
            Selector::parse(".result__snippet").unwrap(),
            Selector::parse(".result__body").unwrap(),
            Selector::parse(".snippet").unwrap(),
        ];

        // Try each result selector
        for result_selector in &result_selectors {
            let elements = document.select(result_selector);

            for element in elements.take(max_results * 2) {
                // Take more to account for duplicates
                let mut title = String::new();
                let mut url = String::new();
                let mut snippet = String::new();

                // Extract title and URL
                for title_selector in &title_selectors {
                    if let Some(title_elem) = element.select(title_selector).next() {
                        title = self.clean_text(&title_elem.text().collect::<String>());
                        if let Some(href) = title_elem.value().attr("href") {
                            url = self.clean_url(href);
                        }
                        break;
                    }
                }

                // Extract snippet
                for snippet_selector in &snippet_selectors {
                    if let Some(snippet_elem) = element.select(snippet_selector).next() {
                        snippet = self.clean_text(&snippet_elem.text().collect::<String>());
                        break;
                    }
                }

                // Skip if title or URL is empty, or if URL is a duplicate
                if !title.is_empty() && !url.is_empty() && seen_urls.insert(url.clone()) {
                    results.push(SearchResult {
                        title,
                        url,
                        snippet,
                        source: Some("DuckDuckGo".to_string()),
                        timestamp: None,
                    });

                    if results.len() >= max_results {
                        break;
                    }
                }
            }

            if !results.is_empty() {
                break; // Stop if we found results with this selector
            }
        }

        // Fallback parsing if no results found
        if results.is_empty() {
            results = self.parse_with_fallback(&document, max_results, &mut seen_urls)?;
        }

        Ok(results)
    }

    fn parse_with_fallback(
        &self,
        document: &Html,
        max_results: usize,
        seen_urls: &mut HashSet<String>,
    ) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        let link_selector = Selector::parse("a[href]").unwrap();
        let links = document.select(&link_selector);

        for link in links {
            if let Some(href) = link.value().attr("href") {
                let text = self.clean_text(&link.text().collect::<String>());

                // Filter out navigation and non-result links
                if !text.is_empty()
                    && href.starts_with("http")
                    && !href.contains("duckduckgo.com")
                    && !href.contains("javascript:")
                    && seen_urls.insert(href.to_string())
                {
                    results.push(SearchResult {
                        title: text,
                        url: href.to_string(),
                        snippet: String::new(),
                        source: Some("DuckDuckGo".to_string()),
                        timestamp: None,
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
        text.replace(['\n', '\t', '\r'], " ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string()
    }

    fn clean_url(&self, url: &str) -> String {
        if let Some(stripped) = url.strip_prefix("/l/?kh=-1&uddg=") {
            urlencoding::decode(stripped)
                .unwrap_or_default()
                .to_string()
        } else if let Some(stripped) = url.strip_prefix("//duckduckgo.com/l/?kh=-1&uddg=") {
            urlencoding::decode(stripped)
                .unwrap_or_default()
                .to_string()
        } else if url.starts_with("http") {
            url.to_string()
        } else if let Some(stripped) = url.strip_prefix("/") {
            format!("https://duckduckgo.com{stripped}")
        } else {
            url.to_string()
        }
    }

    #[allow(dead_code)]
    pub async fn clear_cache(&self) {
        self.cache.invalidate_all();
    }

    #[allow(dead_code)]
    pub fn cache_stats(&self) -> (u64, u64) {
        (self.cache.entry_count(), 0)
    }
}
