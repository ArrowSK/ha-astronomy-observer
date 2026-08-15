# Development and validation

Astronomy Observer has several ways to run, but changes should still behave like changes to one product. The shared Rust modules and shared interface are the centre; Home Assistant, web/Docker and Android add only the platform-specific edges they need.

## Repository layout

```text
astronomy_observer/       Home Assistant app + shared Rust/runtime source
  src/                    astronomy, weather, scoring, targets and source handling
  scripts/                build-time C helper/catalogue tooling
  data/                   meteor table + compact World Atlas derivative
  web/                    shared observing interface
  dashboard/              optional HA dashboard preset
  config.yaml             HA app manifest
  DOCS.md                 Home Assistant user guide
webapp/                    standalone HTTP/Docker adapter
android/                   standalone Android shell, JNI adapter and APK build
  app/                     Java activity/WebView shell
  native-rust/             Android cdylib that compiles shared Rust modules
  native/                  JNI + embedded Astronomy Engine adapter
  ui/                      small Android-specific UI adapter
  generated/               build output; never committed
dashboard/                 public copy of the HA dashboard preset
docs/                      user/design/source documentation
tests/                     repository validation
tools/                     optional desktop-side data preparation tools
```

## Normal local checks

From the repository root:

```bash
python3 tests/validate_repository.py
python3 webapp/validate.py
python3 android/validate.py
python3 tests/validate_atlas.py
cargo test --locked --manifest-path astronomy_observer/Cargo.toml
cargo clippy --locked --manifest-path astronomy_observer/Cargo.toml --all-targets -- -D warnings
```

A normal non-Android Rust release can be checked with:

```bash
cargo build --locked --manifest-path astronomy_observer/Cargo.toml --release
astronomy_observer/target/release/astronomy-observer --self-test
```

The CI workflow additionally builds/smoke-tests the standalone Docker image and compiles the pinned Astronomy Engine helper.

## Android build

The Android build is intentionally reproducible from source and does not commit generated native libraries or APK files.

Build prerequisites are documented in [`../android/README.md`](../android/README.md). In short, CI uses JDK 17, Gradle 8.13, Android platform 36, build-tools 35.0.0, NDK 27.0.12077973, Rust 1.89 with the Android targets, Python 3 and curl.

The full preview build is:

```bash
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/27.0.12077973"
./android/build-apk.sh
```

That build performs four important things rather than wrapping a remote URL:

1. builds the reduced OpenNGC observing catalogue from the pinned source;
2. compiles the pinned Astronomy Engine C code and shared Rust core into Android native libraries;
3. prepares the Android interface from the shared HTML plus the small Android adapter;
4. assembles and lints an installable debug-signed APK.

The dedicated Android Actions workflow also inspects the APK to ensure both supported ABIs and the offline catalogue/atlas/licence assets are really inside it.

## Validation expectations

A change is not ready until the relevant checks establish at least:

- repository/app YAML parses and required keys exist;
- HA manifest and default HA container version agree;
- every documented HA dashboard entity is actually published;
- embedded/public HA dashboard presets remain identical;
- documentation links point to existing files;
- the shared interface retains required controls/endpoints;
- standalone web packaging still compiles shared runtime modules;
- Android packaging still compiles the shared runtime modules rather than a forked scoring implementation;
- Android has no hard-coded Astronomy Observer server dependency;
- Android WebView remote loading safeguards remain in place;
- Android contains the necessary catalogue/atlas and licence material;
- Rust unit tests, formatting and Clippy pass;
- the pinned C astronomy source compiles;
- the standalone Docker image builds and passes its smoke test;
- the Home Assistant multi-architecture container build succeeds;
- the Android APK assembles and its package/native/assets structure is inspected.

The Rust crate's package version is internal build metadata. Product release versions are kept in the HA manifest/container and Android package. Change the internal crate version only when the lock file is intentionally regenerated in a suitable development environment.

## Keep platform differences narrow

Do not copy the scoring or target-ranking code into `android/` or `webapp/`. Both adapters deliberately compile modules from `astronomy_observer/src`.

When a platform genuinely needs different behavior, make the boundary explicit. Current examples are:

- HA obtains location/entities through Home Assistant;
- web obtains an explicit request location;
- Android obtains GPS/manual location from its local shell;
- HA/web can launch the Astronomy Engine helper process;
- Android links the same Astronomy Engine C implementation into its native library;
- Android can fall back to an explicitly stale unknown-weather planning snapshot when completely offline, while HA/web preserve the existing fresh-weather requirement.

Do not use a platform problem as a reason to redesign unrelated shared behavior.

## Changing scores or target ranking

A scoring/ranking change should include the observational problem, changed formula/threshold/selection rule, a test for the intended behavior, an update to `docs/SCORING.md` when appropriate, and a changelog note if observing decisions can materially change.

Do not tune a score simply to make one favourite night look better. Overcast must remain poor, daytime must not become a good deep-sky window, targets below the horizon must not be recommended, and uncertain object classes must not dominate simply because their geometry is easy to score.

## Adding a data source

Prefer public, documented sources usable without credentials. A source should have a clear provider, documented terms, bounded request cadence, timeout/cache behavior, explicit failure state and no requirement to send more precise location than it needs.

Licensing/attribution is part of the implementation. If a source requires attribution or change notices, update `THIRD_PARTY_LICENSES.md` and any distributable in-app notice at the same time. Do not assume the project licence can be applied to third-party data.

## Android signing

Every installable APK is signed. Development/CI preview builds use Android debug signing so they can be sideloaded without a Play Store release.

Do **not** put a permanent release signing key in the repository or a downloadable build artifact. If public sideload releases are introduced, keep one stable private release key in protected release infrastructure and back it up securely. Changing that key breaks the normal Android update path for users who installed an earlier release signed by the old key.

## Release process

1. Update the shared product version where required (`astronomy_observer/config.yaml`, HA Docker default and Android package version).
2. Update changelogs and user documentation for behavior/platform changes.
3. Run repository, web, Android and Rust validation.
4. Let CI build/smoke-test Docker and build/inspect the Android preview APK.
5. Let the Home Assistant builder validate both supported container architectures.
6. Merge only when the relevant checks are green.
7. Main publishes the Home Assistant container through the established builder workflow. Android preview artifacts remain CI artifacts unless/until a stable project-controlled release signing process is deliberately added.
8. Before promoting the experimental status, test each distributed edition on real target hardware rather than relying only on CI.
