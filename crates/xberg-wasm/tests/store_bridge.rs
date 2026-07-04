#![cfg(target_arch = "wasm32")]
use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn js_vector_store_round_trip() {
    let stub = js_sys::eval(
        r#"({
            ensureCollection: async (spec) => null,
            dropCollection: async (name) => null,
            getCollection: async (name) => null,
            upsertDocument: async (coll, doc, chunks) => ({ id: "doc-1" }),
            deleteDocuments: async (coll, ids) => 0,
            deleteByFilter: async (coll, filter) => 0,
            retrieve: async (coll, query) => ({
                mode: "vector",
                chunks: [{
                    chunk_id: "c1",
                    document_id: "doc-1",
                    ordinal: 0,
                    content: "hello world",
                    score: 0.95,
                    chunk_metadata: {},
                }],
                primary_latency_ms: 1,
            }),
            collectionStats: async (coll) => ({
                documents: 1,
                chunks: 1,
                last_ingested_at: null,
            }),
        })"#,
    )
    .unwrap()
    .dyn_into::<js_sys::Object>()
    .unwrap();

    let store = xberg_wasm::bridge::store::JsVectorStore::new("test".into(), stub);

    // ensure_collection
    let spec = xberg_rag::types::CollectionSpec::new("test", 2);
    store.ensure_collection(&spec).await.unwrap();

    // upsert_document
    let doc = xberg_rag::types::DocumentRecord {
        full_text: "hello world".into(),
        ..Default::default()
    };
    let chunk = xberg_rag::types::ChunkRecord {
        ordinal: 0,
        content: "hello world".into(),
        embedding: vec![0.1, 0.2],
        ..Default::default()
    };
    let id = store.upsert_document("test", &doc, &[chunk]).await.unwrap();
    assert_eq!(id.0, "doc-1");

    // retrieve
    let query = xberg_rag::query::RetrieveQuery {
        mode: xberg_rag::query::RetrieveMode::Vector,
        query_text: Some("hello".into()),
        top_k: 5,
        ..Default::default()
    };
    let output = store.retrieve("test", &query).await.unwrap();
    assert_eq!(output.chunks.len(), 1);
    assert_eq!(output.chunks[0].content, "hello world");

    // collection_stats
    let stats = store.collection_stats("test").await.unwrap();
    assert_eq!(stats.documents, 1);
}
