# Astronomy Observer for Home Assistant

Astronomy Observer is a Home Assistant app for deciding whether a night is worth observing and what is worth looking at. It follows a Home Assistant `person` entity when one is configured, evaluates the sky and forecast at that location, and ranks up to ten targets for the night ahead.

The app is aimed mainly at experienced amateur astronomers. It keeps the underlying factors visible instead of hiding them behind a single weather badge: cloud layers, transparency, estimated seeing, darkness, Moon interference, wind, dew margin and data confidence are all available separately. Target ranking also considers altitude, airmass, the local horizon, Moon separation, sky brightness, object type and the configured aperture.

The runtime is a small Rust service with a C astronomy helper. It is designed for Home Assistant hosts where memory and background CPU use matter.

## Main features

- Home Assistant `person` location, with Home coordinates as a fallback.
- Seven-night observing outlook and a best two-hour window for tonight.
- Total, low, middle and high cloud layers.
- Visibility, humidity, dew point, aerosol optical depth, surface wind and upper-air wind.
- Separate overall, deep-sky, planetary and imaging scores.
- Local Sun, Moon and planet calculations.
- Target-specific Moon separation and altitude penalties.
- Deep-sky catalogue built from a pinned OpenNGC snapshot.
- Milky Way / Galactic Centre opportunity.
- Major meteor showers.
- Potentially observable comets from Minor Planet Center elements.
- Visible-satellite passes from CelesTrak orbital elements.
- NOAA OVATION aurora probability.
- User-defined horizon mask.
- Telescope and binocular aperture filtering.
- Fixed SQM, Home Assistant SQM sensor, or an imported light-pollution grid.
- Nearby darker-point search when a local light-pollution grid is available.
- Home Assistant entities for dashboards and automations.
- Built-in Ingress view and a dependency-free native dashboard preset.

## Installation

In Home Assistant, add this repository to the app store:

`https://github.com/ArrowSK/ha-astronomy-observer`

Install **Astronomy Observer**, review the configuration, then start it. The default configuration uses Home Assistant's Home coordinates. To follow a person, set `primary_person` to an entity such as `person.alex`.

The full setup and operating guide is in [`astronomy_observer/DOCS.md`](astronomy_observer/DOCS.md).

## Documentation

- [Installation, configuration and Home Assistant entities](astronomy_observer/DOCS.md)
- [Scoring method](docs/SCORING.md)
- [Data sources, caching and fallbacks](docs/DATA_SOURCES.md)
- [Light-pollution and SQM setup](docs/LIGHT_POLLUTION.md)
- [Privacy and location handling](docs/PRIVACY.md)
- [Architecture and resource budget](docs/ARCHITECTURE.md)
- [Local Ingress endpoints](docs/API.md)
- [Scientific and technical references](docs/SCIENTIFIC_REFERENCES.md)
- [Development and validation](docs/DEVELOPMENT.md)
- [Known limits and roadmap](docs/LIMITS_AND_ROADMAP.md)

## Licence

Project code is licensed under the **PolyForm Noncommercial License 1.0.0**. It may be used, changed and redistributed for permitted non-commercial purposes. Commercial use of this code or a derivative is not licensed. The intention is straightforward: the project should remain freely usable by amateur astronomers, clubs, education and non-commercial research, rather than becoming the basis of a paid wrapper or service.

This is a source-available non-commercial licence, not an OSI open-source licence.

Third-party code and data keep their own licences. See [`THIRD_PARTY_LICENSES.md`](THIRD_PARTY_LICENSES.md).

## Release status

`0.1.0` is the first public release and is marked experimental in Home Assistant. The calculations are deliberately explicit about uncertainty. Estimated seeing is a weather-derived proxy rather than a forecast in arcseconds, recurring meteor-shower dates are a planning aid rather than a substitute for the current IMO calendar, and sky brightness stays unknown when no SQM or local atlas data are available.
