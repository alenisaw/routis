# Changelog

## [0.1.0] - Unreleased

First public CLI router release candidate.

### Added

- CLI entrypoint with `--policy`, `--dry-run`, `--execute`, `--help`, and `--version`
- policy presets for `default`, `cheap`, `balanced`, `deep`, and `extradeep`
- dynamic `default` behavior that resolves to an effective execution profile
- explainable rule-based classification and route selection
- thin Codex adapter that prepares and optionally executes the Codex CLI command
- local shell installation through `npm link`
- Windows install finalization so `routis` works as the primary PowerShell command after local install
- lightweight smoke tests for policy loading, routing, adapter planning, and failure handling

### Changed

- CLI output now shows requested policy, effective profile, model, reasoning, selected route, rationale, execution mode, and Codex command
- package payload is now limited to built CLI artifacts and policy configs for cleaner release packaging
