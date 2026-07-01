# LoRA Privacy API Plan — SDD Progress
# Plan: docs/superpowers/plans/2026-06-30-lora-privacy-api.md
# Started: 2026-06-30
# Branch: feature/gliner2-onnx-backend
# Base commit (before Task 1): cbe1c32b66
Task 1: complete (commits cbe1c32b66..d2b5e861db, review clean)
Task 2: complete (commits d2b5e861db..13960ccc74, review clean)
Task 3: complete (commits 13960ccc74..14a311abab, review clean — Minor: stubs for Tasks 4-7 in same commit)
Task 4: complete (commits 14a311abab..a3ab225d89, review clean)
Task 5a: complete (commits a3ab225d89..fbe60234a8, review clean)
Task 5b: complete (commits fbe60234a8..835caecf56, review clean — two non-blocking notes: dead device param in count_lstm::forward per spec, relu-after-cat in span_rep matches anno source)
Task 6: complete (commits 835caecf56..d43ee3fxxx, reviewer found end_word clamp bug + .expect() in lib code, fixed in follow-up commit d43ee3f)
Task 7: complete (commits bae96115ac..8c06f81, reviewer found i64/u32 index_select mismatch, fixed in follow-up 8c06f81)
Task 8: complete (commit 1674cba, 10 tests pass, smoke test #[ignore]d — Part 1 (xberg-gliner-candle engine crate) done)
Task 9: complete (commit 33cbca6, ignored GLiNER2 ONNX smoke test added to gline.rs)
Task 10: complete (already done in prior session — GlinerArchitecture/hfArchitecture in xberg-node, no new commit needed)
Task 11 (brief): complete (commit 578af53 — MCP hf_architecture wiring in intelligence.ts/ingest.ts/README)
Task 11 (plan): complete (commit f64e31c — encrypted rehydration map + RehydrationStore; 8 redaction regression + 7 rehydration/store tests pass)
Task 12: complete (commit 9d506d9f23 — POST /v1/process, 3 handler tests pass; agent also applied fmt/bindings/docs/Cargo.lock fixups)
Task 13: complete (commit 680b341 — POST /v1/documents/{id}/rehydrate, 3 tests pass; reviewer findings were false positives caused by agent being on wrong branch — correct branch (feature/gliner2-onnx-backend) already had moka TTL store and no RwLock::expect; non-blocking: encrypt_map uses XbergError::validation for internal crypto errors, noted for follow-up)
# PLAN COMPLETE — all 13 tasks shipped on feature/gliner2-onnx-backend
