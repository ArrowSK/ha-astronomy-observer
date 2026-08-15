# Privacy and location handling

Astronomy Observer needs a location to answer astronomical questions, so the project treats coordinates as working data rather than profile data. There is no Astronomy Observer account, analytics endpoint, advertising system or usage telemetry.

The exact storage boundary depends on how you run it, but the basic rule is the same: **keep precise coordinates local for astronomy and send no more precision to a weather provider than the forecast needs.**

## What exact coordinates are used for

Full local latitude/longitude are useful for:

- Sun, Moon and planet positions;
- target altitude and azimuth;
- horizon checks and airmass;
- topocentric comet/satellite geometry;
- local World Atlas/light-pollution lookup;
- nearby darker-area search.

Those calculations happen in the local runtime: Home Assistant app, your Docker container, or the Android native library.

## Coordinates sent to weather providers

Weather services need a location. Before a weather request is sent, latitude and longitude are rounded according to `external_location_precision`.

Typical latitude scale:

- 2 decimal places: roughly 1.1 km;
- 3 decimal places: roughly 110 m;
- 4 decimal places: roughly 11 m;
- 5 decimal places: roughly 1.1 m.

Longitude distance per decimal place varies with latitude. The default is 3 decimal places.

The current MET Norway request path also stays within that provider's documented coordinate-precision expectations. Open-Meteo and MET Norway remain visible as sources when used.

## Other live sources

NOAA OVATION, CelesTrak and Minor Planet Center downloads are global datasets. Astronomy Observer does not put the observer's coordinates into those download URLs.

The World Atlas, meteor-shower table and reduced OpenNGC observing catalogue are local runtime resources and do not require a location request to a remote service.

## Home Assistant edition

When a Home Assistant `person` is selected, the runtime reads that person's current coordinates for local calculation. The persistent UI settings store the selected `person.*` entity ID and horizon settings, not a copy of the person's latitude/longitude.

If the selected person temporarily has no usable coordinates, Home is used as the fallback.

The current in-memory snapshot necessarily contains the working location because the Ingress interface needs it. The HA HTTP service accepts normal UI traffic only through Home Assistant Ingress/loopback. `/api/people` returns entity ID, friendly name and coordinate availability, not the coordinates themselves.

Most published HA entities do not contain location coordinates. The nearby-dark-site entity can include candidate coordinates because those are the point of that feature. Anyone who can read those HA states can therefore see the suggested darker point; Home Assistant access control is the boundary.

## Android edition

The Android APK does not use an Astronomy Observer server. Device/manual coordinates are passed from the local WebView shell into the native Rust runtime on the same phone.

The selected observing site and horizon settings are stored in the app's private local storage so the app can reopen at the same site. The observation journal is local as well. Android backup is disabled for the application, so the app is not asking Android to copy that local journal/location state into cloud backup.

Location permission is optional. If it is denied, manual coordinates continue to work.

The embedded WebView is intentionally not a normal browser:

- its interface comes from APK assets;
- remote network loading is disabled inside it;
- file-to-network access from the embedded page is disabled;
- mixed content is disabled;
- explicit external links are handed to the system browser;
- documented provider requests are made by the native Rust runtime, not arbitrary page JavaScript.

This design prevents the APK from silently becoming dependent on a remotely hosted Astronomy Observer page later.

## Docker / hosted web edition

The standalone web adapter accepts the observing location explicitly through its API/UI and calculates on the server/container you chose to run. If you deploy it on a third-party host such as Railway, that host is therefore part of your deployment's data boundary.

Astronomy Observer itself adds no account or telemetry layer around that deployment. If location privacy is important, self-host Docker or use the standalone Android edition so the application runtime stays on equipment you control.

## Observing notes

The Home Assistant edition stores observation records in its persistent app data. The Android edition stores them in private app-local browser storage. Records can contain SQM, seeing, transparency, limiting magnitude, free-text notes and a location label, but the journal format does not intentionally add exact latitude/longitude to each record.

Free-text notes are user-provided data. Do not put sensitive information into notes unless you are comfortable retaining it in the relevant local app/container storage.

## Build-time downloads are not runtime dependencies

Astronomy Engine and OpenNGC source data are downloaded while building images/APKs from pinned commits. The installed Android app contains the compiled/reduced outputs and does not need this GitHub repository in order to start or calculate later.

## Telemetry

There is none. Network requests are limited to the documented astronomy/weather sources needed by enabled features and, when you explicitly open one, normal external links in your chosen browser.
