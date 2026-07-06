pub mod embedder;
pub mod ner;
pub mod ocr;
pub mod store;

use wasm_bindgen::prelude::*;

const BRIDGE_TIMEOUT_MS: u32 = 30_000;

/// Wrap a JS `Promise` with a timeout.
///
/// If the promise does not resolve within `ms` milliseconds, the returned
/// promise rejects with an `Error("bridge call timed out")`.  Uses
/// `Promise.race` under the hood so the original promise is still cancellable.
pub fn with_timeout(promise: js_sys::Promise, ms: u32) -> js_sys::Promise {
    let racer = js_sys::Function::new_with_args(
        "p, ms",
        "return Promise.race([
            p,
            new Promise((_, reject) =>
                setTimeout(() => reject(new Error('bridge call timed out')), ms)
            )
        ])",
    );
    match racer.call2(&promise, &JsValue::from(ms)) {
        Ok(val) => val.into(),
        Err(_) => promise,
    }
}

/// Convenience wrapper: create a timed-out `JsFuture` from a `Promise`.
pub fn timed_js_future(promise: js_sys::Promise) -> wasm_bindgen_futures::JsFuture {
    wasm_bindgen_futures::JsFuture::from(with_timeout(promise, BRIDGE_TIMEOUT_MS))
}
