# Changelog

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
