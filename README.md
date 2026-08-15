# Astronomy Observer

Astronomy Observer helps answer the questions that matter before you carry a telescope outside: **Is tonight worth observing? When is the best window? And what is actually worth looking at?**

It is not limited to Home Assistant. The same observing engine can be used in several ways, depending on how you want to run it.

## Highlights

- **More than a weather score.** Cloud layers, transparency, seeing proxy, darkness, Moon interference, wind, dew margin and confidence stay visible instead of being hidden behind a single badge.
- **Useful target suggestions, not a random object list.** The app ranks up to ten worthwhile targets using altitude, airmass, the local horizon, Moon separation, sky brightness, object type and your configured aperture.
- **Several ways to run it.** Use the full Home Assistant app, run the standalone web version in Docker, or deploy that same standalone container on Railway.
- **No account or paid astronomy API required.** Core calculations are local, the light-pollution atlas is bundled, and changing weather/orbital data comes from public sources.

## Choose how to run it

### Home Assistant

Best if you already use Home Assistant and want Astronomy Observer to follow a `person` entity, publish entities for dashboards and automations, keep an observation journal, and live inside the normal Home Assistant interface.

Add this repository to the Home Assistant app store:

`https://github.com/ArrowSK/ha-astronomy-observer`

Install **Astronomy Observer** and start it. The default setup uses Home Assistant's Home coordinates. After the first successful refresh, open Astronomy Observer, open the hamburger menu, choose **Setup**, then select a Home Assistant person and the lowest useful altitude for your observing site.

The full Home Assistant guide is in [`astronomy_observer/DOCS.md`](astronomy_observer/DOCS.md).

### Standalone Docker web app

You do not need Home Assistant to use the observing engine. The repository also contains a standalone web version in [`webapp/`](webapp/) that reuses the same Rust astronomy, weather, scoring, target-ranking and light-pollution code.

From the repository root:

```sh
docker build -f webapp/Dockerfile -t astronomy-observer-web .
docker run --rm -p 8080:8080 -e PORT=8080 astronomy-observer-web
```

Then open `http://localhost:8080` and enter the observing-site location in the web Setup screen.

### Railway

The standalone web version is also prepared for Railway. The root [`railway.toml`](railway.toml) points Railway at `webapp/Dockerfile` and uses `/health` for deployment health checks.

Connecting this repository to a Railway service gives you the same standalone web application without maintaining a separate codebase. The repository does **not** automatically create or deploy a Railway project; deployment remains an explicit owner action.

See [Standalone web deployment](docs/WEBAPP.md) for Docker, Railway, privacy and deployment details.

## What Astronomy Observer looks at

Astronomy Observer is aimed mainly at experienced amateur astronomers, but the interface tries to keep the reasoning understandable. Instead of simply saying that a night is "good" or "bad", it shows the factors behind the result and lets you inspect them individually.

The current feature set includes:

- Seven-night observing outlook with deep-sky, planetary, imaging, clear-sky, transparency, Moon-impact and confidence detail.
- Best two-hour observing window for tonight.
- Total, low, middle and high cloud layers.
- Visibility, humidity, dew point, aerosol optical depth, surface wind and upper-air wind.
- Separate overall, deep-sky, planetary and imaging scores.
- Grouped condition lists that separate observing-quality scores from forecast measurements.
- Plain-language explanations available directly from the condition rows.
- Local Sun, Moon and planet calculations.
- Target-specific Moon separation and altitude penalties.
- Deep-sky catalogue built from a pinned OpenNGC snapshot.
- Milky Way / Galactic Centre opportunities.
- Major meteor showers.
- Potentially observable active comets from Minor Planet Center elements.
- Visible-satellite passes from CelesTrak orbital elements, with unknown brightness deliberately de-weighted.
- Category-aware Top 10 selection so satellites and comets cannot crowd out a normal observing programme.
- NOAA OVATION aurora probability.
- Simple lowest-useful-altitude horizon setup, with an optional advanced directional mask.
- Telescope and binocular aperture filtering.
- Automatic location-based light-pollution estimate from the bundled World Atlas derivative.
- Fixed SQM, Home Assistant SQM sensor and higher-resolution local CSV overrides when better local data is available.
- Nearby darker-area search from the built-in atlas without manual setup.
- Source-status indicators showing current/local, cached/fallback and unavailable data.

The Home Assistant version additionally provides `person`-based location, HA entities, dashboard/automation support, the Ingress view and the local observation journal.

## Light pollution

No separate light-pollution file is required for normal use. Astronomy Observer looks up the selected location in its bundled approximately 3-arcminute World Atlas grid and uses that estimate when calculating overall, deep-sky and imaging conditions.

If you have better local information, a real SQM sensor, a fixed SQM value or a higher-resolution local grid can override the bundled estimate.

## One engine, two interfaces

The Home Assistant and standalone web versions intentionally share the core calculation source rather than maintaining two separate astronomy implementations. The standalone adapter changes the location/setup and public HTTP layer; it does not weaken or expose Home Assistant's Supervisor-token handling, entity publishing or Ingress source-address restriction.

The runtime is a small Rust service with a C astronomy helper. The Home Assistant build is designed for hosts where memory and background CPU use matter, while the standalone Docker image packages the same calculation engine for normal web deployment.

## Documentation

- [Home Assistant installation, configuration and entities](astronomy_observer/DOCS.md)
- [Standalone Docker and Railway deployment](docs/WEBAPP.md)
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
