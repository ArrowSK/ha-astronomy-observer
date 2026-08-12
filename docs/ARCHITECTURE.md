# Architecture and resource budget

Astronomy Observer is deliberately small for a continuously running Home Assistant host.

## Runtime layout

```text
Home Assistant Core API
        │
        ├── person/Home location
        ├── optional SQM sensor
        └── published astronomy states/events
        │
        ▼
Astronomy Observer (Rust)
        │
        ├── weather providers + caches
        ├── bundled World Atlas light-pollution reader
        ├── optional local light-pollution grid reader
        ├── OpenNGC compact catalogue reader
        ├── comet propagator
        ├── SGP4 satellite propagator
        ├── scoring / target ranking
        └── small Ingress HTTP server
        │
        └── astro-helper (C, Astronomy Engine)
```

There is no database server, browser runtime, Node service or Python interpreter in the final image. Python is used only during builds to reduce OpenNGC and to create the bundled light-pollution derivative from the official source raster.

## Why Rust plus a C helper

The long-running process is Rust because it gives predictable memory use without a managed runtime. Astronomy Engine's C implementation is mature, small and easy to call in one batch: the Rust service sends the required timestamps to `astro-helper`, which returns the Sun/Moon/planet samples and exits.

This keeps the astronomy library out of the main process address space between refreshes.

## Memory budget

Design targets for a normal Raspberry Pi-class installation are:

| State | Target |
|---|---:|
| Idle / between refreshes | under 30 MB RSS |
| Normal refresh | under 45 MB RSS |
| Heavy refresh with moving targets enabled | under 70 MB RSS |
| Design ceiling | 80 MB RSS |

These are release targets, not a guarantee for every allocator/kernel/build combination. A release should be measured on real Home Assistant hardware before the `experimental` stage is removed.

The architecture avoids the common sources of unnecessary memory use:

- weather data are only seven days of hourly records;
- astronomy samples are only 30-minute points for the configured forecast horizon;
- the approximately 42 MB bundled light-pollution atlas stays on disk;
- a normal atlas lookup reads one 16-bit cell and the darker-area search keeps one approximately 14 KB row buffer;
- an optional CSV is streamed line by line;
- the full OpenNGC database is discarded at build time;
- satellite elements are limited to CelesTrak's visual group;
- no global star catalogue is held in memory;
- the web interface is static HTML included in the binary.

The bundled atlas therefore increases installed image/storage size, not the working set by tens of megabytes.

## CPU behaviour

Between refreshes the service mostly sleeps and serves the occasional Ingress request. The normal calculation runs every 30 minutes by default.

The heavier operations are:

- deep-sky ranking across the compact catalogue;
- SGP4 propagation for the visual satellite group;
- comet propagation for current MPC elements;
- the bounded nearby darker-area atlas search when enabled.

They run as short batches rather than continuous loops. Satellite, comet and aurora features can be disabled independently if a very small host needs less work.

The direct sky-brightness lookup is effectively a file seek plus a two-byte read. The darker-area search reads only rows that can intersect the configured radius.

## Storage

Persistent `/data` contains only changing network caches, observing notes and timestamps. Typical cache contents are weather JSON, CelesTrak visual elements, MPC comet elements and NOAA OVATION JSON.

The final container includes:

- the Rust binary;
- the C astronomy helper;
- the reduced OpenNGC catalogue;
- the meteor-shower table;
- the approximately 41.8 MB bundled 3-arcminute World Atlas derivative and its metadata/notice;
- certificates/base-system files.

It does not contain the multi-gigabyte World Atlas source GeoTIFF or the full OpenNGC source database.

An optional user-supplied light-pollution CSV lives in the separate app configuration folder mounted at `/config`. Its file size affects scan time and storage more than RAM because it is streamed.

## Network budget

With the default 30-minute refresh:

- weather is requested at most once per refresh, plus a separate Open-Meteo air-quality request when the primary provider succeeds;
- CelesTrak data are normally reused for 12 hours;
- MPC comet data are normally reused for 12 hours;
- NOAA OVATION is checked on refresh when enabled;
- the light-pollution atlas is local and never requires a runtime network request;
- static catalogues are not downloaded at runtime.

The app uses no paid API and no API key.

## Failure boundaries

Each changing source is isolated. A failed satellite download removes satellite recommendations but does not invalidate the weather score. A failed aurora product removes the aurora input. The bundled light-pollution atlas is a static local resource; if the observer is outside its geographic coverage or a cell has no value, sky brightness falls through to unavailable unless a higher-priority SQM/local-grid input exists.

Weather is required for a new complete snapshot. If both live weather providers fail, a recent cache can be used. If that cache is too old, the refresh fails rather than publishing a fresh timestamp against stale conditions.

## Home Assistant permissions

The app requests:

- Home Assistant Core API access;
- Ingress;
- read-only access to its own app-configuration folder.

It does not request privileged mode, host networking, Docker socket access or write access to Home Assistant configuration.
