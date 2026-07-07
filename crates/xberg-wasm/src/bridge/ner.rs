//! NER (Named Entity Recognition) bridge with injected-first dispatch.
//!
//! The WASM engine never bundles its own NER model bytes — instead it
//! delegates to an externally-injected JavaScript object that implements
//! a `ner(text, categories)` async method.  When no injection is
//! provided the bridge attempts an in-binary fallback under
//! `#[cfg(feature = "ner-candle-wasm")]`, but that path requires model
//! bytes that are not yet wired up, so it returns a clear diagnostic
//! error for now.

#[cfg(target_arch = "wasm32")]
use js_sys::{Function, Object, Promise, Reflect};
use wasm_bindgen::prelude::*;

use xberg::types::entity::{Entity, EntityCategory};

/// Resolve the best available NER backend for the current request.
///
/// 1. If `injected` is `Some(obj)`, call `obj.ner(text, categories)` — the
///    host is expected to return a JSON-serializable array of Entity objects.
/// 2. If `injected` is `None` and the `ner-candle-wasm` feature is enabled,
///    an in-binary Candle backend could be used, but model bytes are not yet
///    available so a descriptive error is returned.
/// 3. Otherwise return an error explaining that NER is unavailable.
pub async fn resolve_ner(
    injected: Option<js_sys::Object>,
    text: &str,
    categories: &[EntityCategory],
) -> Result<Vec<Entity>, JsValue> {
    resolve_ner_with_timeout(injected, text, categories, crate::bridge::BRIDGE_TIMEOUT_MS).await
}

/// Like [`resolve_ner`] but with a configurable bridge timeout.
pub async fn resolve_ner_with_timeout(
    injected: Option<js_sys::Object>,
    text: &str,
    categories: &[EntityCategory],
    timeout_ms: u32,
) -> Result<Vec<Entity>, JsValue> {
    match injected {
        Some(obj) => call_injected_ner(obj, text, categories, timeout_ms).await,
        None => fallback_ner(),
    }
}

/// Call the injected JS `ner(text, categories)` method and deserialize the
/// returned promise into a Vec<Entity>.
async fn call_injected_ner(
    obj: Object,
    text: &str,
    categories: &[EntityCategory],
    timeout_ms: u32,
) -> Result<Vec<Entity>, JsValue> {
    let fn_val = Reflect::get(&obj, &JsValue::from_str("ner"))
        .map_err(|e| js_from_any(format!("failed to read 'ner' property: {e:?}")))?;
    let func: Function = fn_val.dyn_into().map_err(|_| {
        js_from_any("injected NER object has no 'ner' function")
    })?;

    let js_text = JsValue::from_str(text);
    let js_cats = js_sys::Array::new();
    for c in categories {
        let cat_str = serde_json::to_value(c)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_default();
        js_cats.push(&JsValue::from_str(&cat_str));
    }
    let args = js_sys::Array::of2(&js_text, &js_cats);

    let result = func.apply(&obj, &args)?;
    let promise = Promise::from(result);
    let js_val = crate::bridge::timed_js_future_with_timeout(promise, timeout_ms).await?;

    let entities: Vec<Entity> = serde_wasm_bindgen::from_value(js_val)
        .map_err(|e| js_from_any(format!("failed to deserialize NER result: {e}")))?;
    Ok(entities)
}

/// In-binary NER fallback.
///
/// When no JS backend is injected this path is tried.  Under the
/// `ner-candle-wasm` feature the Candle GLiNER2 backend is available,
/// but it requires model bytes that are not yet wired up through the
/// WASM bridge, so we return a clear diagnostic error.
fn fallback_ner() -> Result<Vec<Entity>, JsValue> {
    #[cfg(feature = "ner-candle-wasm")]
    {
        Err(js_from_any(
            "NER unavailable: no injected backend and ner-candle-wasm not initialized with model bytes",
        ))
    }
    #[cfg(not(feature = "ner-candle-wasm"))]
    {
        Err(js_from_any(
            "NER unavailable: no injected backend and ner-candle-wasm disabled",
        ))
    }
}

/// Convert a Display error into a JsValue suitable for propagation.
fn js_from_any(v: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&v.to_string())
}
