# Light-pollution and SQM setup

Astronomy Observer works without any light-pollution file. If you do nothing, sky brightness stays unknown, the parts of the score that depend on local darkness are deliberately conservative, and the confidence value reflects the missing input.

For most users there is no reason to prepare a CSV on first installation. Use one of the options below only when you want a more accurate local darkness estimate or the nearby darker-point search.

## Easiest options

### Fixed SQM value

For a regular observing site with a representative moonless zenith measurement, set:

```yaml
sqm_override: 20.7
```

This takes priority over every other light-pollution input. A fixed value is useful for a stable site but it does not follow a moving observer.

### Home Assistant SQM sensor

If a sky-quality meter already exists in Home Assistant and its state is in mag/arcsec², set:

```yaml
sqm_override: 0
sqm_entity: sensor.observatory_sqm
```

The app accepts values between 15 and 23 mag/arcsec². Invalid, unavailable or non-numeric states are ignored and the app continues to the atlas-grid fallback if one is available.

A real local meter is the best input because it can respond to local lighting and atmospheric conditions that a static atlas cannot know.

## Optional Falchi atlas CSV

Use the CSV only when you specifically want a local grid based on the Falchi World Atlas. The app expects:

```text
latitude,longitude,artificial_mcd_m2
47.5000,19.0000,2.134
47.5100,19.0000,1.981
...
```

The third value is artificial zenith luminance in mcd/m². The app finds the nearest point within 10 km and can also scan points within `nearby_dark_site_radius_km` for a materially darker candidate.

The file is streamed line by line rather than loaded into memory.

### Where to put the file

The app maps its own Home Assistant app-configuration folder read-only at `/config`. Put the CSV there and keep the default:

```yaml
light_pollution_file: light_pollution.csv
```

If you do not already have a CSV, leave this setting alone. The app does not require an empty placeholder file.

## Creating a grid from the Falchi GeoTIFF

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

`--step 1` keeps every source pixel in the selected area. Increase the step to 2 or 3 for a smaller file over a very large search radius.

The tool reads only the requested window and writes latitude/longitude plus artificial luminance. It does not upload the atlas or the selected location.

## Atlas-to-SQM estimate

When an atlas value is used, the app reports an estimated SQM-like moonless zenith brightness by combining the atlas artificial luminance with a fixed natural reference of 0.174 mcd/m²:

```text
total = 0.174 + artificial_mcd_m2
SQM estimate = 22.0 - 2.5 log10(total / 0.174)
```

This is useful for ranking but is not a live SQM measurement. The atlas represents artificial zenith brightness at its source epoch and does not know tonight's cloud, snow, local luminaire changes or temporary lighting.

## Nearby darker point

When the imported grid covers a radius around the current location, the app returns the darkest grid point within the configured straight-line radius only if it is materially darker than the current point.

This is not a statement that the point is accessible, legal, safe, public, drivable or suitable for setting up equipment. Check access and terrain separately.

## Licence

The World Atlas dataset is distributed by GFZ Data Services under CC BY-NC 4.0. The repository does not redistribute the atlas. A grid derived from it remains subject to the dataset's licence and attribution requirements.
