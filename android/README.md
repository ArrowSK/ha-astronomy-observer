# Astronomy Observer for Android

This directory builds the self-contained Android edition of Astronomy Observer. It is not a bookmark or a remote WebView wrapper: the interface, astronomy engine, compact observing catalogue and light-pollution atlas travel inside the APK, and the Rust calculation engine runs on the phone.

Fresh weather, comet elements, satellite elements and aurora data still need internet access. Those requests go directly from the phone to the documented public providers and use the same cache rules as the other editions. If live weather is unavailable and there is no recent cache, the Android edition can still calculate local astronomy and targets, but the weather inputs remain unknown and confidence is deliberately reduced.

## Build

The reproducible build expects JDK 17, Gradle 8.13, Android SDK platform 36, build-tools 35.0.0, Android NDK 27.0.12077973, Rust 1.89, Python 3 and curl.

From the repository root:

```sh
rustup target add aarch64-linux-android x86_64-linux-android
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/27.0.12077973"
./android/build-apk.sh
```

The resulting local/CI preview is:

`android/generated/apk/astronomy-observer-0.3.4-debug.apk`

Android's debug signing makes that APK installable for sideload testing. It is not a permanent public update identity. A public release should be signed with one stable project-controlled release key kept outside the repository; the private key must never be committed.

## What is shared

The Android Rust library compiles the same `astronomy_observer/src` modules used by Home Assistant and the standalone web service. The Android-only layer provides device/manual location, persistent app directories and a JNI bridge. The shared HTML interface is transformed at build time by `prepare_assets.py`, so normal UI changes are inherited instead of copied by hand.

Astronomy Engine is compiled into the native library rather than launched as a helper process on Android. The input/output format is intentionally the same as the existing helper, which keeps the higher-level astronomy path unchanged.

## Privacy

The embedded WebView is intentionally unable to load remote web content. Exact saved observing coordinates and the observation journal stay in Android app storage. The Rust runtime makes only the documented data-provider requests. See [`../docs/ANDROID.md`](../docs/ANDROID.md) and [`../docs/PRIVACY.md`](../docs/PRIVACY.md).

## Licensing

The APK contains material under several compatible-but-separate licences. The project licence applies only to original project code. OpenNGC-derived catalogue data remain CC BY-SA 4.0, the World Atlas derivative remains CC BY-NC 4.0, and Astronomy Engine remains MIT. Provider attribution is visible from **About & licences** inside the Android app. See [`../THIRD_PARTY_LICENSES.md`](../THIRD_PARTY_LICENSES.md).
