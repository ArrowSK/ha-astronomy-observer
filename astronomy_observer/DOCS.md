# Astronomy Observer documentation

## What the app does

Astronomy Observer answers three practical questions:

1. How good are the observing conditions at the selected Home Assistant location?
2. When is the best two-hour window tonight?
3. Which targets are most worthwhile during that window and the surrounding night?

Astronomical positions, light-pollution lookup and target geometry are calculated locally. Current weather and changing orbital or space-weather data are downloaded from public sources and cached. No account or API key is required.

## First start

The default configuration works without a `person` entity. In that case the app uses the latitude, longitude, elevation and time zone configured for Home Assistant.

After the first successful refresh, open the Astronomy Observer panel and press **Setup**. The observer field lists the `person.*` entities already available in Home Assistant, so there is normally no need to type an entity ID by hand.

If a selected person temporarily has no usable latitude and longitude, the app falls back to Home rather than failing the entire refresh.

The first refresh can take longer than later refreshes because the app may need to collect current weather, comet and satellite data. The bundled light-pollution lookup is local and does not add a network request.

## Setup panel

The Setup panel is intended for the settings most people need frequently.

### Observer

Choose **Home** or one of the Home Assistant people shown in the list. Saving the setting triggers a refresh immediately.

The selection is stored in the app's persistent data folder and overrides the `primary_person` value from the app configuration until it is changed again from Setup.

### Horizon

For most users the only horizon setting needed is **Lowest useful altitude**.

Typical choices are:

- 10–15° for a very open observing site;
- 20° as the general default;
- 25° where low buildings or trees are common;
- 30–35° for a strongly restricted urban or balcony view.

Targets below this altitude are ignored.

The **Advanced directional horizon** section is optional. It exists for observers whose horizon is blocked by different amounts in different directions. If that does not describe the site, leave it at the flat default.

An advanced mask is a comma-separated set of `azimuth:altitude` points, for example:

```text
0:12,45:18,90:30,135:24,180:12,225:8,270:10,315:14
```

The app interpolates between points and wraps through north.

### Light pollution

Nothing needs to be configured. Astronomy Observer includes a compact approximately 3-arcminute derivative of the Falchi World Atlas and looks up the selected observer location locally on every refresh.

The resulting moonless sky-brightness estimate feeds the darkness component immediately, so the headline, deep-sky and imaging scores already reflect the current location's light pollution.

Observers with better local data can still use a fixed SQM value, a Home Assistant SQM sensor or an optional higher-resolution local CSV. Those inputs take priority over the bundled atlas. See [Light pollution and sky brightness](../docs/LIGHT_POLLUTION.md).

## Full app configuration

The Home Assistant app configuration remains available for less frequently changed settings.

### Location and refresh

`primary_person`
: Optional `person.*` entity. The Setup panel is the easier way to choose it. Blank means Home.

`refresh_minutes`
: Normal refresh interval, 10–180 minutes. Default: 30. Location is checked on every refresh.

`forecast_days`
: Number of nights in the outlook, 2–7. Default: 7.

`observing_window_hours`
: How far ahead tonight's target search runs, 8–18 hours. Default: 14.

`external_location_precision`
: Decimal places used for coordinates sent to weather providers. Default: 3. Local astronomy and light-pollution lookup keep the unrounded Home Assistant coordinates. See [Privacy](../docs/PRIVACY.md).

### Observing limits

`minimum_target_altitude`
: Lowest useful altitude in degrees. Default: 20°. The Setup panel exposes this as a simple list of common values.

`horizon_mask`
: Optional directional obstruction mask. Most users should leave the default unchanged and use `minimum_target_altitude` instead.

`good_observing_threshold`
: Threshold for `binary_sensor.astronomy_observer_good_observing`. Default: 70.

### Equipment

`telescope_aperture_mm`
: Clear telescope aperture in millimetres. Set to 0 when telescope-only targets should not be considered.

`binocular_aperture_mm`
: Binocular objective diameter in millimetres. Set to 0 when binocular-only targets should not be considered.

Aperture is currently used as a broad visibility gate. It is not a substitute for a complete optical train model; focal length, field of view, filters and imaging scale remain future work.

### Sky brightness and light pollution

The app uses this priority order:

1. `sqm_override`, when greater than zero;
2. `sqm_entity`, when it contains a valid numeric state;
3. `light_pollution_file`, when a suitable local CSV exists;
4. the bundled location-based World Atlas estimate;
5. unknown only when none of those sources can provide a usable value.

`sqm_override`
: Fixed moonless zenith sky brightness in mag/arcsec². Set to 0 to disable.

`sqm_entity`
: Optional Home Assistant `sensor.*` entity whose state is an SQM-style value in mag/arcsec².

`light_pollution_file`
: Optional file name inside the app configuration folder. Default: `light_pollution.csv`. The file does not need to exist. It is now an optional higher-resolution/local override rather than a requirement for atlas-based scoring.

`nearby_dark_site_radius_km`
: Straight-line radius used to search for a materially darker atlas cell. Default: 75 km. The bundled atlas is used automatically unless a valid custom local grid has taken priority. Set to 0 to disable. This is a sky-quality search, not a route, access or safety check.

The bundled atlas is a 2015 planning baseline, not a live SQM measurement. It adds about 42 MB to the installed image but is accessed directly from disk. A normal lookup reads one cell and a nearby search uses a small row buffer rather than loading the whole grid into memory.

### Changing targets and events

`enable_satellites`
: Searches current CelesTrak visual-group orbital elements for potentially visible passes.

`enable_comets`
: Searches current Minor Planet Center comet elements for potentially observable active comets.

`enable_aurora`
: Reads the current NOAA SWPC OVATION product and adds an aurora recommendation when the local probability is meaningful.

## The scores

The headline score is 0–100, but it is not intended to be read alone. The app also publishes:

- clear-sky factor;
- transparency;
- estimated seeing;
- darkness;
- Moon interference;
- wind score;
- dew score;
- deep-sky score;
- planetary score;
- imaging score;
- data confidence.

The score is deterministic and its current formula is documented in [SCORING.md](../docs/SCORING.md). The weights are engineering choices for observing decisions rather than a published meteorological standard. This is why the component scores and raw inputs remain visible.

The built-in World Atlas estimate enters through the darkness component. In the current formula, darkness contributes to the headline overall score and has greater influence on deep-sky work, while planetary observing is intentionally much less sensitive to city light pollution.

The condition tiles in the Ingress page are clickable. Opening a tile explains what the quantity means and how it is used.

## Best-target ranking

The target list is deliberately not a simple numerical sort across every possible object class.

The app applies category limits so one class cannot fill the entire list. In particular, only one satellite and one comet can occupy the final Top 10. Deep-sky objects and planets may occupy several places because they are the normal observing programme for many advanced amateurs.

Satellite pass brightness is not available from the CelesTrak orbital elements used by the app. For that reason ordinary satellites, rocket bodies and debris are deliberately de-weighted even when the geometry of the pass is excellent. Recognisable high-interest objects such as the ISS receive a higher interest weight, but still do not bypass the one-satellite limit.

Minor Planet Center objects with an `A/` designation are excluded from the comet list because that designation is used for asteroid-like objects rather than an active comet observing recommendation.

A list can contain fewer than ten objects if not enough candidates pass the configured observing limits. The app does not pad the list with weak extra satellites or other low-value entries merely to reach ten.

## Seven-night outlook

The Ingress page shows more than a single score for each night. It includes the best time and the corresponding deep-sky, planetary, imaging, clear-sky, transparency, Moon-impact and confidence values.

The Home Assistant `sensor.astronomy_observer_next_good_night` entity keeps the compact outlook structure used by the initial release, so existing automations and dashboard templates remain compatible.

## Home Assistant entities

The app publishes state entities through the authenticated Home Assistant Core API. They are created after a successful refresh and recreated after restart as needed.

Core status:

```text
sensor.astronomy_observer_score
sensor.astronomy_observer_best_window
sensor.astronomy_observer_deep_sky
sensor.astronomy_observer_planetary
sensor.astronomy_observer_imaging
sensor.astronomy_observer_confidence
binary_sensor.astronomy_observer_good_observing
```

Atmosphere and darkness:

```text
sensor.astronomy_observer_cloud
sensor.astronomy_observer_cloud_cover
sensor.astronomy_observer_cloud_layers
sensor.astronomy_observer_transparency
sensor.astronomy_observer_seeing
sensor.astronomy_observer_darkness
sensor.astronomy_observer_visibility
sensor.astronomy_observer_aod
sensor.astronomy_observer_wind
sensor.astronomy_observer_jet_stream
sensor.astronomy_observer_wind_score
sensor.astronomy_observer_dew_margin
sensor.astronomy_observer_dew_score
sensor.astronomy_observer_sky_brightness
sensor.astronomy_observer_moon_illumination
sensor.astronomy_observer_moon_altitude
sensor.astronomy_observer_moon_interference
sensor.astronomy_observer_sun_altitude
```

`sensor.astronomy_observer_sky_brightness` reports the estimated SQM-style value. Its attributes include the source, artificial zenith luminance when atlas-based, and distance to the atlas cell centre. The source makes it clear whether the value came from a fixed override, a Home Assistant SQM sensor, a custom grid or the bundled World Atlas.

Targets and events:

```text
sensor.astronomy_observer_top_target
sensor.astronomy_observer_target_1
...
sensor.astronomy_observer_target_10
sensor.astronomy_observer_meteor_shower
sensor.astronomy_observer_comet
sensor.astronomy_observer_satellite_pass
sensor.astronomy_observer_aurora
```

Planning and diagnostics:

```text
sensor.astronomy_observer_next_good_night
sensor.astronomy_observer_nearby_dark_site
sensor.astronomy_observer_weather_source
sensor.astronomy_observer_source_status
sensor.astronomy_observer_last_update
```

Target sensors include score, category, best time, altitude, azimuth, magnitude when known, Moon separation when relevant, equipment suggestion and a short note as attributes.

The app also fires an `astronomy_observer_updated` event after each successful publish. It contains the overall score, top target and generation time.

## Dashboard preset

A native dashboard preset is included and uses no custom cards. Open the Astronomy Observer panel and press **Copy dashboard YAML**, or copy [`dashboard/astronomy-dashboard.yaml`](../dashboard/astronomy-dashboard.yaml) from the repository.

Create a separate Home Assistant dashboard, open its raw configuration editor and paste the preset. The app deliberately does not write to Home Assistant's dashboard storage or `configuration.yaml`.

## Automations

A simple notification trigger can use the binary sensor:

```yaml
triggers:
  - trigger: state
    entity_id: binary_sensor.astronomy_observer_good_observing
    to: "on"
actions:
  - action: notify.send_message
    target:
      entity_id: notify.mobile_app_your_phone
    data:
      message: >-
        Good observing conditions are available. Best window:
        {{ states('sensor.astronomy_observer_best_window') }}.
        Best target: {{ states('sensor.astronomy_observer_top_target') }}.
```

For more selective alerts, use `astronomy_observer_updated` and inspect the score or target entities in conditions.

## Manual refresh

The Ingress page has a **Refresh now** button. It asks the running service to perform a new refresh immediately. Normal refreshes continue on the configured schedule.

Saving Observer Setup also triggers a refresh. The runtime reloads the persisted setup before every calculation, so changing the observer or simple horizon does not require restarting the app.

## Observing notes

The Ingress page includes a local observing log. During or after a session you can save any combination of:

- measured SQM;
- estimated or measured seeing in arcseconds;
- transparency on a 1–5 observer scale;
- naked-eye limiting magnitude;
- a short free-text note.

Each entry is saved with the current forecast component scores and location label. Exact coordinates are not written to the log. The newest 50 entries can be reviewed in the Ingress page; the full append-only file remains in the app's persistent `/data` directory.

The log is intended to support local calibration over time. It does not automatically change the scoring model in this release.

## Source failures

The app does not invent missing observations. Weather has an independent provider fallback and a limited recent cache. Other changing sources have separate caches with age limits. The World Atlas lookup is bundled and local, so it does not depend on an external service at runtime. The source-status entity and Ingress page show which source was used.

If weather cannot be obtained from either provider and there is no recent cache, the refresh fails and the previous successful Home Assistant states remain in place. Check the app log and `sensor.astronomy_observer_last_update` before relying on an old result.

## Logs and troubleshooting

Useful log messages include:

- `refresh complete` — a full result was calculated;
- `Open-Meteo unavailable` — the primary weather source failed and the fallback is being tried;
- `MET Norway unavailable` — both live weather providers have failed;
- `refresh failed` — no complete new snapshot could be produced;
- `configuration reload failed` — a saved setup value or app configuration is invalid;
- `Home Assistant state publish error` — the calculation succeeded but state publishing had a problem.

If the Ingress page says it is waiting for the first calculation, inspect the app log first. Common first-start causes are temporary lack of weather data or an invalid manually entered advanced horizon mask.

## What the app is not

Astronomy Observer is not an observatory-control system, mount driver, plate solver, cloud safety interlock or severe-weather warning service. It ranks observing opportunity. Equipment safety, local access, road conditions, lightning, wind limits and observatory closure decisions remain separate responsibilities.
