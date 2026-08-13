# Astronomy Observer Web

This directory prepares a standalone Docker web deployment of Astronomy Observer. It is deliberately kept in the same repository as the Home Assistant app and consumes the same calculation source files rather than maintaining a second astronomy/scoring implementation.

The standalone adapter reuses the Home Assistant app's Rust modules for astronomy, weather, scoring, target selection, light pollution, comets, meteor showers, satellites and aurora. It also starts from the same `astronomy_observer/web/index.html` interface and applies only the web-specific location/setup changes at runtime. If the shared interface markers change, the web adapter self-test and `webapp/validate.py` are intended to fail rather than silently drifting.

The Home Assistant runtime is not made public and its Ingress source-address restriction is not changed. The standalone web server is a separate binary and container.

## Current web behaviour

The web version asks for an observing-site name, latitude, longitude, elevation and IANA time zone in Setup. The values are kept only in the open page and are sent with a calculation request; the server does not create a user account or retain the submitted location after the request finishes. Reloading the page clears the site.

The main observing result, Conditions, Targets, Forecast and Sources views use the same shared calculation code as Home Assistant. The Home Assistant dashboard-copy action and Home Assistant observation journal are hidden in the standalone adapter because those features depend on HA-local persistence and should not become shared public-server state accidentally.

Each public calculation uses an isolated temporary working directory and removes it afterwards. This avoids leaking a location-specific weather cache between visitors. It also means the initial standalone version favours privacy/isolation over aggressive server-side caching.

## Docker

Build from the repository root:

```sh
docker build -f webapp/Dockerfile -t astronomy-observer-web .
docker run --rm -p 8080:8080 -e PORT=8080 astronomy-observer-web
```

Then open `http://localhost:8080`. The container exposes `GET /health` for container/orchestrator health checks and `GET /api/platform` for a small runtime identity response.

## Railway preparation

The repository root contains `railway.toml`. It selects `webapp/Dockerfile`, watches the shared Home Assistant source tree plus the web adapter, and configures `/health` as the Railway health check. Connecting this repository and `main` to a Railway service therefore allows the standalone build to follow the same shared source changes.

Nothing in this repository automatically creates or deploys a Railway service. Deployment, domain creation and any Railway project changes remain explicit owner actions.

See [`../docs/WEBAPP.md`](../docs/WEBAPP.md) for architecture, privacy, validation and deployment notes.
