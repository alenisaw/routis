# Changelog

## [0.1.1] - Unreleased

### Fixed

- corrected the release packaging path after the incomplete `v0.1.0` tag content
- fixed installability for standard release packaging and npm publication
- aligned package metadata for public distribution and release verification

### Changed

- moved the published package name to `@alenisaw/routis` while keeping the CLI command name as `routis`
- added publish-ready npm metadata and public access configuration
- added publish-time verification so release builds and smoke tests run before npm publish

## [0.1.0] - Released

First public CLI router release candidate.

### Added

- CLI entrypoint with `--policy`, `--dry-run`, `--execute`, `--help`, and `--version`
- policy presets for `default`, `cheap`, `balanced`, `deep`, and `extradeep`
- dynamic `default` behavior that resolves to an effective execution profile
- explainable rule-based classification and route selection
- thin Codex adapter that prepares and optionally executes the Codex CLI command
- local shell installation through `npm link`
- lightweight smoke tests for policy loading, routing, adapter planning, and failure handling

### Changed

- CLI output now shows requested policy, effective profile, model, reasoning, selected route, rationale, execution mode, and Codex command
- package payload is now limited to built CLI artifacts and policy configs for cleaner release packaging
