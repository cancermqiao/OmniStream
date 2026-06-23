# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog and this project follows Semantic Versioning.

## [Unreleased]

### Added
- Project standardization baseline: CI, lint, formatting, contributor guide.
- Huya monitor bootstrap path from `server/config.json` into DB downloads.
- Download tasks can now be stopped from the web UI, which aborts active work and pauses automatic monitoring.
- Stopped download tasks can be resumed from the same table action area.
- Download task recording files can now be cleared from the web UI when the task is not active.
- GitHub Release automation now packages Web static files, Linux server bundles, PC desktop binaries, and optional Android artifacts.
- Release install/start/stop scripts support binary deployment on lightweight Linux servers.
- The Rust backend can now serve the Dioxus Web UI directly from `BILIUP_WEB_DIR`.

### Changed
- Stream checker error classification to distinguish offline from infra failures.
- Database initialization now auto-creates parent directories and file.
- Download task status now reflects disabled monitoring as `已停止` to make operator intent explicit.
- Web operation feedback now appears as a compact top overlay and auto-dismisses after five seconds.
- CI now checks the Dioxus Web WASM target in addition to Rust format, tests, Clippy, and dependency audit.
- Release and `start-bin.sh` deployments now run a single backend process instead of a separate Python Web server.
