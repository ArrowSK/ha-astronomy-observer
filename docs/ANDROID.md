# Android: a standalone Astronomy Observer in your pocket

The Android edition is for people who want Astronomy Observer without running a server at all. Install the APK, choose a location, and the phone does the astronomy work itself.

This is deliberately different from a thin WebView pointed at somebody else's website. The APK contains the interface, the Rust observing engine, Astronomy Engine, the reduced deep-sky catalogue, meteor-shower table and compact World Atlas light-pollution grid. There is no Astronomy Observer account, hosted backend or owner-operated service that has to stay alive for the app to open and calculate.

## Best parts

- **Nothing to host.** Home Assistant, Docker, Railway and the project repository are not runtime dependencies.
- **The same observing brain.** Scores, target ranking, light-pollution handling, comet/satellite logic and the rest of the calculation path come from the same Rust modules as the other editions.
- **Useful even when the network is poor.** Sun, Moon, planets, geometry, the deep-sky catalogue, horizon checks and the bundled light-pollution atlas are local. Recent downloaded data are cached. If there is no usable weather data, the app says so and lowers confidence instead of pretending the sky is clear.
- **Location stays local where it can.** Exact coordinates are used on the phone for astronomy. Open-Meteo receives rounded coordinates for the forecast, using the same privacy setting as the shared runtime.

## Installing a sideloaded build

An Android APK must be signed before Android will install it. CI/local preview builds use Android's normal debug signing, so they can be sideloaded for testing without Google Play. That debug certificate is not intended as a permanent public update identity.

For a real public sideload release, use one stable project-controlled release key and keep the private key outside GitHub. Users can then install later versions over the existing app because Android sees the same signing identity. Losing or replacing that key breaks that normal update path, so it should be backed up securely rather than committed to the repository.

## First start

Open **Setup** and either choose **Use current location** or type an observing site manually. Location permission is optional: manual latitude/longitude continues to work if permission is denied.

The app stores the chosen site, horizon settings and observation journal in its own private Android app storage. Android cloud backup is disabled for the app. Clearing the app's data removes those local settings and journal entries.

## What works without internet

The following parts are packaged with the app and calculated locally:

- Sun, Moon and planet positions;
- observer altitude/azimuth and local horizon checks;
- target geometry, airmass and Moon separation;
- the reduced OpenNGC observing catalogue;
- the major meteor-shower planning table;
- the bundled approximately 3-arcminute World Atlas light-pollution lookup;
- local darkness and Moon-interference calculations;
- telescope/binocular aperture filtering;
- the observation journal and its search/filter/delete controls.

Fresh forecasts and changing orbital/space-weather datasets naturally require a network connection. The Android edition talks directly to Open-Meteo's free non-commercial API for weather, CelesTrak for current visual-satellite elements, the Minor Planet Center for comet elements and NOAA SWPC for aurora data. None of those requests are proxied through infrastructure operated by this project.

The Home Assistant/Docker editions keep MET Norway as an independent weather fallback. Android intentionally does not call MET Norway directly: its current API terms recommend a proxy for mobile applications except at low volume, and adding an owner-operated proxy would defeat the purpose of this standalone edition.

Weather is the one input that strongly affects whether *tonight* is actually usable. If Open-Meteo is unavailable, a recent local weather cache can be used. If there is no usable cache either, the app can still produce a local astronomy-planning snapshot with weather fields unknown. Source status shows that state and the confidence calculation is intentionally poor; an offline snapshot must not be read as a clear-sky forecast.

The free Open-Meteo endpoint is explicitly for non-commercial use and has published request limits. Astronomy Observer is non-commercial, refreshes on a modest cadence, caches weather locally and keeps a visible **Weather data by Open-Meteo.com** credit/link beside Source status. Weather values are transformed into the observing scores shown by the app; that transformation is also stated in the licence screen.

## WebView, but not a website dependency

The visible interface is still HTML/CSS/JavaScript because that lets Home Assistant, standalone web and Android share the same UI work. On Android the WebView loads only the copy packaged inside the APK.

Remote web loading is blocked inside that WebView. It cannot silently turn into a hosted app later, and the JavaScript bridge is exposed only to the bundled local interface. External links, when explicitly opened, are handed to the normal Android browser instead of running inside the privileged WebView.

Calculations cross a narrow Java/JNI bridge into a native Rust library. Astronomy Engine's C implementation is linked into the same native library on Android, so no helper executable or server process is required.

## APK contents and size

The World Atlas derivative is about 42 MB before APK packaging, so this is intentionally not a tiny shell application. Most of that size buys offline light-pollution coverage. The APK also contains the native libraries for `arm64-v8a` phones and `x86_64` emulators/devices, the reduced OpenNGC catalogue and the shared interface.

## Building it yourself

See [`../android/README.md`](../android/README.md) for exact build prerequisites and the build command. GitHub Actions performs the same native build, assembles the APK and keeps the preview APK as a workflow artifact.

The Android build downloads two pinned build-time sources: Astronomy Engine C source and OpenNGC source CSV files. Those are converted/compiled into the APK during the build. The app does not need GitHub to fetch them after installation.

## Licences and attribution

Standalone distribution makes attribution particularly important, so the Android app has an **About & licences** item in its hamburger menu. It includes the project licence, the exact upstream Astronomy Engine MIT notice, World Atlas transformation notice, OpenNGC attribution/licence, and weather-provider attribution.

The important separation is intentional: original Astronomy Observer code is PolyForm Noncommercial 1.0.0; the OpenNGC-derived catalogue remains CC BY-SA 4.0; the World Atlas derivative remains CC BY-NC 4.0; Astronomy Engine remains MIT; live Open-Meteo data remain CC BY 4.0. No third-party material is presented as if the project relicensed it.

The World Atlas source itself records that its licence changed to CC BY-NC 4.0 after the embargo period on 13 November 2019. The app keeps the Falchi et al. citation, DOI, derivative description and licence with the distributed dataset.

For the complete source list and links, see [`../THIRD_PARTY_LICENSES.md`](../THIRD_PARTY_LICENSES.md).
