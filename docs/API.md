# Local web endpoints

Astronomy Observer exposes a small HTTP interface behind Home Assistant Ingress. It is not intended to be exposed as a public network API. The server accepts requests only from loopback or Home Assistant's Ingress proxy address.

All paths below are relative to the Ingress base URL.

## `GET /health`

Returns `200` with `ok` after the first successful calculation. Before the first complete snapshot it returns `503` with `starting`.

The Home Assistant watchdog uses a TCP check on port 8099 because the HTTP service intentionally rejects non-Ingress source addresses.

## `GET /api/snapshot`

Returns the current calculated snapshot as JSON, or `null` before the first successful refresh.

The snapshot includes the observing location because the Ingress page needs it. The panel is admin-only. Consumers should treat this JSON structure as a local app interface that may gain fields between releases.

The compact `outlook` field is kept for Home Assistant compatibility. The Ingress page also uses `outlook_details`, which contains the best time plus deep-sky, planetary, imaging, clear-sky, transparency, estimated-seeing, darkness, Moon-impact and confidence values for each night.

## `POST /api/refresh`

Queues an immediate refresh and returns:

```json
{"accepted": true}
```

The normal refresh loop remains responsible for the calculation, so repeated requests do not create parallel calculation workers.

## `GET /api/dashboard`

Returns the bundled native Home Assistant dashboard preset as YAML. The Ingress page uses this endpoint for the **Copy dashboard YAML** button.

## `GET /api/people`

Returns Home Assistant `person.*` entities for the observer selector. Each item contains the entity ID, friendly name and whether current latitude/longitude attributes are available.

Example:

```json
[
  {
    "entity_id": "person.alex",
    "name": "Alex",
    "has_location": true
  }
]
```

The endpoint does not expose a person's coordinates. Home is added as a separate choice by the Ingress page.

## `GET /api/settings`

Returns the effective observer and horizon settings currently used by the runtime:

```json
{
  "primary_person": "person.alex",
  "minimum_target_altitude": 20.0,
  "horizon_mask": "0:0,90:0,180:0,270:0"
}
```

Settings saved from the Ingress page override the matching values from Home Assistant app configuration. Other app options remain controlled by the normal Home Assistant configuration page.

## `POST /api/settings`

Saves the observer and horizon values used by the Ingress Setup panel. The body has the same three fields shown above.

The person must be blank or a `person.*` entity. The altitude must be between 0° and 60°. The directional horizon mask is parsed before anything is written. A valid save is written atomically into the app's persistent `/data` directory and queues a refresh.

The simple Setup panel normally changes only `minimum_target_altitude`; the raw directional mask is tucked under the advanced section.

## `GET /api/observations`

Returns the most recent locally saved observing notes, newest first. The response is capped at 50 records. The log is stored in the app's persistent `/data` directory and is not uploaded anywhere.

## `POST /api/observation`

Stores one optional field-note record against the current forecast snapshot. Accepted JSON fields are:

```json
{
  "sqm": 20.7,
  "seeing_arcsec": 1.8,
  "transparency": 4,
  "limiting_magnitude": 5.9,
  "notes": "Thin haze low in the west after midnight."
}
```

All fields are optional, but an empty record is rejected. `transparency` uses a 1–5 observer scale. Numeric fields are range-checked and notes are length-limited. Request bodies over 16 KiB are rejected.

The saved record also contains the current condition scores and a location label so forecasts can be compared with actual observations later. Exact latitude and longitude are not written to the observation log.

This endpoint is intended for personal calibration notes, not as a safety log or scientific database.

## Security boundary

Authentication is provided by Home Assistant Ingress. The app does not open a user-configurable host port and does not implement a second account/password system. Do not expose port 8099 manually.
