use anyhow::Result;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

fn default_max_results() -> usize {
    10
}

pub struct DuckDuckGoClient {
    client: reqwest::Client,
}

impl DuckDuckGoClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
            .build()
            .unwrap();
        
        Self { client }
    }

    pub async fn search(&self, request: SearchRequest) -> Result<Vec<SearchResult>> {
        let query = urlencoding::encode(&request.query);
        let url = format!("https://html.duckduckgo.com/html/?q={}", query);
        
        let response = self.client.get(&url).send().await?;
        let body = response.text().await?;
        
        self.parse_search_results(&body, request.max_results)
    }

    fn parse_search_results(&self, html: &str, max_results: usize) -> Result<Vec<SearchResult>> {
        let document = Html::parse_document(html);
        
        // DuckDuckGo HTML structure selectors
        let result_selector = Selector::parse(".result").unwrap();
        let title_selector = Selector::parse(".result__title a").unwrap();
        let snippet_selector = Selector::parse(".result__snippet").unwrap();
        
        let mut results = Vec::new();
        
        for element in document.select(&result_selector).take(max_results) {
            let title = element
                .select(&title_selector)
                .next()
                .and_then(|e| e.text().collect::<String>().trim().into())
                .unwrap_or_default();
                
            let url = element
                .select(&title_selector)
                .next()
                .and_then(|e| e.value().attr("href"))
                .unwrap_or_default()
                .to_string();
                
            let snippet = element
                .select(&snippet_selector)
                .next()
                .and_then(|e| e.text().collect::<String>().trim().into())
                .unwrap_or_default();
                
            if !title.is_empty() && !url.is_empty() {
                results.push(SearchResult {
                    title,
                    url,
                    snippet,
                });
            }
        }
        
        Ok(results)
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
        let results = client.search_with_params("rust programming", Some(5), None, None).await;
        
        assert!(results.is_ok());
        let results = results.unwrap();
        assert!(!results.is_empty());
        
        for result in results {
            assert!(!result.title.is_empty());
            assert!(!result.url.is_empty());
        }
    }
}
