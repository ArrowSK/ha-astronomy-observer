# Light-pollution and SQM setup

Astronomy Observer does not guess local light pollution from a city name. Use one of the three supported methods when you want sky brightness included in the score.

## 1. Fixed SQM value

For a regular observing site with a representative moonless zenith measurement, set:

```yaml
sqm_override: 20.7
```

This takes priority over every other light-pollution input.

A fixed value is useful for a stable site but it cannot follow a moving `person` entity.

## 2. Home Assistant SQM sensor

If a sky-quality meter already exists in Home Assistant and its state is in mag/arcsec², set:

```yaml
sqm_override: 0
sqm_entity: sensor.observatory_sqm
```

The app accepts values between 15 and 23 mag/arcsec². Invalid, unavailable or non-numeric states are ignored and the app continues to the local-grid fallback.

A real local meter is the best available input because it can respond to local lighting and atmospheric conditions that a static atlas cannot know.

## 3. Local atlas grid

The app can stream a CSV with:

```text
latitude,longitude,artificial_mcd_m2
47.5000,19.0000,2.134
47.5100,19.0000,1.981
...
```

The third value is **artificial zenith luminance** in mcd/m², matching the quantity distributed in the Falchi/GFZ atlas. The app finds the nearest point within 10 km. It also scans points within `nearby_dark_site_radius_km` to find a materially darker candidate.

The file is streamed line by line; it is not loaded into memory.

### Where to put the file

The app maps its own Home Assistant app-configuration folder read-only at `/config`. Put the CSV there and set:

```yaml
light_pollution_file: light_pollution.csv
```

The exact host-side folder name includes Home Assistant's repository identifier and the app slug. Home Assistant's app configuration documentation explains the `addon_config` mount if you need to locate it manually.

## Creating a grid from the Falchi GeoTIFF

The repository includes [`tools/light_pollution_tile.py`](../tools/light_pollution_tile.py). It creates a local CSV from a GeoTIFF you have obtained from the official GFZ dataset.

Install Rasterio on a desktop computer, not on Home Assistant:

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

`--step 1` keeps every source pixel in the selected area. Increase the step to 2 or 3 if you want a smaller file for a very large search radius.

The tool reads only the requested window and writes latitude/longitude plus artificial luminance. It does not upload the atlas or your selected location anywhere.

## Atlas-to-SQM estimate

When an atlas value is used, the app reports an estimated SQM-like moonless zenith brightness by combining the atlas artificial luminance with a fixed natural reference of 0.174 mcd/m²:

```text
total = 0.174 + artificial_mcd_m2
SQM estimate = 22.0 - 2.5 log10(total / 0.174)
```

This conversion is useful for ranking but should not be confused with a live SQM measurement. The Falchi atlas models artificial zenith brightness at the atlas epoch; it does not know tonight's cloud, snow, local luminaire changes or temporary lighting.

## Nearby darker point

When the imported grid covers a radius around the current location, the app returns the darkest grid point within the configured straight-line radius only if it is materially darker than the current grid point.

This is **not** a recommendation that the point is accessible, legal, safe, public, drivable or suitable for setting up equipment. Check access and terrain separately.

## Licence

The World Atlas dataset is distributed by GFZ Data Services under CC BY-NC 4.0. The repository does not redistribute it. A grid you derive from it remains subject to the dataset's licence and attribution requirements.
