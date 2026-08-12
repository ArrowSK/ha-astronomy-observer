# Light pollution and sky brightness

Astronomy Observer estimates light pollution automatically from the selected Home Assistant person or Home location. No CSV, account or external light-pollution service is required for normal use.

The app includes a compact global derivative of the Falchi World Atlas. The source atlas is approximately 30 arcseconds; the bundled lookup grid is averaged to approximately 3 arcminutes. At every refresh, Astronomy Observer looks up the current location locally, converts the atlas artificial zenith luminance to an estimated moonless SQM-like sky brightness, and feeds that value into the observing-condition scores.

This means the headline score is location-sensitive by default. Moving from an urban location to a genuinely darker location can improve the darkness component, deep-sky score and imaging score even when the weather forecast is otherwise identical.

## Input priority

Astronomy Observer keeps the existing override options for observers who have better local information. The priority is:

1. fixed `sqm_override`, when set above zero;
2. a valid Home Assistant SQM sensor configured in `sqm_entity`;
3. an optional user-supplied local CSV grid;
4. the bundled location-based World Atlas estimate;
5. unknown only when none of the above can provide a value.

A real sky-quality meter is usually the best input because it can respond to current local lighting and atmospheric conditions that a static atlas cannot know.

## Built-in World Atlas lookup

The bundled file is `world_atlas_3min.bin`. It stores artificial zenith luminance in mcd/m² using a compact logarithmic unsigned 16-bit encoding. Its companion `world_atlas_3min.json` records the grid geometry, source information, encoding, file size and checksum.

The runtime does not load the approximately 42 MB grid into memory. A normal lookup reads one cell. A nearby darker-area search reads only the relevant rows and keeps a single row buffer, so the extra steady-state memory use remains small.

The atlas is a 2015 baseline. It cannot know about later changes to street lighting, temporary lights, snow cover, local shielding, cloud amplification or individual obstructions. Treat the SQM value as a planning estimate rather than a live measurement.

The source raster covers approximately 60° S to 85° N. Outside that range, or where the source has no usable value, Astronomy Observer reports the sky-brightness input as unavailable unless one of the higher-priority inputs is configured.

## Atlas-to-SQM estimate

The World Atlas provides artificial sky brightness rather than total natural-plus-artificial sky brightness. Astronomy Observer combines the atlas value with a fixed natural reference of 0.174 mcd/m² for its planning estimate:

```text
total = 0.174 + artificial_mcd_m2
SQM estimate = 22.0 - 2.5 log10(total / 0.174)
```

The stored atlas value itself is not altered by this natural-sky reference; the addition is made only when calculating the displayed SQM-style estimate.

## Fixed SQM value

For a regular observing site with a representative moonless zenith measurement, set:

```yaml
sqm_override: 20.7
```

A fixed value takes priority over every other light-pollution input. It is useful for a stable site but does not follow a moving observer.

## Home Assistant SQM sensor

If a sky-quality meter already exists in Home Assistant and its state is in mag/arcsec², set:

```yaml
sqm_override: 0
sqm_entity: sensor.observatory_sqm
```

The app accepts values between 15 and 23 mag/arcsec². Invalid, unavailable or non-numeric states are ignored and the app continues through the remaining fallbacks.

## Optional higher-resolution local CSV

The old CSV feature remains available as an override for observers who want to use a local grid at higher resolution than the bundled approximately 3-arcminute atlas. Nothing needs to be created for ordinary installations.

The format is:

```text
latitude,longitude,artificial_mcd_m2
47.5000,19.0000,2.134
47.5100,19.0000,1.981
...
```

The app finds the nearest point within 10 km. If the local grid does not contain a usable nearby point, it falls back to the bundled atlas automatically.

The file is streamed line by line rather than loaded into memory.

### Where to put a custom CSV

The app maps its own Home Assistant app-configuration folder read-only at `/config`. Put the CSV there and set or keep:

```yaml
light_pollution_file: light_pollution.csv
```

Do not create an empty placeholder file.

### Creating a local CSV from the source GeoTIFF

The repository includes [`tools/light_pollution_tile.py`](../tools/light_pollution_tile.py). It creates a local CSV from a GeoTIFF obtained from the official GFZ dataset.

Run this on a desktop computer rather than on Home Assistant. Install Rasterio:

```bash
python3 -m pip install rasterio
```

Example:

```bash
python3 tools/light_pollution_tile.py \
  --input World_Atlas_2015.tif \
  --output light_pollution.csv \
  --latitude 47.50 \
  --longitude 19.05 \
  --radius-km 100 \
  --step 1
```

`--step 1` keeps every source pixel in the selected area. Increase the step for a smaller file over a larger search radius.

## Nearby darker area

The same location data are also used for `nearby_dark_site_radius_km`. Astronomy Observer searches the atlas around the current observer and returns a candidate only when the artificial zenith luminance is materially lower than at the current position.

The result is deliberately described as a darker area or atlas point, not as a vetted observing site. A dark raster cell may be private property, inaccessible, unsafe, wooded, mountainous or otherwise unsuitable. Check access, terrain and local rules separately.

A custom local CSV takes priority for this search when it supplies the current sky-brightness value. Otherwise the bundled atlas is used automatically.

## How light pollution affects the score

The estimated SQM value feeds the darkness component. Darkness is part of the overall, deep-sky and imaging condition scores, so the main score shown when Astronomy Observer opens already reflects the selected location's light pollution. Planetary observing is affected much less because bright planets can remain useful from a bright site.

The score still keeps weather, transparency, Moon conditions, seeing estimate, wind and dew as separate dimensions. A dark site does not turn an overcast night into a good observing night, and a bright city does not make bright-planet observing impossible.

See [SCORING.md](SCORING.md) for the exact current weighting.

## Licence and attribution

The World Atlas dataset is distributed by GFZ Data Services under Creative Commons Attribution-NonCommercial 4.0 International (CC BY-NC 4.0):

Falchi, F. et al. (2016), *The New World Atlas of Artificial Night Sky Brightness*, GFZ Data Services, DOI 10.5880/GFZ.1.4.2016.001.

Astronomy Observer redistributes a downsampled data derivative, not the original GeoTIFF. The derivative retains the dataset's CC BY-NC 4.0 terms. See [`astronomy_observer/data/WORLD_ATLAS_NOTICE.md`](../astronomy_observer/data/WORLD_ATLAS_NOTICE.md) and [`THIRD_PARTY_LICENSES.md`](../THIRD_PARTY_LICENSES.md).
