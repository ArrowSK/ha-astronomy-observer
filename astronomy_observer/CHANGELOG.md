# Changelog

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
