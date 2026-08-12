# Scientific and technical references

This file records the main references behind calculations and data interpretation. It is not a claim that every scoring coefficient is directly fitted from these papers; heuristic transformations are identified as such in `SCORING.md`.

## Astronomical geometry

- Don Cross, **Astronomy Engine**. Project documentation and source: https://github.com/cosinekitty/astronomy
- Jean Meeus, **Astronomical Algorithms**, 2nd edition, Willmann-Bell. General reference for positional astronomy methods.

## Airmass

- Kasten, F. & Young, A. T. (1989), *Revised optical air mass tables and approximation formula*, Applied Optics 28, 4735–4738. DOI: 10.1364/AO.28.004735.

Astronomy Observer uses the commonly cited Kasten-Young approximation for target airmass.

## Artificial night-sky brightness

- Falchi, F. et al. (2016), *The New World Atlas of Artificial Night Sky Brightness*, Science Advances 2(6), e1600377. DOI: 10.1126/sciadv.1600377.
- Falchi, F. et al. (2016), World Atlas data, GFZ Data Services. DOI: 10.5880/GFZ.1.4.2016.001.

The runtime uses only user-supplied regional extracts or a user-configured SQM value; the full atlas is not bundled.

## Moonlit sky brightness

- Krisciunas, K. & Schaefer, B. E. (1991), *A Model of the Brightness of Moonlight*, Publications of the Astronomical Society of the Pacific 103, 1033–1039.

The current release uses a simpler separation/illumination/altitude observing penalty rather than claiming a full implementation of the paper's radiance model. This reference is retained for the planned validated moonlight model.

## Seeing forecasts

Astronomical seeing requires optical-turbulence information that ordinary surface weather variables do not provide directly. The current release therefore publishes only a relative weather-model proxy and does not convert it to FWHM arcseconds.

## Satellite propagation

- Vallado, D. A. et al., *Revisiting Spacetrack Report #3*, AIAA/AAS Astrodynamics Specialist Conference.
- Rust `sgp4` implementation documentation: https://docs.rs/sgp4/
- CelesTrak GP data documentation: https://celestrak.org/NORAD/documentation/gp-data-formats.php

## Catalogue and transient data

- OpenNGC: https://github.com/mattiaverga/OpenNGC
- Minor Planet Center: https://www.minorplanetcenter.net/
- International Meteor Organization: https://www.imo.net/
- NOAA Space Weather Prediction Center OVATION product: https://www.swpc.noaa.gov/products/aurora-30-minute-forecast
