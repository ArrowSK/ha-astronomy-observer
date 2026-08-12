# Development and validation

## Repository layout

```text
astronomy_observer/       Home Assistant app and runtime source
  src/                    Rust service
  scripts/                build-time C helper and catalogue builder
  data/                   small static observing tables
  web/                    Ingress interface
  dashboard/              dashboard YAML embedded in the Ingress page
  config.yaml             Home Assistant app manifest
  DOCS.md                 user documentation shown by Home Assistant
dashboard/                 public copy of the dashboard preset
docs/                      design and source documentation
tests/                     repository validation
tools/                     optional desktop-side data preparation tools
```

## Local checks

Run the repository validator from the repository root:

```bash
python3 tests/validate_repository.py
```

Run Rust tests:

```bash
cargo test --manifest-path astronomy_observer/Cargo.toml
```

Run the standalone runtime self-test after a release build:

```bash
astronomy-observer --self-test
```

The GitHub workflow also compiles the pinned Astronomy Engine helper and builds the Home Assistant image through the Home Assistant builder actions.

## Validation expectations

A pull request is not ready to merge until the automated checks verify at least:

- repository/app YAML parses and required keys exist;
- app version and Rust package version agree;
- every documented dashboard entity is actually published by the runtime;
- the embedded and public dashboard presets are identical;
- documentation links point to existing files;
- Rust unit tests pass;
- Clippy passes with warnings treated as errors;
- Rust formatting is clean;
- the C astronomy helper compiles against the pinned Astronomy Engine source;
- the multi-architecture Home Assistant container build succeeds.

## Changing the score

A scoring change must include:

1. the observational problem being fixed;
2. the changed formula or threshold;
3. a test that demonstrates the intended behaviour;
4. an update to `docs/SCORING.md`;
5. a note in the changelog when the change can materially alter observing decisions.

Do not tune a score only to make one favourite night look better. A change should improve a class of decisions and preserve understandable behaviour at the extremes: overcast must remain poor, daytime must not become a good deep-sky window, and a target below the horizon must never be recommended.

## Adding a data source

Prefer sources that are public, documented, stable and usable without credentials. A new source should have:

- a clear owner/provider;
- documented terms of use;
- a bounded request cadence;
- timeout and cache behaviour;
- an explicit fallback or a safe "unavailable" state;
- no requirement to send more precise location than the data need.

## Release process

1. Update `version` in `astronomy_observer/config.yaml` and `astronomy_observer/Cargo.toml`.
2. Update both changelogs.
3. Run local validation.
4. Open a pull request and wait for CI and the multi-architecture image build.
5. Merge only after all required checks pass.
6. The main-branch builder workflow publishes the versioned and `latest` images to GHCR.
7. Install or upgrade on a real Home Assistant system and review startup, Ingress, entity publication and memory use before promoting the app from experimental status.
