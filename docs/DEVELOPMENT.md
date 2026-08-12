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
cargo test --locked --manifest-path astronomy_observer/Cargo.toml
```

Run the standalone runtime self-test after a release build:

```bash
astronomy-observer --self-test
```

The GitHub workflow also compiles the pinned Astronomy Engine helper and builds the Home Assistant image through the Home Assistant builder actions.

## Validation expectations

A change is not ready until the automated checks verify at least:

- repository/app YAML parses and required keys exist;
- the Home Assistant app version is valid and matches the default container build version;
- every documented dashboard entity is actually published by the runtime;
- the embedded and public dashboard presets are identical;
- documentation links point to existing files;
- the Ingress interface contains its required local endpoints and controls;
- Rust unit tests pass with the committed lock file;
- Clippy passes with warnings treated as errors;
- Rust formatting is clean;
- the C astronomy helper compiles against the pinned Astronomy Engine source;
- the multi-architecture Home Assistant container build succeeds;
- published images can be inspected without registry credentials and contain both supported architectures.

The Rust crate's package version is internal build metadata. The Home Assistant release version is the value in `astronomy_observer/config.yaml`; the container builder passes that release version into the image. Change the crate version only when the Cargo lock file is intentionally regenerated as part of the same development environment.

## Changing the score or target ranking

A scoring or ranking change should include:

1. the observational problem being fixed;
2. the changed formula, threshold or selection rule;
3. a test that demonstrates the intended behaviour;
4. an update to `docs/SCORING.md` when the calculation or ranking policy changes;
5. a changelog note when the change can materially alter observing decisions.

Do not tune a score only to make one favourite night look better. A change should improve a class of decisions and preserve understandable behaviour at the extremes: overcast must remain poor, daytime must not become a good deep-sky window, a target below the horizon must never be recommended, and an uncertain object class should not dominate the Top 10 simply because its geometry is easy to score.

## Adding a data source

Prefer sources that are public, documented, stable and usable without credentials. A new source should have:

- a clear owner/provider;
- documented terms of use;
- a bounded request cadence;
- timeout and cache behaviour;
- an explicit fallback or a safe unavailable state;
- no requirement to send more precise location than the data need.

## Release process

1. Update `version` in `astronomy_observer/config.yaml` and the default `BUILD_VERSION` in `astronomy_observer/Dockerfile`.
2. Update the changelog and any documentation affected by the behaviour change.
3. Run repository validation and Rust checks.
4. Commit the change and wait for CI and the multi-architecture image build.
5. Merge or publish only after all required checks pass.
6. The main-branch builder workflow publishes the versioned and `latest` images to GHCR.
7. Install or upgrade on a real Home Assistant system and review startup, Ingress, entity publication and memory use before promoting the app from experimental status.
