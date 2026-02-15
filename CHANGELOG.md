# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog and this project follows Semantic Versioning.

## [Unreleased]

### Added
- Project standardization baseline: CI, lint, formatting, contributor guide.
- Huya monitor bootstrap path from `server/config.json` into DB downloads.

### Changed
- Stream checker error classification to distinguish offline from infra failures.
- Database initialization now auto-creates parent directories and file.
