//! Build a [`SearchBackendImpl`] from current settings.

use crate::gateway::web_search::backends::brave::BraveSearchBackend;
use crate::gateway::web_search::backends::llm_backed::LlmBackedSearchBackend;
use crate::gateway::web_search::backends::tavily::TavilySearchBackend;
use crate::gateway::web_search::backend::{SearchBackendImpl, SearchBackendKind};
use crate::providers::ProviderForGateway;

/// Settings needed to construct a backend. Mirrors the fields on
/// `AppSettings` so the gateway layer can pass them in directly.
#[derive(Debug, Clone)]
pub struct BackendSettings {
    pub kind: SearchBackendKind,
    pub brave_api_key: String,
    pub tavily_api_key: String,
    pub max_results: u32,
    pub llm_provider_id: Option<i64>,
    pub proxy_url: String,
}

/// Build a backend from settings. Returns `None` if the selected backend is
/// not yet fully configured (e.g. missing API key) — the interceptor treats
/// `None` as a configuration error and synthesizes a `web_search_tool_result`
/// error block.
pub fn build_backend(
    settings: &BackendSettings,
    providers: &[ProviderForGateway],
) -> Option<SearchBackendImpl> {
    let proxy_url = (!settings.proxy_url.is_empty()).then(|| settings.proxy_url.clone());

    match settings.kind {
        SearchBackendKind::Brave => {
            if settings.brave_api_key.trim().is_empty() {
                return None;
            }
            Some(SearchBackendImpl::Brave(
                BraveSearchBackend::new(settings.brave_api_key.clone()).with_proxy(proxy_url),
            ))
        }
        SearchBackendKind::Tavily => {
            if settings.tavily_api_key.trim().is_empty() {
                return None;
            }
            Some(SearchBackendImpl::Tavily(
                TavilySearchBackend::new(settings.tavily_api_key.clone()).with_proxy(proxy_url),
            ))
        }
        SearchBackendKind::LlmBacked => {
            let provider_id = settings.llm_provider_id?;
            let provider = providers.iter().find(|p| p.id == provider_id)?;
            let api_key = provider.api_key_plaintext.clone();
            if api_key.trim().is_empty() {
                return None;
            }
            let base_url = provider
                .base_urls
                .first()
                .cloned()
                .unwrap_or_default();
            let model = settings_llm_model(settings, provider);
            Some(SearchBackendImpl::LlmBacked(
                LlmBackedSearchBackend::new(
                    provider.id,
                    provider.name.clone(),
                    base_url,
                    api_key,
                    model,
                ),
            ))
        }
    }
}

fn settings_llm_model(_settings: &BackendSettings, _provider: &ProviderForGateway) -> String {
    // For v1 we delegate to the provider's first mapped model that starts with
    // `claude-`. Future: add a dedicated `web_search_llm_model` setting.
    "claude-sonnet-4-5".to_string()
}
