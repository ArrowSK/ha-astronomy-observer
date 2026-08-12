# Astronomy Observer documentation

## What the app does

Astronomy Observer answers three practical questions:

1. How good are the observing conditions at the selected Home Assistant location?
2. When is the best two-hour window tonight?
3. Which targets are most worthwhile during that window and the surrounding night?

The app calculates the astronomy locally. Current weather and changing orbital/space-weather data are downloaded from public sources and cached. No account or API key is required.

## First start

The default configuration works without a `person` entity. In that case the app uses the latitude, longitude, elevation and time zone configured for Home Assistant.

For a moving observer, set:

```yaml
primary_person: person.alex
```

The entity must have latitude and longitude attributes. If the selected person temporarily has no usable coordinates, the app falls back to Home Assistant's Home coordinates rather than failing the entire refresh.

The first refresh can take longer than later refreshes because the app may need to collect current weather, comet and satellite data. After the first successful refresh, the Ingress page becomes available and the Home Assistant entities are populated.

## Configuration

### Location and refresh

`primary_person`
: Optional `person.*` entity to follow. Blank means Home coordinates.

`refresh_minutes`
: Normal refresh interval, 10–180 minutes. Default: 30. Location is checked on every refresh.

`forecast_days`
: Number of nights in the outlook, 2–7. Default: 7.

`observing_window_hours`
: How far ahead tonight's target search runs, 8–18 hours. Default: 14.

`external_location_precision`
: Decimal places used for coordinates sent to weather providers. Default: 3. Local astronomy keeps the unrounded Home Assistant coordinates. See [Privacy](../docs/PRIVACY.md).

### Observing limits

`minimum_target_altitude`
: Global minimum altitude in degrees. Default: 20°. The horizon mask can impose a higher limit at a given azimuth.

`horizon_mask`
: A comma-separated list of `azimuth:minimum_altitude` points. The app interpolates between points and wraps through north. Example:

```text
0:12,45:18,90:30,135:24,180:12,225:8,270:10,315:14
```

Use this to exclude buildings, trees, hills or a poor low horizon. A flat unobstructed horizon is:

```text
0:0,90:0,180:0,270:0
```

`good_observing_threshold`
: Threshold for `binary_sensor.astronomy_observer_good_observing`. Default: 70.

### Equipment

`telescope_aperture_mm`
: Clear telescope aperture in millimetres. Set to 0 when telescope-only targets should not be considered.

`binocular_aperture_mm`
: Binocular objective diameter in millimetres. Set to 0 when binocular-only targets should not be considered.

Aperture is currently used as a broad visibility gate. It is not a substitute for a full optical train model; focal length, field of view, filters and imaging scale are listed on the roadmap.

### Sky brightness and light pollution

The app uses this priority order:

1. `sqm_override`, when greater than zero;
2. `sqm_entity`, when it contains a valid numeric state;
3. `light_pollution_file`, when a suitable local grid exists;
4. unknown.

`sqm_override`
: Fixed moonless zenith sky brightness in mag/arcsec². Set to 0 to disable. This is useful for a regularly used observing site with a known representative value.

`sqm_entity`
: Optional Home Assistant `sensor.*` entity whose state is an SQM-style value in mag/arcsec². This is the preferred option for a live local meter.

`light_pollution_file`
: File name inside this app's configuration folder. Default: `light_pollution.csv`. See [Light-pollution setup](../docs/LIGHT_POLLUTION.md).

`nearby_dark_site_radius_km`
: Straight-line radius used to search the imported grid for a materially darker point. Set to 0 to disable. This is a sky-quality search, not a route or access check.

### Changing targets and events

`enable_satellites`
: Searches current CelesTrak visual-group orbital elements for visible passes.

`enable_comets`
: Searches current Minor Planet Center comet elements for potentially observable comets.

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

The score is deterministic and its current formula is documented in [SCORING.md](../docs/SCORING.md). The weights are engineering choices for observing decisions, not a published meteorological standard. This is why the component scores and raw inputs remain visible.

## Home Assistant entities

The app publishes state entities through the authenticated Home Assistant Core API. They are created when the app successfully refreshes and are recreated after restart as needed.

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

The app also fires an `astronomy_observer_updated` event after each successful publish. It contains the overall score, top target and generation time. This is useful for automations that should react only when a complete result has been published.

## Dashboard preset

A native dashboard preset is included and uses no custom cards. Open the Astronomy Observer Ingress page and press **Copy dashboard YAML**, or copy [`dashboard/astronomy-dashboard.yaml`](../dashboard/astronomy-dashboard.yaml) from the repository.

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

For more selective alerts, use `astronomy_observer_updated` and inspect score/target entities in conditions.

## Manual refresh

The Ingress page has a **Refresh now** button. It asks the running service to perform a new refresh immediately. Normal refreshes continue on the configured schedule.

## Observing notes

The Ingress page includes a small local observing log. After or during a session you can save any combination of:

- measured SQM;
- estimated or measured seeing in arcseconds;
- transparency on a 1–5 observer scale;
- naked-eye limiting magnitude;
- a short free-text note.

Each entry is saved with the current forecast component scores and the location label. Exact coordinates are not written to the log. The newest 50 entries can be reviewed in the Ingress page; the full append-only file remains in the app's persistent `/data` directory.

The log is intended to make local calibration possible over time. It does not change the scoring model automatically in this release.

## Source failures

The app does not silently invent missing observations. Weather has an independent provider fallback and a limited recent cache. Other changing sources have separate caches with age limits. The source-status entity and Ingress page show which source was used.

If weather cannot be obtained from either provider and there is no recent cache, the refresh fails and the previous successful Home Assistant states remain in place. Check the app log and `sensor.astronomy_observer_last_update` before relying on an old result.

## Logs and troubleshooting

Useful log messages include:

- `refresh complete` — a full result was calculated;
- `Open-Meteo unavailable` — the primary weather source failed and the fallback is being tried;
- `MET Norway unavailable` — both live weather providers have failed;
- `refresh failed` — no complete new snapshot could be produced;
- `Home Assistant state publish error` — the calculation succeeded but state publishing had a problem.

If the Ingress page says it is waiting for the first calculation, inspect the app log first. Most first-start failures are either an invalid `person.*` entity, an invalid horizon mask, or temporary lack of weather data.

## What the app is not

Astronomy Observer is not an observatory-control system, mount driver, plate solver, cloud safety interlock or severe-weather warning service. It ranks observing opportunity. Equipment safety, local access, road conditions, lightning, wind limits and observatory closure decisions remain separate responsibilities.
