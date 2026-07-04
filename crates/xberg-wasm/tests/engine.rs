#![cfg(target_arch = "wasm32")]
use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn engine_construction() {
    let injection = js_sys::eval("({})").unwrap().dyn_into::<js_sys::Object>().unwrap();
    let config = js_sys::eval("({})").unwrap();
    let engine = xberg_wasm::engine::XbergEngine::new(config.into(), injection.into());
    assert!(engine.is_ok());
}

#[wasm_bindgen_test]
async fn engine_construction_with_null_injection() {
    let config = js_sys::eval("({})").unwrap();
    let engine = xberg_wasm::engine::XbergEngine::new(config.into(), JsValue::NULL);
    assert!(engine.is_ok());
}

#[wasm_bindgen_test]
async fn detect_pii_returns_empty_for_clean_text() {
    let injection = js_sys::eval("({})").unwrap().dyn_into::<js_sys::Object>().unwrap();
    let config = js_sys::eval("({})").unwrap();
    let engine = xberg_wasm::engine::XbergEngine::new(config.into(), injection.into()).unwrap();
    let result = engine.detect_pii("Hello world".to_string()).unwrap();
    let matches: js_sys::Array = result.dyn_into().unwrap();
    assert_eq!(matches.length(), 0);
}

#[wasm_bindgen_test]
async fn detect_pii_finds_email() {
    let injection = js_sys::eval("({})").unwrap().dyn_into::<js_sys::Object>().unwrap();
    let config = js_sys::eval("({})").unwrap();
    let engine = xberg_wasm::engine::XbergEngine::new(config.into(), injection.into()).unwrap();
    let result = engine
        .detect_pii("Contact alice@example.com for info".to_string())
        .unwrap();
    let matches: js_sys::Array = result.dyn_into().unwrap();
    assert!(matches.length() > 0);
}

#[wasm_bindgen_test]
async fn redact_mask_strategy() {
    let injection = js_sys::eval("({})").unwrap().dyn_into::<js_sys::Object>().unwrap();
    let config = js_sys::eval("({})").unwrap();
    let engine = xberg_wasm::engine::XbergEngine::new(config.into(), injection.into()).unwrap();
    let result = engine
        .redact(
            "Email alice@example.com here".to_string(),
            JsValue::from_str("mask"),
        )
        .unwrap();
    let obj: js_sys::Object = result.dyn_into().unwrap();
    let redacted = js_sys::Reflect::get(&obj, &JsValue::from_str("redacted_text"))
        .unwrap()
        .as_string()
        .unwrap();
    assert!(!redacted.contains("alice@example.com"));
    assert!(redacted.contains("[REDACTED_"));
}

#[wasm_bindgen_test]
async fn redact_token_replace_strategy() {
    let injection = js_sys::eval("({})").unwrap().dyn_into::<js_sys::Object>().unwrap();
    let config = js_sys::eval("({})").unwrap();
    let engine = xberg_wasm::engine::XbergEngine::new(config.into(), injection.into()).unwrap();
    let result = engine
        .redact(
            "Email alice@example.com here".to_string(),
            JsValue::from_str("token_replace"),
        )
        .unwrap();
    let obj: js_sys::Object = result.dyn_into().unwrap();
    let token_map = js_sys::Reflect::get(&obj, &JsValue::from_str("token_map")).unwrap();
    assert!(!token_map.is_undefined());
    assert!(!token_map.is_null());
}

#[wasm_bindgen_test]
async fn extract_missing_injection_errors() {
    let injection = js_sys::eval("({})").unwrap().dyn_into::<js_sys::Object>().unwrap();
    let config = js_sys::eval("({})").unwrap();
    let engine = xberg_wasm::engine::XbergEngine::new(config.into(), injection.into()).unwrap();

    // query without store should error
    let result = engine
        .query("test".to_string(), "coll".to_string(), 5)
        .await;
    assert!(result.is_err());
}

#[wasm_bindgen_test]
async fn ingest_missing_injection_errors() {
    let injection = js_sys::eval("({})").unwrap().dyn_into::<js_sys::Object>().unwrap();
    let config = js_sys::eval("({})").unwrap();
    let engine = xberg_wasm::engine::XbergEngine::new(config.into(), injection.into()).unwrap();

    let doc = js_sys::eval("({ text: 'hello' })").unwrap();
    let result = engine
        .ingest(doc, "coll".to_string())
        .await;
    assert!(result.is_err());
}
