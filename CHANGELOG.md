# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog and this project follows Semantic Versioning.

## [0.2.37] - 2026-11-28

### Fixed
- Retry transient upload-line probing failures for the current file up to three times, waiting 5 and 15 seconds before retrying.
- Recognize aggregate upload-line probe failures as transient so the existing batch retry mechanism can run after per-file retries are exhausted.
- Keep already uploaded parts within the current batch attempt while retrying the current file's line probe.

## [0.2.35] - 2026-11-23

### Added
- Bilibili upload templates support only-self visibility, with legacy templates remaining public.
- Visibility is saved in template and queued-task JSON and submitted through Biliup's native `is_only_self` parameter.

### Changed
- Refresh direct and transitive dependencies for Rust 1.100, including Dioxus 0.7.10, Reqwest 0.13.5, and Biliup 1.3.5.
- Pin Dioxus CLI to the matching framework version in release and container builds.
- Patch Biliup to quick-xml 0.42 and RSA 0.10, preserve RSS entity decoding, and remove RSA, instant, and pin-utils audit exceptions.

## [Unreleased]

### Added
- Project standardization baseline: CI, lint, formatting, contributor guide.
- Huya monitor bootstrap path from `server/config.json` into DB downloads.
- Download tasks can now be stopped from the web UI, which aborts active work and pauses automatic monitoring.
- Stopped download tasks can be resumed from the same table action area.
- Download task recording files can now be cleared from the web UI when the task is not active.
- GitHub Release automation now packages Web static files, Linux server bundles, PC desktop binaries, and optional Android artifacts.
- Release install/start/stop scripts support binary deployment on lightweight Linux servers.
- The Rust backend now runs as a Dioxus Fullstack server with SSR, Server Functions, and bundled Web assets.
- GitHub Release desktop artifacts now include Windows x64, macOS Intel, and macOS Apple Silicon download packages with bundled usage notes.

### Changed
- Stream checker error classification to distinguish offline from infra failures.
- Database initialization now auto-creates parent directories and file.
- Download task status now reflects disabled monitoring as `已停止` to make operator intent explicit.
- Web operation feedback now appears as a compact top overlay and auto-dismisses after five seconds.
- CI now checks the Dioxus Web WASM target in addition to Rust format, tests, Clippy, and dependency audit.
- Release and `start-bin.sh` deployments now run a single Fullstack backend process instead of a separate static Web process.
- Desktop Release packages now rename the executable to `OmniStream`/`OmniStream.exe` for easier end-user download and launch.
