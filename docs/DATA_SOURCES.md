# Data sources and fallbacks

Astronomy Observer keeps each outside source replaceable and cached separately. A failure in a comet or aurora source should not stop the core observing score. Weather is the exception: without weather or a recent weather cache, a complete new observing result cannot be produced.

## Source table

| Data | Primary | Fallback | Normal refresh/cache rule |
|---|---|---|---|
| Person location | Home Assistant `person.*` | Home coordinates | Checked every app refresh |
| Weather | Open-Meteo Forecast API | MET Norway Locationforecast | App refresh; recent weather cache retained up to 12 h |
| Aerosols / PM2.5 | Open-Meteo Air Quality | unavailable under MET Norway fallback | App refresh |
| Sun, Moon, planets | Astronomy Engine, local | none required | Recalculated locally |
| Deep-sky catalogue | OpenNGC pinned at build | build fails if source unavailable | Static in image |
| Meteor showers | Bundled major-shower table | none | Static; reviewed by release |
| Comets | Minor Planet Center elements | second MPC comet-elements endpoint | 12 h normal cache; stale cache rejected after 7 d |
| Satellites | CelesTrak visual GP data | recent local cache | 12 h normal cache; stale cache rejected after 48 h |
| Aurora | NOAA SWPC OVATION | recent local cache | Live request; stale cache rejected after 3 h |
| Light pollution | fixed SQM / HA SQM sensor / user atlas grid | unknown | Local only |

## Weather: Open-Meteo

The primary forecast requests hourly:

- temperature;
- relative humidity;
- dew point;
- total, low, middle and high cloud cover;
- visibility;
- precipitation probability;
- 10 m wind and gusts;
- 500 hPa and 200 hPa wind;
- 850 hPa and 500 hPa temperature.

A second Open-Meteo air-quality request supplies aerosol optical depth, dust and PM2.5 when available.

Coordinates are rounded before these requests according to `external_location_precision`. The local astronomy calculations do not use the rounded coordinates.

Documentation: https://open-meteo.com/en/docs and https://open-meteo.com/en/docs/air-quality-api

## Weather fallback: MET Norway

When Open-Meteo fails, the app requests MET Norway Locationforecast. It supplies the core surface forecast and cloud layers but not the same aerosol and upper-air set used by the primary source. Transparency and estimated-seeing confidence therefore fall when the fallback is active.

The request uses a project-identifying User-Agent and is cached rather than repeatedly retried.

Documentation: https://api.met.no/weatherapi/locationforecast/2.0/documentation

## Local astronomy: Astronomy Engine

The image builds a small C helper from Astronomy Engine v2.1.19, pinned to commit `61dc07020aaa6885d2c7f688a4d82beaf6edb9ef`.

It calculates:

- Sun, Moon and planet equatorial coordinates;
- local altitude and azimuth;
- apparent magnitude where provided by the library;
- illuminated fraction;
- Earth's heliocentric ecliptic vector for local comet propagation.

The upstream project is MIT licensed.

Upstream: https://github.com/cosinekitty/astronomy

## Deep sky: OpenNGC

The container build downloads a pinned OpenNGC snapshot and creates a smaller tab-separated observing catalogue. The full upstream database is not retained in the runtime image.

The selected runtime fields include J2000 position, type, constellation, visual/blue magnitude, galaxy surface brightness, angular size, Messier cross-reference and common name.

OpenNGC is licensed CC BY-SA 4.0. Its derived runtime catalogue remains under that data licence.

Upstream: https://github.com/mattiaverga/OpenNGC

## Comets: Minor Planet Center

The app first tries the MPC all-comet elements file and then the MPC comet ephemeris elements file. It stores the downloaded element set under `/data`, so a short source outage does not remove comet recommendations.

The local propagator is intentionally lightweight. It is suitable for filtering likely observing opportunities but does not replace a high-precision ephemeris for acquisition.

Source: https://www.minorplanetcenter.net/iau/MPCORB/CometEls.html

## Satellites: CelesTrak

Only the CelesTrak `visual` group is downloaded. This keeps CPU use and target noise low. The data are normally refreshed no more often than every 12 hours, substantially less often than the upstream GP update cadence.

Source and GP documentation: https://celestrak.org/NORAD/documentation/gp-data-formats.php

## Aurora: NOAA SWPC

The app reads the latest OVATION aurora JSON grid from NOAA Space Weather Prediction Center and selects the nearest grid point to the observing location. It is treated as a short-lived source: cached OVATION data older than three hours are rejected.

Product: https://www.swpc.noaa.gov/products/aurora-30-minute-forecast

## Meteor showers: International Meteor Organization

The bundled table is based on recurring properties of major showers published in IMO calendars. It is deliberately not updated from a network service at runtime.

Current annual calendars should be checked for exact maximum times, Moon circumstances and unexpected activity.

Source: https://www.imo.net/resources/calendar/

## Light pollution: Falchi atlas

The project does not bundle the World Atlas of Artificial Night Sky Brightness. The official GFZ dataset is large and is distributed under CC BY-NC 4.0 with its own access/attribution conditions. Users who obtain it can create a local grid with the included conversion tool.

Dataset: https://dataservices.gfz-potsdam.de/contact/showshort.php?id=escidoc:1541893

Paper: Falchi et al. (2016), *The new world atlas of artificial night sky brightness*, Science Advances 2(6), e1600377.

## Cache location

Changing network data are stored in the app's persistent `/data` volume. They are not exposed through Home Assistant configuration and survive normal app restarts/upgrades according to Supervisor volume handling.

The app configuration folder mounted at `/config` is read-only and is used only for user-supplied files such as `light_pollution.csv`.
