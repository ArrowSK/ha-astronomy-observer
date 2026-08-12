# Third-party code and data

Astronomy Observer keeps third-party licensing separate from the project licence. The PolyForm Noncommercial licence applies to original project code, not to third-party material that retains the terms listed below.

## Astronomy Engine

The build downloads the C implementation of Astronomy Engine from commit `61dc07020aaa6885d2c7f688a4d82beaf6edb9ef` (release `v2.1.19`) in `cosinekitty/astronomy`.

Astronomy Engine is copyright Don Cross and is distributed under the MIT License. The downloaded source files carry the MIT notice and are compiled without relicensing.

Upstream: https://github.com/cosinekitty/astronomy

## OpenNGC

The build downloads OpenNGC from commit `da90466031b0372c896588b85be6016c617e205b` and creates a smaller observing catalogue containing Messier cross-references and brighter/large visually useful NGC/IC objects.

OpenNGC is credited to Mattia Verga and licensed under Creative Commons Attribution-ShareAlike 4.0. The generated catalogue is a data derivative under CC BY-SA 4.0; it is not relicensed under PolyForm Noncommercial.

Upstream: https://github.com/mattiaverga/OpenNGC

## World Atlas of Artificial Night Sky Brightness

Astronomy Observer can read user-prepared grids derived from:

Falchi, F. et al. (2016), *The New World Atlas of Artificial Night Sky Brightness*, GFZ Data Services, DOI 10.5880/GFZ.1.4.2016.001.

The GFZ dataset is distributed under CC BY-NC 4.0. It is not bundled in this repository or downloaded by the app. Any grid derived from it remains subject to the dataset licence and attribution requirements.

Dataset: https://dataservices.gfz-potsdam.de/contact/showshort.php?id=escidoc:1541893

## Open-Meteo

Forecast and air-quality data are requested from Open-Meteo's public non-commercial APIs. Data and API use remain subject to Open-Meteo's published terms and attribution requirements.

Service: https://open-meteo.com/

## MET Norway

MET Norway Locationforecast is used as an independent weather fallback. Requests identify this project through the HTTP User-Agent and are cached.

Service: https://api.met.no/weatherapi/locationforecast/2.0/documentation

## CelesTrak

Current general perturbations orbital elements for the visual satellite group are obtained from CelesTrak. Requests are cached and kept deliberately infrequent.

Service: https://celestrak.org/

## Minor Planet Center

Current comet orbital elements are obtained from the International Astronomical Union Minor Planet Center and propagated locally for target filtering.

Service: https://www.minorplanetcenter.net/

## NOAA Space Weather Prediction Center

Aurora data are read from NOAA SWPC's OVATION aurora product.

Product: https://www.swpc.noaa.gov/products/aurora-30-minute-forecast

## International Meteor Organization

The bundled major-shower table is based on recurring information published in International Meteor Organization calendars. Exact annual maxima and lunar circumstances can vary; the current IMO calendar remains the reference for a dedicated meteor session.

Calendars: https://www.imo.net/resources/calendar/
