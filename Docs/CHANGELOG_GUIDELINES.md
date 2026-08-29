# Changelog Guidelines

All notable user-facing, developer-facing, operational, or security-relevant changes MUST be recorded in `CHANGELOG.md`.

The changelog follows the conventions of **Keep a Changelog 1.1.0** and, where versions are used, **Semantic Versioning 2.0.0**.

## Purpose

The changelog is written for humans.

It should explain the meaningful differences between versions of the project rather than reproduce Git history.

When modifying the project, determine whether the change is notable. If it is, update `CHANGELOG.md` as part of the same change.

## Changelog Structure

Use this general structure:

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog,
and this project adheres to Semantic Versioning.

## [Unreleased]

### Added

- Add project configuration discovery.

### Changed

- Improve validation errors when configuration is invalid.

### Fixed

- Prevent the application from crashing when the configuration file is missing.

## [1.2.0] - 2026-08-28

### Added

- Add support for loading configuration from environment variables.
```

## Unreleased Changes

All changes that have not yet been released MUST be placed under:

```markdown
## [Unreleased]
```

Do not create a new version number or release date unless explicitly performing a release.

When adding entries:

1. Locate the `Unreleased` section.
2. Determine the appropriate change category.
3. Add the entry to that category.
4. Create the category heading if it does not already exist.
5. Do not create empty category headings.

For example:

```markdown
## [Unreleased]

### Added

- Add interactive project configuration.

### Fixed

- Prevent duplicate configuration entries.
```

## Change Categories

Use only the following standard categories unless the project explicitly defines additional ones.

### Added

For new functionality.

Examples:

```markdown
### Added

- Add interactive project initialization.
- Add support for TOML configuration files.
- Add semantic search for previously discovered requirements.
```

### Changed

For changes to existing functionality.

Examples:

```markdown
### Changed

- Improve project discovery to prioritize workspace configuration.
- Update validation output to include the source of each finding.
```

### Deprecated

For functionality that remains available but is planned for removal.

Examples:

```markdown
### Deprecated

- Deprecate the legacy JSON configuration format in favor of TOML.
```

Whenever possible, state what users should migrate to.

### Removed

For functionality that has been removed.

Examples:

```markdown
### Removed

- Remove the deprecated `--legacy-config` command-line option.
```

### Fixed

For bug fixes.

Examples:

```markdown
### Fixed

- Prevent project discovery from scanning ignored directories.
- Preserve user answers when returning to a previous TUI step.
```

### Security

For changes related to vulnerabilities or security behavior.

Examples:

```markdown
### Security

- Prevent configuration files outside the workspace from being loaded automatically.
```

Do not include sensitive exploit details when doing so would create unnecessary risk.

## What Counts as a Notable Change

Add a changelog entry when a change affects one or more of the following:

- User-visible behavior.
- Public APIs.
- Command-line interfaces.
- Configuration formats or options.
- Data formats or database schemas.
- External integrations.
- Installation or deployment behavior.
- Performance in a meaningful way.
- Security.
- Compatibility.
- Dependencies when the dependency change affects users.
- Developer workflows when contributors need to know about the change.
- Deprecated functionality.
- Removed functionality.
- Important bug fixes.

Do not normally add entries for:

- Typographical corrections.
- Formatting-only changes.
- Comment-only changes.
- Internal refactoring with no observable behavior change.
- Test changes that only verify existing behavior.
- Routine dependency lockfile updates with no meaningful effect.
- Merge commits.
- Individual Git commits that are implementation details.

An internal change MAY still warrant an entry when it has meaningful consequences such as substantial performance, reliability, compatibility, or maintainability improvements.

## Writing Changelog Entries

Write entries from the perspective of the person using or maintaining the project.

Prefer describing **what changed and why it matters** over describing the implementation.

Good:

```markdown
- Add semantic search for previously answered project questions.
```

Less useful:

```markdown
- Add LanceDB code.
```

Good:

```markdown
- Prevent duplicate findings from appearing during project analysis.
```

Less useful:

```markdown
- Fix deduplication function.
```

Good:

```markdown
- Improve startup performance by avoiding repeated workspace scans.
```

Less useful:

```markdown
- Refactor scanner loop.
```

## Entry Style

Each changelog entry MUST:

- Be a Markdown bullet.
- Describe one logical change.
- Start with an imperative-style verb when practical.
- Be understandable without reading the commit or source code.
- Be concise while preserving meaningful context.
- Describe observable behavior rather than implementation details where possible.

Prefer:

```markdown
- Add automatic detection of Rust workspaces.
- Improve error messages for invalid configuration files.
- Fix project discovery when symbolic links are present.
```

Avoid:

```markdown
- Added stuff.
- Fixed bug.
- Refactored things.
- Updates.
- Miscellaneous improvements.
```

Do not end entries with issue identifiers alone.

Bad:

```markdown
- Fix #123.
```

Better:

```markdown
- Prevent configuration discovery from failing on empty workspaces (#123).
```

## Do Not Copy Commit Messages Blindly

Git history is an input to changelog creation, not the changelog itself.

Agents MUST NOT generate changelogs by simply copying:

```text
git log
```

or commit titles.

Several commits may represent one logical change and should therefore become one changelog entry.

For example, commits such as:

```text
add database
fix database initialization
add database tests
cleanup database API
```

may result in a single changelog entry:

```markdown
### Added

- Add persistent storage for discovered project requirements.
```

Conversely, one large commit may contain several independently notable changes and should produce multiple entries.

## Verify Changes Before Writing Entries

When generating or updating the changelog from repository changes, inspect the actual changes whenever possible.

Useful sources include:

```bash
git status
git diff
git diff --staged
git log
```

Use these sources to understand the change, but summarize the resulting behavior rather than reproducing their contents.

Do not claim functionality that cannot be verified from the repository changes or task context.

Do not invent:

- Features.
- Fixes.
- Breaking changes.
- Version numbers.
- Release dates.
- Issue numbers.
- Pull request numbers.

## Breaking Changes

Breaking changes MUST be made highly visible.

A breaking change includes changes that require users or integrations to modify existing usage, including:

- Removing a public API.
- Renaming CLI arguments.
- Changing configuration formats.
- Changing persisted data formats incompatibly.
- Changing API request or response structures incompatibly.
- Removing previously supported behavior.

Record the change under the most appropriate category and explicitly identify the compatibility impact.

Example:

```markdown
### Changed

- **Breaking:** Rename the `--config-path` option to `--config`. Existing scripts using `--config-path` must be updated.
```

If a replacement or migration path exists, include it.

Example:

```markdown
### Removed

- **Breaking:** Remove the deprecated JSON project configuration format. Convert existing configurations to TOML before upgrading.
```

Do not hide breaking changes behind vague descriptions such as:

```markdown
- Update configuration handling.
```

## Deprecations

Deprecations should appear in a release before the deprecated functionality is removed whenever practical.

Example:

```markdown
### Deprecated

- Deprecate `--json-config` in favor of `--config`. The old option will be removed in a future major release.
```

A later release can then contain:

```markdown
### Removed

- Remove the deprecated `--json-config` option.
```

## Security Changes

Security-relevant changes belong under:

```markdown
### Security
```

Describe the affected behavior and mitigation clearly without unnecessarily publishing exploitable implementation details.

Example:

```markdown
### Security

- Restrict configuration discovery to files inside the project workspace.
```

## Release Entries

Released versions use the following format:

```markdown
## [VERSION] - YYYY-MM-DD
```

For example:

```markdown
## [2.1.0] - 2026-08-28
```

Dates MUST use ISO `YYYY-MM-DD` format.

Released versions MUST appear in reverse chronological order, with the newest release first.

Example:

```markdown
## [Unreleased]

## [2.1.0] - 2026-08-28

## [2.0.1] - 2026-08-14

## [2.0.0] - 2026-08-01
```

## Preparing a Release

When explicitly asked to prepare a release:

1. Determine the intended version.
2. Determine the release date.
3. Move all relevant entries from `Unreleased` into the new version.
4. Preserve their category headings.
5. Remove empty headings.
6. Leave a new empty `Unreleased` section above the release.
7. Update version comparison links if the changelog maintains them.

Example:

Before:

```markdown
## [Unreleased]

### Added

- Add project templates.

### Fixed

- Prevent invalid templates from crashing initialization.

## [1.4.0] - 2026-08-01
```

After releasing `1.5.0`:

```markdown
## [Unreleased]

## [1.5.0] - 2026-08-28

### Added

- Add project templates.

### Fixed

- Prevent invalid templates from crashing initialization.

## [1.4.0] - 2026-08-01
```

Never assign a release version based solely on assumptions.

If the task does not explicitly involve creating a release, keep changes under `Unreleased`.

## Semantic Versioning

When the project follows Semantic Versioning:

```text
MAJOR.MINOR.PATCH
```

Interpret versions as:

- `MAJOR`: incompatible or breaking changes.
- `MINOR`: backward-compatible functionality.
- `PATCH`: backward-compatible bug fixes.

Do not change the project's version simply because a changelog entry was added.

Version selection is part of the release process.

## Yanked Releases

Do not delete a released version from the changelog if it is withdrawn.

Mark it as:

```markdown
## [1.4.1] - 2026-08-20 [YANKED]
```

When useful, briefly explain why the release was withdrawn.

## Linkable Versions

When the existing changelog uses version comparison links, preserve the convention.

Typical structure:

```markdown
[unreleased]: <comparison between latest release and HEAD>
[1.5.0]: <comparison between 1.4.0 and 1.5.0>
[1.4.0]: <comparison between 1.3.0 and 1.4.0>
```

Agents should update these links when preparing releases if the repository already maintains them.

Do not introduce a new link convention unless requested.

## Avoid Duplicate Entries

Before adding an entry, inspect the current `Unreleased` section for an existing entry describing the same logical change.

If one exists:

- Update it if the new work extends the same change.
- Do not add a nearly identical second entry.

For example, prefer:

```markdown
- Add project discovery for Cargo and Node.js workspaces.
```

over:

```markdown
- Add Cargo workspace discovery.
- Add Node.js workspace discovery.
```

when both are part of the same feature.

Separate entries are appropriate when changes are independently meaningful to users.

## Preserve Existing Project Conventions

These guidelines define the default behavior.

If the existing `CHANGELOG.md` has additional established conventions, preserve them unless they conflict with explicit task instructions.

Examples include:

- Issue or pull-request references.
- Component prefixes.
- Package-specific sections.
- Comparison links.
- Monorepo package headings.

Do not rewrite historical changelog entries merely to match newer wording or formatting unless specifically requested.

## Agent Checklist

Before completing any task containing notable changes, verify:

- [ ] I determined whether the change is changelog-worthy.
- [ ] I added notable unreleased changes under `## [Unreleased]`.
- [ ] I used the correct Keep a Changelog category.
- [ ] I described user-visible behavior rather than implementation details.
- [ ] I avoided copying commit messages blindly.
- [ ] I combined multiple commits representing one logical change.
- [ ] I called out breaking changes explicitly.
- [ ] I did not invent a version or release date.
- [ ] I did not add empty sections.
- [ ] I avoided duplicate changelog entries.
- [ ] I preserved existing changelog conventions.
- [ ] I verified that every changelog statement is supported by the actual change.

## Core Rule

When in doubt, optimize the changelog for someone asking:

> "What changed for me between these versions?"

The changelog should answer that question clearly without requiring the reader to inspect commits, pull requests, or source code.