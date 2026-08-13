# Standalone web deployment

Astronomy Observer has a standalone Docker adapter in `webapp/`. The purpose is to make the same observing engine usable outside Home Assistant without creating a second scoring implementation that can drift over time.

## What is shared

The web build compiles the same Rust source files used by the Home Assistant app for:

- astronomy and coordinate calculations;
- weather and atmospheric inputs;
- condition scoring and the best observing window;
- deep-sky and solar-system target selection;
- meteor showers, comets, satellites and aurora;
- the bundled light-pollution atlas and darker-area logic;
- snapshot/result models.

It also starts from the same `astronomy_observer/web/index.html` interface. `webapp/src/ui.rs` performs a small runtime transformation for the standalone setup screen and startup behaviour. This keeps the Conditions, Targets, Forecast, Sources and bottom-navigation presentation tied to the Home Assistant interface instead of copying the full HTML into another directory.

`webapp/validate.py` checks that the shared source references and important interface markers still exist. The web helper/catalogue build scripts also pin the same Astronomy Engine and OpenNGC revisions used by the Home Assistant Dockerfile.

## What stays separate

Home Assistant keeps its existing Supervisor token handling, `person.*` location support, entity publishing, dashboard preset, observation storage and Ingress source-address restriction. None of those security boundaries are relaxed for the web deployment.

The standalone binary has its own public HTTP server. It exposes only the page, health/runtime information, a refresh acknowledgement used by the shared interface, and the web snapshot calculation endpoint. It does not expose the Home Assistant Supervisor API or HA-only dashboard/journal endpoints.

The observation journal is hidden in the first standalone adapter rather than being converted into shared server-side storage. This avoids mixing observations from unrelated visitors. If web journal persistence is added later, it should remain browser-local or use a proper per-user storage model.

## Location and privacy

Standalone Setup currently accepts a site name, latitude, longitude, elevation and IANA time-zone name manually. These values remain in the open browser page and are sent with the calculation request. Reloading the page clears them.

The server validates coordinate, elevation, time-zone and horizon ranges before calculation. A calculation gets its own randomly named temporary workspace, which is deleted after the response. Location-specific provider caches therefore are not reused between visitors.

The calculation still follows the existing external-coordinate precision setting used by Astronomy Observer when querying supported external weather/data providers. The light-pollution atlas lookup and astronomy calculations use the location inside the container.

The web service does not require an account, database or persistent volume for the initial deployment.

## Docker

From the repository root:

```sh
docker build -f webapp/Dockerfile -t astronomy-observer-web .
docker run --rm -p 8080:8080 -e PORT=8080 astronomy-observer-web
```

The service listens on `PORT`, defaulting to `8080` when the variable is absent.

Useful endpoints:

- `GET /health` — simple `200 ok` readiness check.
- `GET /api/platform` — identifies the runtime as `web`, reports the shared app version and indicates that the adapter uses the shared runtime.
- `POST /api/web/snapshot` — validates an explicit observing site and returns the normal Astronomy Observer snapshot.

The public server intentionally returns `404` for Home Assistant-only routes that are not implemented by the adapter.

## Railway

`railway.toml` is prepared at repository root. Railway's Dockerfile builder is pointed at `webapp/Dockerfile`; watch patterns include `astronomy_observer/**`, `webapp/**` and `railway.toml`; `/health` is configured as the deployment health check.

A future Railway deployment therefore only needs the repository connected to a service. Railway provides `PORT` automatically. No application secret is required by this initial stateless web adapter.

The repository does **not** create a Railway project, service, domain or deployment automatically. Those remain explicit deployment-owner actions.

## Resource behaviour

The bundled World Atlas remains the largest static addition to the image. It is read in the same compact manner as the Home Assistant app and is not loaded as a full global raster into RAM.

A public snapshot calculation performs the same core work as a Home Assistant refresh. The standalone adapter uses temporary per-request provider caches to prevent cross-visitor fallback contamination, so repeated public calculations can cause more external provider traffic than the long-running Home Assistant app. This is an intentional first-version trade-off. A future shared cache should separate location-independent feeds from location-specific weather data before it is enabled globally.

The web server handles HTTP requests on separate threads so a long astronomy/provider calculation does not prevent `/health` or the static page from being served.

## Validation and drift control

`webapp/validate.py` checks repository structure, source pin parity, Railway configuration and shared-interface markers. In addition, the standalone adapter was compiled through the repository's normal Rust formatting, Clippy, test and release-build pipeline while it was being introduced. The final Home Assistant image remains built from its original runtime binary; the web adapter is packaged only by `webapp/Dockerfile`.

When changing shared astronomy or UI code, run at least:

```sh
python3 webapp/validate.py
cargo fmt --manifest-path astronomy_observer/Cargo.toml -- --check
cargo clippy --manifest-path astronomy_observer/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path astronomy_observer/Cargo.toml

docker build -f webapp/Dockerfile -t astronomy-observer-web .
```

A Docker smoke test should then verify `/health`, `/api/platform`, the root page and rejection of invalid coordinates before deploying anywhere.
