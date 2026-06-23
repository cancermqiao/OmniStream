# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog and this project follows Semantic Versioning.

## [Unreleased]

### Added
- Project standardization baseline: CI, lint, formatting, contributor guide.
- Huya monitor bootstrap path from `server/config.json` into DB downloads.
- Download tasks can now be stopped from the web UI, which aborts active work and pauses automatic monitoring.
- Stopped download tasks can be resumed from the same table action area.

### Changed
- Stream checker error classification to distinguish offline from infra failures.
- Database initialization now auto-creates parent directories and file.
- Download task status now reflects disabled monitoring as `已停止` to make operator intent explicit.
