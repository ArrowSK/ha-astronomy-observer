# Changelog

## 0.1.1 - 2026-08-12

- Reworked the Top 10 selection so satellites and comets cannot crowd out normal observing targets.
- Limited the final list to one satellite and one comet while allowing a broader mix of deep-sky objects, planets, meteor showers, the Moon, the Milky Way and aurora when relevant.
- De-weighted satellite passes when brightness is not modelled, with stronger penalties for rocket bodies and debris and a smaller penalty for familiar high-interest spacecraft.
- Excluded `A/`-designated asteroid-like objects from comet recommendations.
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
