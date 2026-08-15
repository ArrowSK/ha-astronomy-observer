# Architecture and resource budget

Astronomy Observer now has three runtime shapes, but deliberately only one observing engine. The Home Assistant app, standalone web service and Android APK compile the same Rust astronomy, weather, scoring, target-ranking and light-pollution modules.

## Shared core, thin platform edges

```text
                         shared Rust observing core
               ┌────────────────┼────────────────┐
               │                │                │
               ▼                ▼                ▼
        Home Assistant      Docker / web        Android APK
        HA location         manual location     GPS/manual location
        HA entities         HTTP API            JNI bridge
        Ingress UI          web UI              local WebView UI
               │                │                │
               └────────────────┼────────────────┘
                                │
                 astronomy / scoring / targets
                 weather / caches / catalogues
                 comets / satellites / aurora
                 World Atlas light pollution
```

This is intentional. A scoring fix should not have to be copied into an Android implementation and then maintained separately.

## Astronomy Engine

The shared Rust code expects Astronomy Engine results in one compact line-oriented format.

On Home Assistant and Docker, a small C helper process calculates the requested Sun/Moon/planet samples and exits. On Android, spawning that helper would be awkward and unnecessary, so the same pinned Astronomy Engine C implementation is linked into the native Android library. A small FFI adapter returns exactly the format expected by the shared Rust parser.

Astronomy Engine remains under its upstream MIT licence in every edition.

## Home Assistant runtime

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
        ├── public weather/data providers + caches
        ├── bundled World Atlas reader
        ├── optional local light-pollution grid
        ├── reduced OpenNGC catalogue
        ├── comet and satellite propagation
        ├── scoring / target ranking
        └── small Ingress HTTP server
        │
        └── astro-helper (C, Astronomy Engine)
```

There is no Node service, database server or Python interpreter in the final HA image. Python is build-time tooling only.

## Standalone web runtime

The Docker/web edition uses the same Rust modules and static interface but replaces Home Assistant-specific location/entity handling with an ordinary HTTP adapter. Its security boundary is a separate public server mode; the Home Assistant Ingress source-address restriction is not weakened to make the web version possible.

Railway configuration simply points at this Docker build. Railway is therefore a deployment option, not a separate implementation.

## Android runtime

The Android edition is deliberately self-contained:

```text
bundled WebView UI
       │
       ▼
Java bridge
       │ JNI
       ▼
shared Rust core (.so)
       │
       ├── linked Astronomy Engine C code
       ├── packaged reduced OpenNGC catalogue
       ├── packaged meteor table
       ├── packaged World Atlas derivative
       └── direct public-provider requests + app-local caches
```

The WebView loads only `file:///android_asset/` content packaged in the APK. Remote network loading is disabled inside that privileged WebView. Explicit external links are handed to the normal Android browser. The Rust layer, not the WebView, performs the documented weather/astronomy-provider requests.

The APK copies large read-only observing assets into its private app files on first start so the shared file readers can use normal filesystem access. It does not contact this GitHub repository at runtime.

## Offline boundary on Android

Local astronomy does not need a network connection: Sun/Moon/planets, target geometry, horizon checks, the reduced deep-sky catalogue, meteor table and World Atlas lookup are packaged with the app.

Weather, current comet elements, current satellite elements and current aurora data are naturally time-sensitive. The phone fetches those directly when possible and reuses recent caches under the same rules as the other editions. If both weather providers fail and Android has no recent weather cache, the Android edition can still calculate a local astronomy-planning snapshot using unknown weather fields. That snapshot is marked stale/low-confidence; it is not presented as a current clear-sky forecast.

The Home Assistant and Docker editions keep their established behavior: without live weather or a sufficiently recent weather cache, a fresh complete snapshot is not published.

## Home Assistant memory budget

Design targets for a normal Raspberry Pi-class HA installation remain:

| State | Target |
|---|---:|
| Idle / between refreshes | under 30 MB RSS |
| Normal refresh | under 45 MB RSS |
| Heavy refresh with moving targets enabled | under 70 MB RSS |
| Design ceiling | 80 MB RSS |

These are release targets, not guarantees for every allocator/kernel/build combination.

The architecture avoids the usual unnecessary memory costs:

- weather data are only seven days of hourly records;
- astronomy samples are 30-minute points for the configured horizon;
- the approximately 42 MB World Atlas stays on disk;
- a normal atlas lookup reads one 16-bit cell and a dark-site search keeps one small row buffer;
- optional CSV data are streamed;
- the full OpenNGC source database is reduced at build time;
- satellite elements are limited to CelesTrak's visual group;
- no global star catalogue is held in memory;
- the interface is static HTML rather than a JavaScript server runtime.

The atlas therefore mainly increases installed storage size, not working memory by tens of megabytes.

## Android storage and APK size

Android intentionally trades package size for independence. The World Atlas derivative alone is about 42 MB before APK compression, and the APK also carries the reduced observing catalogue, shared UI and native libraries for `arm64-v8a` and `x86_64`.

Changing network data are kept in Android's private app storage. Exact observing-site settings and the local observation journal remain there as well; Android backup is disabled for the app.

## Network behavior

With the normal refresh cadence:

- weather is requested at most once per refresh, plus Open-Meteo air-quality data when applicable;
- CelesTrak visual elements are normally reused for 12 hours;
- MPC comet elements are normally reused for 12 hours;
- NOAA OVATION is checked when enabled;
- the World Atlas and observing catalogues are local;
- no runtime downloads are made from this project repository.

The app uses no paid API and no project-operated backend.

## Failure boundaries

Changing sources remain isolated. A failed satellite download removes current satellite recommendations rather than invalidating weather. A failed aurora product removes that input. Light pollution remains available from the bundled atlas unless the location is outside coverage or a cell has no value.

Unknown inputs stay unknown and reduce confidence rather than being replaced with invented observations.

## Home Assistant permissions

The HA edition requests Home Assistant Core API access, Ingress and read-only access to its own app-configuration folder. It does not request privileged mode, host networking, Docker socket access or write access to Home Assistant configuration.

The Android edition requests internet access and optional coarse/fine location. Manual coordinates work without granting location permission.
