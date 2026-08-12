# Privacy and location handling

Astronomy Observer can follow a Home Assistant `person` entity, so location handling is treated as sensitive even though the app has no account system of its own.

## What stays inside Home Assistant

The full Home Assistant latitude and longitude are used locally for:

- Sun, Moon and planet positions;
- target altitude and azimuth;
- horizon checks;
- airmass;
- local light-pollution-grid lookup;
- comet and satellite topocentric calculations.

The app does not write the selected person's exact coordinates to its persistent cache.

When the observer is chosen from the built-in Setup panel, `/data/ui_settings.json` stores only the selected `person.*` entity ID and the horizon settings. It does not store that person's latitude or longitude. Changing the selected observer replaces the stored entity ID.

The current in-memory snapshot does contain the location because it is needed by the Ingress interface. The panel is admin-only and the HTTP service rejects normal requests that do not come from Home Assistant Ingress or loopback.

The `/api/people` endpoint used by the selector returns only each person's entity ID, friendly name and whether current coordinates are available. It does not return the coordinates themselves.

## Coordinates sent to weather providers

Weather services need a location. Before a request is sent, latitude and longitude are rounded according to `external_location_precision`.

Typical latitude scale:

- 2 decimal places: roughly 1.1 km;
- 3 decimal places: roughly 110 m;
- 4 decimal places: roughly 11 m;
- 5 decimal places: roughly 1.1 m.

Longitude distance per decimal place varies with latitude.

The default is 3 decimal places. Lower it to 2 if kilometre-scale weather location is sufficient and you prefer less precise disclosure. Increase it only when a more precise forecast location is genuinely useful.

## Other network sources

The NOAA OVATION, CelesTrak and Minor Planet Center downloads are global datasets. The app does not put the observer's coordinates into those request URLs.

OpenNGC and Astronomy Engine are downloaded only while the container image is built, not during normal runtime.

## Home Assistant state

Home Assistant state entities contain observing results. Most do not include location coordinates. The nearby-dark-site entity includes candidate latitude/longitude as attributes when a local grid is configured, because that information is necessary for the feature.

Anyone who can read your Home Assistant states may therefore see astronomy results and, when enabled, the darker-point coordinates. Home Assistant access control remains the security boundary.

## Local observing notes

The Ingress page can save optional SQM, seeing, transparency, limiting-magnitude and free-text notes. These records are stored only in `/data/observations.jsonl`. They include a location label and the forecast scores that were current when the note was saved, but they do not include exact latitude or longitude.

Free-text notes are user-provided data. Do not put information in them that you would not want retained with the Home Assistant app backup. Removing the app's persistent data removes the local log and Setup-panel overrides.

## Telemetry

The app has no analytics endpoint, crash-reporting service, advertising system or usage telemetry. Network requests are limited to the documented astronomy and weather sources required for the enabled features.
