# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Xberg is the next iteration of [Kreuzberg](https://github.com/kreuzberg-dev/kreuzberg-v4-lts).
The changelog starts fresh at `1.0.0-rc.1`. For the Kreuzberg v1–v4 history, see the
[Kreuzberg v4 LTS changelog](https://github.com/kreuzberg-dev/kreuzberg-v4-lts/blob/main/CHANGELOG.md).

---

## [Unreleased]

### Added

- **Durable rehydration-map storage.** `POST /v1/process` (with
  `operations.redact.rehydrate=true`) and `POST
  /v1/documents/{id}/rehydrate` now persist encrypted PII rehydration maps
  through a new `xberg-doc-store` crate. The default backend is unchanged
  (in-memory, 24h TTL, lost on restart); setting `XBERG_REHYDRATION_DB_PATH`
  and building with the `doc-store-sqlite` feature switches to a durable,
  WAL-mode SQLite backend that survives process restarts. No wire-format
  change to either endpoint.

---

## [1.0.0-rc.1] - 2026-06-26

Initial Xberg release candidate. Xberg continues the Kreuzberg document-intelligence
engine under a new name with a reset v1 version line. This is a full rebrand with no
back-compat aliases; the published `kreuzberg` packages remain frozen on the v4 LTS line.

### Changed

- **Rebranded Kreuzberg → Xberg.**
  - **Rust:** crate `kreuzberg` → `xberg` (and every `kreuzberg-*` workspace crate →
    `xberg-*`); the `kreuzberg::` namespace → `xberg::`; `KreuzbergError` → `XbergError`.
  - **CLI:** binary `kreuzberg` → `xberg`; config discovery `kreuzberg.{toml,yaml,json}` →
    `xberg.{toml,yaml,json}`; all `KREUZBERG_*` environment variables → `XBERG_*`; cache
    directory `.kreuzberg/` → `.xberg/`.
  - **FFI:** symbol prefix `kreuzberg_*` → `xberg_*`; header `kreuzberg.h` → `xberg.h`; lib
    `kreuzberg_ffi` → `xberg_ffi`.
  - **Package coordinates:** PyPI `xberg`, npm `@xberg-io/*`, RubyGems/Hex/pub.dev `xberg`,
    Maven `io.xberg`, NuGet `Xberg`, Packagist `xberg-io/xberg`, Homebrew `xberg`.
  - **Go:** module `github.com/xberg-io/xberg` with no `/vN` suffix (v1); the binding lives at
    `packages/go/`.
  - **Docs:** documentation now at `docs.xberg.io`.
- **ner-onnx:** vendored the stripped span-mode GLiNER runtime as `xberg-gliner`, replaced the
  ORP pipeline wrapper with direct `ort` session management, and moved runtime model downloads
  to the `xberg-io/gliner-models` artifact repository. The public `ner-onnx` feature and NER
  config shape are unchanged.

[1.0.0-rc.1]: https://github.com/xberg-io/xberg/releases/tag/v1.0.0-rc.1
