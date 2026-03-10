use std::sync::Arc;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::error::CoreResult;

// ---------------------------------------------------------------------------
// Tipos públicos
// ---------------------------------------------------------------------------

/// Recursos que se pueden bloquear para aligerar la carga de la página.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BlockedResource {
    Images,
    Fonts,
    Media,
    Stylesheet,
    /// Cualquier URL que matchee el patrón (substring match)
    Pattern(String),
}

/// Condición de espera antes de considerar la página "lista".
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WaitFor {
    /// Esperar a que el DOM esté parseado (más rápido)
    DomReady,
    /// Esperar a que no haya requests en vuelo (más completo)
    NetworkIdle,
    /// Esperar a que exista un selector CSS
    Selector(String),
}

impl Default for WaitFor {
    fn default() -> Self { WaitFor::DomReady }
}

/// Opciones para una llamada one-shot al headless.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HeadlessOptions {
    /// Método HTTP (default: GET)
    #[serde(default = "default_method")]
    pub method: String,

    /// Headers adicionales
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,

    /// Body (para POST/PUT)
    #[serde(default)]
    pub body: Option<String>,

    /// Cuándo considerar la página lista
    #[serde(default)]
    pub wait_for: WaitFor,

    /// JS a evaluar tras cargar; el resultado va en `HeadlessResponse::result`
    #[serde(default)]
    pub javascript: Option<String>,

    /// Recursos a bloquear
    #[serde(default)]
    pub block: Vec<BlockedResource>,

    /// Patrones de URL cuyas requests/responses capturar (substring match)
    #[serde(default)]
    pub capture: Vec<String>,

    /// Timeout en ms (default: 15 000)
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

fn default_method() -> String { "GET".to_string() }
fn default_timeout_ms() -> u64 { 15_000 }

/// Una request/response interceptada.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedRequest {
    pub url:     String,
    pub method:  String,
    pub status:  Option<u16>,
    pub body:    Option<String>,
    pub headers: std::collections::HashMap<String, String>,
}

/// Resultado de una llamada headless one-shot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadlessResponse {
    pub url:      String,
    pub status:   u16,
    /// HTML final tras ejecutar JS
    pub html:     String,
    /// Resultado del `javascript` evaluado, si se pasó
    pub result:   Option<serde_json::Value>,
    /// Requests capturadas que matchearon algún patrón de `capture`
    pub captured: Vec<CapturedRequest>,
    pub cookies:  Vec<Cookie>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookie {
    pub name:      String,
    pub value:     String,
    pub domain:    String,
    pub path:      String,
    pub secure:    bool,
    pub http_only: bool,
}

// ---------------------------------------------------------------------------
// Trait principal
// ---------------------------------------------------------------------------

#[async_trait]
pub trait HeadlessBrowser: Send + Sync {
    async fn fetch(&self, url: &str, options: HeadlessOptions) -> CoreResult<HeadlessResponse>;

    /// true si esta plataforma tiene headless disponible.
    /// El sandbox lo expone a JS para que extensiones degraden graciosamente.
    fn is_available(&self) -> bool;
}

// ---------------------------------------------------------------------------
// Implementación nula — Axum y cualquier plataforma sin webview
// ---------------------------------------------------------------------------

pub struct NoopHeadless;

#[async_trait]
impl HeadlessBrowser for NoopHeadless {
    async fn fetch(&self, _url: &str, _options: HeadlessOptions) -> CoreResult<HeadlessResponse> {
        Err(crate::error::CoreError::Internal(
            "Headless browser not available on this platform".into()
        ))
    }

    fn is_available(&self) -> bool { false }
}

// ---------------------------------------------------------------------------
// Handle compartido que vive en AppState
// ---------------------------------------------------------------------------

pub type HeadlessHandle = Arc<dyn HeadlessBrowser>;

pub fn noop_headless() -> HeadlessHandle {
    Arc::new(NoopHeadless)
}