# Changelog

## 0.3.4 - 2026-08-22

- Kept bundled target thumbnails at 192 px so Home Assistant, Docker and Android packages do not grow merely to improve the expanded view.
- Home Assistant and standalone web now request a larger licence-verified Wikimedia Commons preview only when a target image is opened; the local image remains the immediate/offline fallback.
- Deep-sky/NGC/IC targets, comets and the ISS can acquire a small on-demand preview when they previously had only the generic astronomy marker, with at most two thumbnail lookups in flight.
- Runtime images use the same Public Domain / CC0 / CC BY / CC BY-SA allow-list as bundled images and send target titles or image filenames, not observing coordinates, to Wikimedia.
- Standalone Android keeps its hardened local-only WebView network boundary and therefore continues to use bundled images and the normal fallback marker offline.

## 0.3.3

- Target thumbnails now open in a mobile-safe expanded view with an explicit close control.
- Expanded images show an image-credit and licence link directly below the image; the separate list-level **i** button remains available without stealing the image tap target.
- The image-credits page now uses an icon-only return control while keeping its accessible label.

## 0.3.2 - 2026-08-18

- Added an always-visible **Back to Astronomy Observer** control to object-image credits, including deep-linked entries opened inside Home Assistant Ingress.
- Wikimedia source and licence links now open separately instead of replacing the embedded Astronomy Observer view.
- Kept all target images, image licences, target ranking, scoring and observing behaviour unchanged.

## 0.3.1 - 2026-08-17

- Added small offline target thumbnails for Messier objects plus familiar planets, the Moon, the Milky Way and major meteor showers.
- Images are selected at build time from Wikimedia's representative free images and then licence-checked against Wikimedia Commons metadata; only Public domain, CC0, CC BY and CC BY-SA files are accepted.
- Added per-image creator/source/licence credits inside every installation; third-party images remain under their original licences rather than the project PolyForm licence.
- Targets without a verified bundled image keep the existing ranking/list behaviour and show a neutral fallback marker, so image coverage never changes observing recommendations.
- Kept the thumbnails fully local in Home Assistant, Docker/web and Android; rendering the target list does not depend on Wikipedia, Wikimedia Commons or an Astronomy Observer server.

## 0.3.0 - 2026-08-15

- Added a self-contained Android edition that can be sideloaded without Home Assistant or an Astronomy Observer server.
- Android packages the shared interface, Rust observing engine, pinned Astronomy Engine C code, reduced OpenNGC observing catalogue, meteor-shower table and compact World Atlas light-pollution grid inside the APK.
- Added optional phone-location access plus manual site coordinates; exact observing coordinates remain local for astronomical calculations.
- Kept the Android WebView local-only: remote page/subresource loading is blocked and explicit external links open in the normal browser.
- Added a narrow Java/JNI/native bridge so Android compiles the same Rust scoring, target-ranking, weather, light-pollution, comet, satellite and aurora modules as the Home Assistant and Docker editions rather than maintaining a second calculation implementation.
- Added an Android-only offline planning fallback: if live weather and its recent cache are both unavailable, local astronomy can still be calculated with weather fields unknown and confidence intentionally reduced.
- Added app-local observation journal persistence and retained the existing search/filter/delete interface behavior on Android.
- Added reproducible Android build tooling for `arm64-v8a` and `x86_64`, APK inspection checks and a GitHub Actions preview artifact.
- Added in-app **About & licences** notices and clarified that OpenNGC-derived data remain CC BY-SA 4.0, the World Atlas derivative remains CC BY-NC 4.0 and Astronomy Engine remains MIT rather than being relicensed under the project licence.
- Expanded the main documentation to present Android, Home Assistant, Docker and Railway/container hosting as first-class ways of using the same observing engine.

## 0.2.3 - 2026-08-13

- Renamed the bottom navigation label from **Outlook** to **Forecast** while keeping the seven-night section and its internal anchor compatible.
- Added per-entry delete controls to the Observation journal.
- Added list-level multi-selection, **Select visible**, and **Delete selected** for removing several journal entries together.
- Added confirmation prompts before every journal deletion and an authenticated Ingress `DELETE /api/observations` endpoint for permanent local removal.
- Journal deletion rewrites the local JSONL file atomically and preserves unrelated or unparsable lines rather than silently discarding them.
- Added a runtime regression test for selective observation deletion and updated repository validation/documentation.

## 0.2.2 - 2026-08-13

- Added a persistent Android-style bottom navigation bar for quick jumps between Tonight, Conditions, Targets, Outlook and Sources.
- Added active-section highlighting to the bottom navigation while scrolling and safe-area padding so it does not sit under the system gesture area.
- Reworked the header so manual refresh remains directly accessible while Setup and the Observation journal live in a spaced hamburger menu.
- Replaced the observation pencil with a document-style journal icon.
- Increased header, menu and bottom-navigation touch targets and spacing to reduce accidental taps.
- Increased the text size and line spacing of expandable condition explanations and advanced Setup explanations.
- Kept Setup auto-close-on-save, observation search/filtering, condition scoring, target ranking and Home Assistant entities unchanged.
- Extended repository validation to require the new menu and bottom-navigation markers.

## 0.2.1 - 2026-08-12

- Reworked **Conditions in the best window** into compact label/value lists instead of tiles.
- Visually separated observing-quality scores from forecast measurements while preserving the existing scoring and raw data.
- Made every condition row expandable directly below the row with a concise explanation of how the value is interpreted or used.
- Added the Falchi/GFZ World Atlas reference and current light-pollution source details to the sky-brightness explanation.
- Replaced the text Setup and Refresh controls with compact cogwheel and refresh icons.
- Moved **Copy dashboard YAML** into Setup because it is normally only needed during dashboard creation.
- Made a successful Setup save close the panel automatically while the recalculation continues.
- Moved Observing Notes behind a toolbar icon and made observation history collapsible by default.
- Added client-side history search plus metric-presence and time-range filters across the retained observation history.
- Added green, amber and red source-status dots for current/local, cached/fallback/disabled and unavailable/failed sources.
- Added static repository checks for the new condition-list, notes/history and source-status interface markers.

## 0.2.0 - 2026-08-12

- Added a built-in approximately 3-arcminute global light-pollution grid derived from the Falchi World Atlas 2015 dataset.
- Light pollution now follows the selected Home Assistant person or Home location automatically; no CSV is required for normal use.
- The location-based sky-brightness estimate feeds the existing darkness component, so the headline, deep-sky and imaging scores reflect the observer's light pollution from the first calculation.
- Kept fixed SQM values, Home Assistant SQM sensors and user-supplied local CSV grids as higher-priority overrides for observers with better local data.
- Extended the nearby darker-area search to work with the bundled atlas without manual setup.
- Added a compact binary atlas reader that reads only the required cell or nearby rows instead of loading the global grid into memory.
- Added reproducible atlas-generation tooling, source metadata, checksum validation and third-party data attribution.

## 0.1.1 - 2026-08-12

- Reworked the Top 10 selection so satellites and comets cannot crowd out normal observing targets.
- Limited the final list to one satellite and one comet while allowing a broader mix of deep-sky objects, planets, meteor showers, the Moon, the Milky Way and aurora when relevant.
- De-weighted satellite passes when brightness is not modelled, with stronger penalties for rocket bodies and debris and a smaller penalty for familiar high-interest spacecraft.
- Excluded `A/`-designated asteroid-like objects from comet recommendations.
- Fixed the OpenNGC catalogue build so J2000 right ascension and declination are converted from OpenNGC sexagesimal values into the decimal-degree format used by the runtime, and the magnitude, surface-brightness and size fields are mapped to the correct columns.
- Fixed the bundled meteor-shower CSV parser so major showers, including the Perseids, are actually loaded and ranked.
- Added regression/self-tests for the OpenNGC coordinate conversion and meteor-shower CSV schema.
- Added a Home Assistant person selector to the Ingress Setup panel and persisted the selection without requiring a restart.
- Simplified horizon setup to a single lowest-useful-altitude choice for normal use while keeping the directional mask under an advanced section.
- Clarified that the Falchi CSV is optional and that the app works without any light-pollution file.
- Added a dedicated sky-brightness tile and plain-language light-pollution status in Setup.
- Expanded the seven-night Ingress outlook with deep-sky, planetary, imaging, clear-sky, transparency, Moon-impact and confidence values.
- Made the condition tiles interactive so each one explains what it represents and how it is used.
- Kept the existing Home Assistant outlook attributes and dashboard entities compatible with the initial release.

## 0.1.0 - 2026-08-12

- Initial public release.
- Home Assistant person-aware location with Home fallback.
- Open-Meteo forecast and atmospheric inputs with MET Norway weather fallback and bounded cache.
- Local Sun, Moon and planetary calculations through pinned Astronomy Engine code.
- Separate overall, deep-sky, planetary and imaging condition scores with published components and confidence.
- Best two-hour observing window and seven-night outlook.
- Deep-sky ranking from a pinned OpenNGC-derived observing catalogue.
- Milky Way / Galactic Centre recommendation.
- Major meteor-shower ranking.
- Minor Planet Center comet elements with local propagation and bounded cache fallback.
- CelesTrak visible-satellite pass search with local SGP4 propagation and bounded cache fallback.
- NOAA OVATION aurora input with short-lived cache fallback.
- Fixed SQM, Home Assistant SQM sensor and local Falchi-atlas grid support.
- Nearby darker-point search from the local grid.
- User-defined horizon mask and telescope/binocular aperture gates.
- Home Assistant state publishing and update event.
- Local observing notes for SQM, seeing, transparency, limiting magnitude and forecast comparison.
- Admin-only Ingress interface and dependency-free native dashboard preset.
- Multi-architecture GitHub Actions build and repository validation.
