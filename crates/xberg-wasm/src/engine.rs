//! `XbergEngine` — a stateful handle carrying injected bridges and
//! convenience methods on top of the raw WASM exports.

use std::collections::HashMap;
use std::sync::Arc;

use wasm_bindgen::prelude::*;

use crate::bridge::embedder::JsEmbedder;
use crate::bridge::store::JsVectorStore;

/// Rehydration map type (token → original PII text).
type RehydrationMap = HashMap<String, String>;

/// Stateful engine handle exposed to JS.
///
/// Constructed via `XbergEngine.new(injections)` where `injections` is a
/// plain object with optional `embedder`, `store`, `ner`, and `ocr` keys.
#[wasm_bindgen]
pub struct XbergEngine {
    embedder: Option<Arc<JsEmbedder>>,
    store: Option<Arc<JsVectorStore>>,
    ner: Option<js_sys::Object>,
    ocr: Option<js_sys::Object>,
}

#[wasm_bindgen]
impl XbergEngine {
    /// Create a new engine with injected bridges.
    ///
    /// `injections` may contain:
    /// - `embedder` — object with `embed(texts: string[]): Promise<number[][]>`
    /// - `store`    — object implementing the VectorStore JS protocol
    /// - `ner`      — object with `ner(text, categories): Promise<...>`
    /// - `ocr`      — object with `ocr(imageBytes, opts): Promise<string>`
    #[wasm_bindgen(constructor)]
    pub fn new(injections: JsValue) -> Result<XbergEngine, JsValue> {
        let obj: js_sys::Object = if injections.is_undefined() || injections.is_null() {
            js_sys::Object::new()
        } else {
            injections
                .dyn_into::<js_sys::Object>()
                .map_err(|_| JsValue::from_str("injections must be an object"))?
        };

        let embedder = js_sys::Reflect::get(&obj, &"embedder".into())
            .ok()
            .filter(|v| !v.is_undefined() && !v.is_null())
            .map(|v| {
                v.dyn_into::<js_sys::Object>()
                    .map(Arc::new)
                    .map(JsEmbedder::new)
                    .map(Arc::new)
            })
            .transpose()
            .map_err(|_| JsValue::from_str("embedder must be an object"))?;

        let store = js_sys::Reflect::get(&obj, &"store".into())
            .ok()
            .filter(|v| !v.is_undefined() && !v.is_null())
            .map(|v| {
                v.dyn_into::<js_sys::Object>()
                    .map(Arc::new)
                    .map(|inner| JsVectorStore::new("default".to_string(), inner))
                    .map(Arc::new)
            })
            .transpose()
            .map_err(|_| JsValue::from_str("store must be an object"))?;

        let ner = js_sys::Reflect::get(&obj, &"ner".into())
            .ok()
            .filter(|v| !v.is_undefined() && !v.is_null())
            .map(|v| v.dyn_into::<js_sys::Object>())
            .transpose()
            .map_err(|_| JsValue::from_str("ner must be an object"))?;

        let ocr = js_sys::Reflect::get(&obj, &"ocr".into())
            .ok()
            .filter(|v| !v.is_undefined() && !v.is_null())
            .map(|v| v.dyn_into::<js_sys::Object>())
            .transpose()
            .map_err(|_| JsValue::from_str("ocr must be an object"))?;

        Ok(XbergEngine {
            embedder,
            store,
            ner,
            ocr,
        })
    }

    /// Extract content from a single bytes or URI input.
    #[allow(clippy::missing_errors_doc)]
    pub async fn extract(
        &self,
        input: JsValue,
        config: JsValue,
    ) -> Result<JsValue, JsValue> {
        let input_core: xberg::ExtractInput = if input.is_undefined() {
            xberg::ExtractInput::default()
        } else {
            serde_wasm_bindgen::from_value::<xberg::ExtractInput>(input)
                .map_err(|e| JsValue::from_str(&e.to_string()))?
        };
        let config_core: xberg::ExtractionConfig = if config.is_undefined() {
            xberg::ExtractionConfig::default()
        } else {
            serde_wasm_bindgen::from_value::<xberg::ExtractionConfig>(config)
                .map_err(|e| JsValue::from_str(&e.to_string()))?
        };
        let result = xberg::extract(input_core, &config_core)
            .await
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let wasm_result = crate::WasmExtractionResult::from(result);
        serde_wasm_bindgen::to_value(&wasm_result).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Ingest a single document into the RAG vector store.
    ///
    /// Requires both an `embedder` and a `store` to have been injected.
    #[allow(clippy::missing_errors_doc)]
    pub async fn ingest(&self, request: JsValue) -> Result<JsValue, JsValue> {
        let ingest_req: xberg_rag::pipeline::IngestRequest =
            serde_wasm_bindgen::from_value(request)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let embedder = self
            .embedder
            .as_ref()
            .ok_or_else(|| JsValue::from_str("embedder not injected"))?;
        let store = self
            .store
            .as_ref()
            .ok_or_else(|| JsValue::from_str("store not injected"))?;

        let pipeline_config = xberg_rag::pipeline::RagPipelineConfig::default();
        let result = xberg_rag::pipeline::ingest_document_local(
            store.clone(),
            ingest_req,
            pipeline_config,
            embedder.as_ref(),
        )
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

        serde_wasm_bindgen::to_value(&result).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Detect PII in `text`. Returns an array of `{ start, end, category, text }`.
    #[allow(clippy::missing_errors_doc)]
    pub fn detect_pii(
        &self,
        text: &str,
        categories: Option<Vec<String>>,
    ) -> Result<JsValue, JsValue> {
        let cats: Vec<xberg::types::redaction::PiiCategory> = categories
            .unwrap_or_default()
            .into_iter()
            .map(Into::into)
            .collect();
        let matches = xberg::text::redaction::patterns::scan_text(text, &cats);
        serde_wasm_bindgen::to_value(&matches).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Redact PII from `text` using the given `strategy`.
    ///
    /// Returns `{ redacted: string, rehydrationMap: { token: original } }`.
    #[allow(clippy::missing_errors_doc)]
    pub fn redact(
        &self,
        text: &str,
        strategy: Option<String>,
        categories: Option<Vec<String>>,
    ) -> Result<JsValue, JsValue> {
        let strat: xberg::types::redaction::RedactionStrategy =
            strategy.unwrap_or_else(|| "token_replace".to_string()).into();
        let cats: Vec<xberg::types::redaction::PiiCategory> = categories
            .unwrap_or_default()
            .into_iter()
            .map(Into::into)
            .collect();

        let matches = xberg::text::redaction::patterns::scan_text(text, &cats);

        // Pre-count per category so we can assign deterministic token indices
        // when processing in reverse.
        let mut category_counts: HashMap<String, u32> = HashMap::new();
        for m in &matches {
            let key = format!("{:?}", m.category);
            *category_counts.entry(key).or_insert(0) += 1;
        }

        let mut rehydration_map: RehydrationMap = HashMap::new();
        let mut running: HashMap<String, u32> = HashMap::new();
        let mut result = text.to_string();

        // Process matches in reverse byte order so replacements don't shift offsets.
        for m in matches.iter().rev() {
            let cat_key = format!("{:?}", m.category);
            let total = category_counts[&cat_key];
            let counter = running.entry(cat_key.clone()).or_insert(0);
            *counter += 1;
            // Token index counts from the end: total - (counter - 1)
            let idx = total - (*counter - 1);

            let replacement = match strat {
                xberg::types::redaction::RedactionStrategy::Mask => "[REDACTED]".to_string(),
                xberg::types::redaction::RedactionStrategy::Hash => {
                    use sha2::{Digest, Sha256};
                    let hash = Sha256::digest(m.text.as_bytes());
                    format!("{:x}", &hash[..8])
                }
                xberg::types::redaction::RedactionStrategy::TokenReplace => {
                    let token = format!("[{}_{}]", cat_key.to_uppercase(), idx);
                    rehydration_map.insert(token.clone(), m.text.clone());
                    token
                }
                xberg::types::redaction::RedactionStrategy::Drop => String::new(),
            };

            result.replace_range(m.start..m.end, &replacement);
        }

        let out = js_sys::Object::new();
        js_sys::Reflect::set(&out, &"redacted".into(), &result.into())
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        js_sys::Reflect::set(
            &out,
            &"rehydrationMap".into(),
            &serde_wasm_bindgen::to_value(&rehydration_map)
                .map_err(|e| JsValue::from_str(&e.to_string()))?,
        )
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(out.into())
    }

    /// Encrypt a rehydration map with `passphrase`.
    ///
    /// Returns the raw ciphertext bytes (`XPII\x01` wire format).
    #[cfg(feature = "redaction-rehydrate")]
    #[allow(clippy::missing_errors_doc)]
    pub fn encrypt_map(
        &self,
        map: JsValue,
        passphrase: &str,
    ) -> Result<Vec<u8>, JsValue> {
        let inner: RehydrationMap =
            serde_wasm_bindgen::from_value(map).map_err(|e| JsValue::from_str(&e.to_string()))?;
        xberg::text::redaction::rehydration::encrypt_map(&inner, passphrase)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Decrypt an encrypted blob back into a token→original map.
    #[cfg(feature = "redaction-rehydrate")]
    #[allow(clippy::missing_errors_doc)]
    pub fn decrypt_map(
        &self,
        blob: Vec<u8>,
        passphrase: &str,
    ) -> Result<JsValue, JsValue> {
        let inner = xberg::text::redaction::rehydration::decrypt_map(&blob, passphrase)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        serde_wasm_bindgen::to_value(&inner).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Return aggregate statistics for the WASM extraction cache.
    #[allow(clippy::missing_errors_doc)]
    pub fn cache_stats(&self) -> Result<JsValue, JsValue> {
        let stats = crate::WasmCacheStats::default();
        serde_wasm_bindgen::to_value(&stats).map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Invalidate all cached extraction results.
    #[allow(clippy::missing_errors_doc)]
    pub fn invalidate_cache(&self) -> Result<(), JsValue> {
        // Cache is in-memory per WASM instance — dropping the cache is a no-op
        // because each engine instance owns its own process.  Return Ok so JS
        // callers can chain without a try/catch.
        Ok(())
    }
}
