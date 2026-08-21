# Third-party code and data

Astronomy Observer keeps third-party licensing separate from the project licence. The PolyForm Noncommercial License 1.0.0 applies to original Astronomy Observer code; bundled or downloaded material listed below keeps its own licence and attribution.

This separation matters particularly for the standalone Android build because the APK contains several datasets and compiled third-party code. The Android **About & licences** screen includes the relevant notices so attribution travels with the application rather than existing only in this repository.

## Astronomy Engine

The builds use the C implementation of Astronomy Engine from commit `61dc07020aaa6885d2c7f688a4d82beaf6edb9ef` (release `v2.1.19`) in `cosinekitty/astronomy`.

Astronomy Engine is copyright Don Cross and is distributed under the MIT License. Home Assistant/Docker compile it into the small astronomy helper. Android links it into the native library instead. The Android build also packages the exact upstream `LICENSE` text; Astronomy Engine is not relicensed under PolyForm.

Upstream: https://github.com/cosinekitty/astronomy

## OpenNGC

The build downloads OpenNGC from commit `da90466031b0372c896588b85be6016c617e205b` and creates a smaller observing catalogue containing Messier cross-references and brighter/large visually useful NGC/IC objects.

OpenNGC is credited to Mattia Verga and licensed under Creative Commons Attribution-ShareAlike 4.0 International (CC BY-SA 4.0). The generated observing catalogue remains a CC BY-SA 4.0 data derivative; it is distributed alongside the application and is **not** relicensed under the project's PolyForm licence.

Upstream: https://github.com/mattiaverga/OpenNGC

Licence: https://creativecommons.org/licenses/by-sa/4.0/

## World Atlas of Artificial Night Sky Brightness

Astronomy Observer includes a compact adapted derivative of:

Falchi, F. et al. (2016), *The New World Atlas of Artificial Night Sky Brightness*, GFZ Data Services, DOI `10.5880/GFZ.1.4.2016.001`.

The GFZ dataset is distributed under Creative Commons Attribution-NonCommercial 4.0 International (CC BY-NC 4.0). The bundled `world_atlas_3min.bin` remains subject to that licence and is not relicensed under PolyForm Noncommercial.

The official approximately 30-arcsecond 2015 artificial-zenith-luminance raster is averaged to an approximately 3-arcminute global lookup grid and stored in a compact logarithmic unsigned 16-bit representation. The bundled metadata records the transformation and checksum. The transformation and attribution notice is included in the Android APK as well as the Home Assistant/Docker images.

See `astronomy_observer/data/WORLD_ATLAS_NOTICE.md` and `docs/LIGHT_POLLUTION.md` for details.

Dataset: https://dataservices.gfz-potsdam.de/contact/showshort.php?id=escidoc:1541893

DOI: https://doi.org/10.5880/GFZ.1.4.2016.001

Licence: https://creativecommons.org/licenses/by-nc/4.0/


## Wikimedia Commons object thumbnails

Astronomy Observer bundles small target thumbnails for the Messier catalogue and a limited set of familiar planets, the Moon, the Milky Way and major meteor showers. The images are selected through the English Wikipedia PageImages API using its `free`-image filter and are then independently checked through the Wikimedia Commons Imageinfo `extmetadata` API before they are accepted.

The builder accepts only files reported as **Public domain, CC0, CC BY, or CC BY-SA**. Fair-use/non-free, NonCommercial, NoDerivatives, GFDL-only and unknown licences are rejected. Each accepted image keeps its own per-file licence and attribution; the image files are not relicensed under Astronomy Observer's PolyForm licence.

The bundled copy is a reduced WebP thumbnail. `astronomy_observer/data/object_images/manifest.json` records the original Commons file, creator, licence, source URL and the thumbnail transformation. `object-images/credits.html` is shipped with Home Assistant, Docker/web and Android so those credits remain accessible from the target image itself.

In Home Assistant and the standalone web edition, the target list may also request a missing preview or a clearer expanded copy at runtime. This uses the same Wikipedia/Commons path and the same licence allow-list. A dynamically displayed image retains its original Wikimedia licence and source; its Commons source/licence link is shown from the target UI. These optional runtime images are not added to the bundled application image and are not relicensed under PolyForm. Android keeps its embedded WebView local-only and does not use this runtime image path.

Wikimedia Commons: https://commons.wikimedia.org/

Wikimedia developer guidance for image licensing: https://foundation.wikimedia.org/wiki/Legal:Wikimedia_Developer_App_Guidelines

## Open-Meteo

Forecast and air-quality values are requested directly from Open-Meteo. The current free/public service configuration is intended for this project's non-commercial use. Open-Meteo documents API data under Creative Commons Attribution 4.0 International (CC BY 4.0) and requires attribution.

Astronomy Observer combines and transforms those forecast values into observing scores; the original weather provider remains visible in **Source status**. The Android licence screen also carries the attribution `Data from Open-Meteo`.

Service: https://open-meteo.com/

Licence/terms: https://open-meteo.com/en/license

CC BY 4.0: https://creativecommons.org/licenses/by/4.0/

## MET Norway

MET Norway Locationforecast is used as an independent weather fallback. Requests identify Astronomy Observer through the HTTP User-Agent, use bounded coordinate precision and are cached.

MET Norway documents its open data products under NLOD 2.0 and/or CC BY 4.0 unless a product states otherwise. Astronomy Observer credits MET Norway in the Android licence screen and keeps the provider visible in **Source status** when that forecast is used.

Service: https://api.met.no/weatherapi/locationforecast/2.0/documentation

Licence information: https://api.met.no/doc/License

CC BY 4.0: https://creativecommons.org/licenses/by/4.0/

## CelesTrak

Current general-perturbations orbital elements for the visual satellite group are obtained directly from CelesTrak. They are changing runtime data rather than APK-bundled catalogue data. Requests are cached and deliberately infrequent.

Service: https://celestrak.org/

## Minor Planet Center

Current comet orbital elements are obtained directly from the International Astronomical Union Minor Planet Center and propagated locally for target filtering. They are not bundled into the APK.

Service: https://www.minorplanetcenter.net/

## NOAA Space Weather Prediction Center

Aurora data are read directly from NOAA SWPC's OVATION aurora product and cached as changing runtime data.

Product: https://www.swpc.noaa.gov/products/aurora-30-minute-forecast

## International Meteor Organization

The bundled major-shower table is based on recurring information published in International Meteor Organization calendars. It is a planning aid: exact annual maxima and lunar circumstances can vary, so the current IMO calendar remains the reference for a dedicated meteor session.

Calendars: https://www.imo.net/resources/calendar/

## No endorsement

Attribution means identifying the source and preserving its terms; it does not imply that any provider above endorses Astronomy Observer.
