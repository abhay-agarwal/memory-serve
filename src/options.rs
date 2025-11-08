use axum::http::StatusCode;

use crate::CacheControl;

/// Minimum file size (in bytes) to consider compression
pub(crate) const MIN_COMPRESS_SIZE: u64 = 128;

/// File mime types that can possibly be compressed
pub(crate) const COMPRESS_TYPES: &[&str] = &[
    "text/html",
    "text/css",
    "application/json",
    "text/javascript",
    "application/javascript",
    "application/xml",
    "text/xml",
    "image/svg+xml",
    "application/wasm",
];

/// Internal configuration shared across `MemoryServe` handlers.
#[derive(Debug)]
pub(super) struct ServeOptions {
    pub(super) index_file: Option<&'static str>,
    pub(super) index_on_subdirectories: bool,
    pub(super) fallback: Option<&'static str>,
    pub(super) fallback_status: StatusCode,
    pub(super) html_cache_control: CacheControl,
    pub(super) cache_control: CacheControl,
    pub(super) enable_brotli: bool,
    pub(super) enable_gzip: bool,
    pub(super) enable_clean_url: bool,
    pub(super) policies: Vec<Policy>,
}

impl Default for ServeOptions {
    /// Provide the default serving configuration used by `MemoryServe::default`.
    fn default() -> Self {
        Self {
            index_file: Some("/index.html"),
            index_on_subdirectories: false,
            fallback: None,
            fallback_status: StatusCode::NOT_FOUND,
            html_cache_control: CacheControl::Short,
            cache_control: CacheControl::Medium,
            enable_brotli: !cfg!(debug_assertions),
            enable_gzip: !cfg!(debug_assertions),
            enable_clean_url: false,
            policies: Vec::new(),
        }
    }
}

pub(super) struct Policy {
    matcher: Box<dyn Fn(&str) -> bool + Send + Sync>,
    cache_control: CacheControl,
}

impl std::fmt::Debug for Policy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Policy")
            .field("cache_control", &self.cache_control)
            .finish()
    }
}

impl Policy {
    pub(super) fn new<F>(matcher: F, cache_control: CacheControl) -> Self
    where
        F: Fn(&str) -> bool + Send + Sync + 'static,
    {
        Self {
            matcher: Box::new(matcher),
            cache_control,
        }
    }

    pub(super) fn matches(&self, path: &str) -> bool {
        (self.matcher)(path)
    }

    pub(super) fn cache_control(&self) -> CacheControl {
        self.cache_control
    }
}

impl ServeOptions {
    pub(super) fn add_policy<F>(&mut self, matcher: F, cache_control: CacheControl)
    where
        F: Fn(&str) -> bool + Send + Sync + 'static,
    {
        self.policies.push(Policy::new(matcher, cache_control));
    }

    pub(super) fn policy_for(&self, path: &str) -> Option<CacheControl> {
        self.policies
            .iter()
            .find(|policy| policy.matches(path))
            .map(Policy::cache_control)
    }
}
