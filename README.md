# Astronomy Observer for Home Assistant

Astronomy Observer is a Home Assistant app for deciding whether a night is worth observing and what is worth looking at. It can follow a Home Assistant `person` entity, evaluates the sky and forecast at that location, and ranks up to ten worthwhile targets for the night ahead.

The app is aimed mainly at experienced amateur astronomers. It keeps the underlying factors visible instead of hiding them behind a single weather badge: cloud layers, transparency, estimated seeing, darkness, Moon interference, wind, dew margin and data confidence are all available separately. Target ranking also considers altitude, airmass, the local horizon, Moon separation, sky brightness, object type and the configured aperture.

The runtime is a small Rust service with a C astronomy helper. It is designed for Home Assistant hosts where memory and background CPU use matter.

## Main features

- Home Assistant `person` location, with Home coordinates as a fallback.
- Person selection directly from the built-in Setup panel.
- Seven-night observing outlook with deep-sky, planetary, imaging, clear-sky, transparency, Moon-impact and confidence detail.
- Best two-hour observing window for tonight.
- Total, low, middle and high cloud layers.
- Visibility, humidity, dew point, aerosol optical depth, surface wind and upper-air wind.
- Separate overall, deep-sky, planetary and imaging scores.
- Grouped condition lists that visually separate observing-quality scores from forecast measurements.
- Every condition row expands directly below itself with a plain-language explanation of the calculation or interpretation.
- Local Sun, Moon and planet calculations.
- Target-specific Moon separation and altitude penalties.
- Deep-sky catalogue built from a pinned OpenNGC snapshot.
- Milky Way / Galactic Centre opportunity.
- Major meteor showers.
- Potentially observable active comets from Minor Planet Center elements.
- Visible-satellite passes from CelesTrak orbital elements, with unknown brightness deliberately de-weighted.
- Category-aware Top 10 selection so satellites and comets cannot crowd out the normal observing programme.
- NOAA OVATION aurora probability.
- Simple lowest-useful-altitude horizon setup, with an optional advanced directional mask.
- Telescope and binocular aperture filtering.
- Automatic location-based light-pollution estimate from the bundled World Atlas derivative, with the Falchi/GFZ reference visible in the interface.
- Fixed SQM, Home Assistant SQM sensor and higher-resolution local CSV overrides for observers with better local data.
- Nearby darker-area search from the built-in atlas without manual setup.
- Collapsible local observing history with search, metric filters and time filters.
- Source-status indicators with green, amber and red states for current/local, cached/fallback and unavailable sources.
- Home Assistant entities for dashboards and automations.
- Built-in Ingress view and a dependency-free native dashboard preset.

## Installation

In Home Assistant, add this repository to the app store:

`https://github.com/ArrowSK/ha-astronomy-observer`

Install **Astronomy Observer** and start it. The default configuration uses Home Assistant's Home coordinates. After the first successful refresh, open Astronomy Observer and open the hamburger menu, choose **Setup**, and select a Home Assistant person and the lowest useful altitude for the observing site. Saving closes Setup automatically and triggers a recalculation.

The header keeps manual refresh directly accessible and places **Setup** and the **Observation journal** in a spaced hamburger menu. The journal uses a document icon rather than an edit/pencil symbol. A persistent bottom navigation bar jumps between Tonight, Conditions, Targets, Outlook and Sources, with the current section highlighted while scrolling. The dashboard YAML copy action remains inside Setup because it is normally only needed when creating the optional native dashboard.

No light-pollution file is required. Astronomy Observer looks up the selected location in its bundled approximately 3-arcminute World Atlas grid and uses the resulting sky-brightness estimate in the initial overall, deep-sky and imaging scores. A real SQM sensor, a fixed SQM value or a higher-resolution local grid can still override that estimate.

The full setup and operating guide is in [`astronomy_observer/DOCS.md`](astronomy_observer/DOCS.md).

## Documentation

- [Installation, configuration and Home Assistant entities](astronomy_observer/DOCS.md)
- [Scoring method](docs/SCORING.md)
- [Data sources, caching and fallbacks](docs/DATA_SOURCES.md)
- [Light pollution and sky brightness](docs/LIGHT_POLLUTION.md)
- [Privacy and location handling](docs/PRIVACY.md)
- [Architecture and resource budget](docs/ARCHITECTURE.md)
- [Local Ingress endpoints](docs/API.md)
- [Scientific and technical references](docs/SCIENTIFIC_REFERENCES.md)
- [Development and validation](docs/DEVELOPMENT.md)
- [Known limits and roadmap](docs/LIMITS_AND_ROADMAP.md)

## Licence

Project code is licensed under the **PolyForm Noncommercial License 1.0.0**. It may be used, changed and redistributed for permitted non-commercial purposes. Commercial use of this code or a derivative is not licensed. The intention is straightforward: the project should remain freely usable by amateur astronomers, clubs, education and non-commercial research, rather than becoming the basis of a paid wrapper or service.

This is a source-available non-commercial licence, not an OSI open-source licence.

Third-party code and data keep their own licences. The bundled World Atlas derivative is separately covered by CC BY-NC 4.0. See [`THIRD_PARTY_LICENSES.md`](THIRD_PARTY_LICENSES.md).

## Release status

`0.2.3` remains marked experimental in Home Assistant. The calculations are deliberately explicit about uncertainty. Estimated seeing is a weather-derived proxy rather than a forecast in arcseconds, recurring meteor-shower dates are a planning aid rather than a substitute for the current IMO calendar, satellite brightness is not inferred from orbital geometry alone, and the bundled light-pollution atlas is a 2015 planning baseline rather than a live sky-quality measurement.
