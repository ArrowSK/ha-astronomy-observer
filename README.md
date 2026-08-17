# Astronomy Observer

Astronomy Observer helps with the part of amateur astronomy that is surprisingly hard to answer from an ordinary weather app: **Is tonight actually worth going out? When is the useful window? And what should I point the telescope at?**

It combines weather, darkness, Moon interference, local light pollution, observing geometry and target visibility into one practical view — while still showing the ingredients behind the answer.

## Highlights

- **A useful night plan, not just a cloud percentage.** See the best two-hour window, separate deep-sky/planetary/imaging scores, and the conditions that produced them.
- **Targets are ranked for your sky and your equipment.** Altitude, local horizon, Moon separation, sky brightness, object type and configured aperture all matter.
- **Recognisable targets now look recognisable.** Small offline thumbnails are bundled for Messier objects plus familiar planets and major meteor showers; if a target has no licensed image, the list simply falls back to the normal astronomy marker.
- **Light pollution works out of the box.** A compact World Atlas derivative is bundled, so location already affects darkness-sensitive scoring without asking you to find an SQM map or CSV first.
- **Run it the way that suits you.** Home Assistant, a completely standalone Android APK, self-hosted Docker, or the same web container on Railway/another container host all use the same observing engine.
- **No Astronomy Observer account and no paid astronomy API.** The project has no telemetry or advertising service. Changing public data are fetched from their documented sources; core astronomy and the bundled catalogues stay local.

## Choose how to run it

### Android — nothing to host

The Android edition is the most independent way to use Astronomy Observer. The APK contains the interface, Rust observing engine, Astronomy Engine, reduced deep-sky catalogue, meteor-shower table and compact World Atlas light-pollution grid. It does **not** load Astronomy Observer from a website and it does not need Home Assistant, Railway, this repository or any server run by the project owner after installation.

Use the phone's location or enter an observing site manually. Sun/Moon/planet calculations, target geometry, the catalogue, horizon handling, light pollution and the local observation journal work on the device. Fresh weather, comet, satellite and aurora information naturally needs internet access and is requested directly from the documented public providers; recent changing data are cached locally.

If live weather is unavailable and there is no recent cache, the Android app can still make an astronomy-planning snapshot from its local data, but it clearly marks weather as unavailable and lowers confidence rather than pretending that the sky is clear.

See [Android: a standalone Astronomy Observer in your pocket](docs/ANDROID.md) for installation, offline behaviour, privacy and build details.

### Home Assistant

Best if Astronomy Observer should be part of your smart home. The Home Assistant edition can follow a `person` entity, publish sensors for dashboards and automations, use a Home Assistant SQM sensor, and keep the observing interface inside HA Ingress.

Add this repository to the Home Assistant app store:

`https://github.com/ArrowSK/ha-astronomy-observer`

Install **Astronomy Observer** and start it. The default setup uses Home Assistant's Home coordinates. After the first successful refresh, open Astronomy Observer, open the hamburger menu, choose **Setup**, then select the observer and lowest useful altitude for the site.

See the [Home Assistant guide](astronomy_observer/DOCS.md).

### Standalone Docker web app

If you want a browser-based service on your own machine or NAS, build the standalone container. It uses the same astronomy, weather, scoring, target-ranking and light-pollution modules as the Home Assistant and Android editions.

From the repository root:

```sh
docker build -f webapp/Dockerfile -t astronomy-observer-web .
docker run --rm -p 8080:8080 -e PORT=8080 astronomy-observer-web
```

Then open `http://localhost:8080` and enter the observing site in Setup.

### Railway or another container host

The standalone web container is also prepared for Railway. The root [`railway.toml`](railway.toml) points Railway at `webapp/Dockerfile` and uses `/health` for deployment health checks. The repository does not create a Railway project or deploy anything automatically.

See [Standalone web deployment](docs/WEBAPP.md) for Docker, Railway, privacy and deployment details.

## What goes into the answer

Astronomy Observer is aimed mainly at amateur observers who want more than a generic “good stargazing” badge, but the interface tries to keep the reasoning readable. Condition rows can be expanded when you want to understand where a value came from.

The current feature set includes:

- best two-hour observing window for tonight;
- seven-night forecast with deep-sky, planetary, imaging, clear-sky, transparency, Moon-impact and confidence detail;
- total, low, middle and high cloud layers;
- visibility, humidity, dew point, aerosol optical depth, surface wind and upper-air wind;
- separate overall, deep-sky, planetary and imaging scores;
- relative seeing estimate, with no fake arcsecond precision;
- local Sun, Moon and planet calculations;
- target-specific Moon separation, altitude, airmass and horizon penalties;
- a reduced observing catalogue built from a pinned OpenNGC snapshot;
- Milky Way / Galactic Centre opportunities and major meteor showers;
- current active-comet candidates from Minor Planet Center elements;
- visible-satellite passes from CelesTrak elements, with unknown brightness deliberately de-weighted;
- category-aware Top 10 selection so satellites or comets cannot crowd out the normal observing programme;
- compact target thumbnails for licensed Wikimedia Commons images, with in-app per-image credit/licence links and no remote image dependency;
- NOAA OVATION aurora probability;
- simple lowest-useful-altitude setup plus an optional directional horizon mask;
- telescope and binocular aperture filtering;
- automatic location-based light-pollution estimate from the bundled World Atlas derivative;
- nearby darker-area search from that atlas;
- source-status indicators showing current/local, cached/fallback and unavailable inputs.

Some integrations are edition-specific. Home Assistant adds `person` tracking, HA entities/dashboard/automation support and optional HA SQM input. Android adds on-device location and a self-contained local runtime. The web edition provides a normal HTTP service suitable for Docker hosting.

## Light pollution without another setup project

No separate light-pollution file is required for normal use. Astronomy Observer looks up the observing location in its bundled approximately 3-arcminute World Atlas grid. That estimate already feeds the darkness component, so moving from a city centre to a dark site changes the relevant observing scores as well as the target ranking.

It is deliberately described as an estimate: the atlas is a static 2015 planning baseline, not tonight's measured SQM. The Home Assistant edition can use a real SQM sensor, and fixed/custom higher-resolution inputs can override the atlas where supported.

## One observing engine

The project does not maintain separate scoring formulas for each platform. Home Assistant, Docker/web and Android compile the same Rust astronomy/scoring/target modules. Platform adapters deal with location, storage and presentation around that shared engine.

On Home Assistant and Docker, Astronomy Engine's C implementation runs through a small helper executable. On Android it is linked into the native library and called locally through JNI, so the phone does not need a helper process or remote backend.

## Privacy by design

There is no Astronomy Observer account, analytics endpoint, crash-reporting service or advertising system. Exact location is kept for local astronomical calculations. Coordinates sent to weather providers are rounded according to the shared privacy setting; global comet, satellite and aurora downloads do not include the observer coordinates in their request URLs.

The Android WebView is an embedded local interface rather than a hosted application. Remote pages are blocked inside that privileged WebView; explicit external links open in the normal browser.

See [Privacy and location handling](docs/PRIVACY.md) for the details.

## Documentation

- [Android standalone APK](docs/ANDROID.md)
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

## Licence and third-party data

Original Astronomy Observer code is distributed under the **PolyForm Noncommercial License 1.0.0**. It is a source-available non-commercial licence, not an OSI open-source licence.

Bundled and downloaded third-party material keeps its own licence and attribution; it is not silently relicensed as Astronomy Observer code. In particular, the OpenNGC-derived catalogue remains CC BY-SA 4.0, the bundled World Atlas derivative remains CC BY-NC 4.0, and Astronomy Engine remains MIT. The Android app also exposes attribution and licence information from its **About & licences** menu.

The project is configured for non-commercial use, which is important for the bundled World Atlas and the free Open-Meteo service. See [`THIRD_PARTY_LICENSES.md`](THIRD_PARTY_LICENSES.md) for the complete separation and source links.

## Release status

`0.3.1` remains experimental. The project is intentionally conservative about uncertainty: the seeing value is a weather-derived relative proxy rather than an arcsecond forecast, recurring meteor-shower dates are a planning aid rather than a replacement for the current IMO calendar, satellite brightness is not invented from orbital geometry, and a stale/offline Android snapshot is clearly separated from a current weather forecast.
