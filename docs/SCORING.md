# Scoring method

Astronomy Observer uses 0–100 scores as decision aids. A high score means the model expects favourable observing conditions for the stated use; it does not mean a 90-point night is physically "90% good".

The score is intentionally inspectable. Every major component is published separately, and the raw inputs for the best window are shown in the Ingress page.

## Principles

The current model follows five rules:

1. Cloud can veto an otherwise excellent night.
2. Deep-sky work depends strongly on transparency, darkness and lunar conditions.
3. Planetary work depends more strongly on the seeing proxy and target altitude.
4. A target is not recommended merely because it is above the mathematical horizon.
5. Missing data reduce confidence instead of being replaced with an invented precise value.

The weights below are empirical engineering choices for practical observing. They are not presented as a published standard, and they should be changed only with tests and a documented reason.

## Cloud factor

When total cloud cover `C` is available:

```text
clear = (1 - C/100)^1.45
```

High cloud and low cloud then apply smaller additional penalties:

```text
high factor = (1 - high_cloud/100)^0.25
low factor  = (1 - low_cloud/100)^0.15
cloud factor = clear × high factor × low factor
```

The exponents make total cloud the dominant term while retaining sensitivity to thin upper cloud and low cloud/fog that can be especially troublesome for observing.

If total cloud is unavailable, the maximum available layer fraction is used. If no cloud information is available, the cloud factor is deliberately mediocre and confidence is reduced.

## Dew score

The preferred input is the temperature-to-dew-point margin:

```text
margin = temperature - dew point
```

Current mapping:

- 6 °C or more: full dew score;
- 3–6 °C: mildly reduced;
- 1–3 °C: increasingly poor;
- below 1 °C: high dew risk.

When dew point is unavailable, relative humidity is used as a weaker fallback and confidence is reduced.

This is an observing-risk indicator, not a physical prediction of when a particular corrector plate, mirror or camera body will reach the dew point. Equipment thermal state and local radiative cooling matter.

## Transparency

Transparency combines the atmospheric variables that are available for a given provider:

- visibility: 35% of the available transparency inputs;
- aerosol optical depth: 35%;
- relative humidity: 20%;
- PM2.5: 10%;
- dew factor: an additional 15% term in the final weighted mean.

The current transforms are:

```text
visibility factor = clamp((visibility_km - 3) / 27, 0, 1)
aerosol factor    = clamp(exp(-3 × AOD), 0.05, 1)
humidity factor   = clamp((105 - RH) / 35, 0, 1)
PM2.5 factor      = clamp(exp(-PM2.5 / 45), 0.1, 1)
```

Only available terms are averaged. This is why transparency can still be calculated from the MET Norway fallback, but its confidence is lower because visibility and aerosol fields are not available there in the same form.

Aerosol optical depth is useful because it describes column aerosol extinction more directly than surface humidity alone. Open-Meteo exposes AOD from its air-quality data source.

## Estimated seeing

The seeing score is explicitly a **proxy**. It is not a DIMM/MASS measurement and is not converted to arcseconds.

The model currently uses:

- 200 hPa wind: 45% of the available seeing evidence;
- 500 hPa wind: 35%;
- surface wind: 20%.

High upper-air wind is penalised progressively. Very calm surface air is also given a small penalty because strong near-ground stability can coexist with poor local seeing; moderate surface wind is treated more favourably until mechanical wind begins to dominate.

This is the least physically complete part of the atmospheric model. A proper optical-turbulence profile requires information that general weather products do not directly provide. The app therefore reports **Estimated Seeing 0–100** and a separate confidence contribution rather than a fabricated FWHM value.

## Darkness

Solar altitude controls the first part of the darkness factor:

```text
Sun ≥ -6°        0
-6° to -12°      rises from 0 to 0.35
-12° to -18°     rises from 0.35 to 1
Sun ≤ -18°       1
```

When sky brightness is known, the moonless-site term is derived from SQM-style brightness:

```text
site factor = clamp((SQM - 17.0) / 4.7, 0, 1)
```

The site term is softened before multiplication so that a bright urban site does not imply that all bright-object observing is impossible.

If sky brightness is unknown, the darkness dimension uses a neutral fallback and its confidence contribution is reduced.

## Moon interference

The general Moon-interference score uses:

- illuminated fraction;
- Moon altitude.

A Moon below roughly -5° contributes no general penalty. Above the horizon, the penalty grows with illuminated fraction and altitude.

Target ranking adds the important missing geometric term: angular separation between the target and the Moon. Faint galaxies and dark/low-surface-brightness objects are more sensitive than double stars, clusters or compact planetary nebulae.

This is an interference index rather than a full spectral sky-brightness calculation. The implementation follows the observational fact that lunar sky glow depends on phase, altitude and target separation; a future version may expose a calibrated lunar sky-brightness model separately.

Reference for the physical basis of lunar sky-brightness modelling: Krisciunas & Schaefer (1991), *A model of the brightness of moonlight*, PASP 103, 1033.

## Wind score

Surface wind is mapped conservatively:

- up to 8 km/h: full score;
- 8–20 km/h: gradual penalty;
- 20–40 km/h: stronger penalty;
- over 40 km/h: poor score.

This is not an equipment-specific safety limit. Large Dobsonians, long imaging trains and exposed sites can have much lower practical wind limits.

## Composite scores

Composite scores use a weighted geometric mean rather than a simple arithmetic mean. This prevents one excellent variable from fully compensating for one disastrous variable.

### Overall

```text
cloud            35%
transparency     22%
darkness         18%
Moon quality     10%
wind              7%
dew               8%
```

### Deep sky

```text
cloud            34%
transparency     27%
darkness         24%
Moon quality     10%
wind              5%
```

### Planetary

```text
cloud            40%
estimated seeing 32%
transparency     10%
wind             12%
darkness          6%
```

Transparency and darkness have floors in the planetary score because bright planets can remain worthwhile under conditions that would be poor for faint deep-sky targets.

### Imaging

```text
cloud            40%
transparency     22%
estimated seeing 13%
darkness         16%
wind              9%
```

The current imaging score is generic. It does not yet distinguish broadband, narrowband, planetary video or focal length.

## Best observing window

For the configured search horizon, the app evaluates rolling two-hour windows. A window is eligible only when the Sun is below -6° at its start. The window with the highest average overall score is selected.

The component scores published as the "best window" are averages across that selected period. Dew margin uses the minimum margin found in the window because the worst point is the operationally useful one.

## Airmass and horizon

Targets are scored only when they are above both:

- the configured lowest useful altitude; and
- the optional directional horizon at that azimuth.

For normal use the single lowest-useful-altitude value is enough. The directional mask is an advanced override for sites where buildings, trees or terrain differ substantially by direction.

Airmass uses the Kasten-Young approximation:

```text
X = 1 / (sin(h) + 0.50572 × (h + 6.07995)^-1.6364)
```

where `h` is apparent altitude in degrees. Airmass then penalises targets close to the horizon even when they have technically cleared the limit.

Reference: Kasten, F. & Young, A. T. (1989), *Revised optical air mass tables and approximation formula*, Applied Optics 28(22), 4735–4738.

## Deep-sky target ranking

For each catalogue target and each candidate time, the score combines:

- the deep-sky condition score;
- airmass;
- target-specific lunar interference;
- local sky-brightness sensitivity by object type;
- integrated magnitude when available;
- surface brightness when available;
- a small priority for Messier/common-name objects so the list remains observationally useful;
- aperture-based feasibility.

The catalogue is deliberately compact rather than a dump of every NGC/IC row. The build keeps all Messier cross-references and selects brighter or large visually relevant objects from OpenNGC.

OpenNGC stores right ascension as `HH:MM:SS.SS` and declination as signed `DD:MM:SS.SS`. The image build converts those J2000 coordinates to decimal degrees before the runtime catalogue is written. The build also maps V magnitude, B magnitude, galaxy surface brightness and major-axis size into explicit columns. A self-test covers the coordinate conversion so a schema mismatch cannot silently turn the deep-sky catalogue into an empty recommendation set.

### Aperture gate

The current aperture gate uses a broad limiting-magnitude relation:

```text
limiting magnitude ≈ 2 + 5 log10(aperture_mm)
```

It is used only to reject clearly impractical targets, not to predict whether a particular observer will detect an object. Surface brightness, magnification, sky brightness, visual acuity and experience can dominate actual detectability.

## Milky Way / Galactic Centre

The Galactic Centre uses the J2000 position of Sagittarius A* as a stable reference point. It is considered only under nautical darkness or better, then scored from deep-sky conditions, airmass, Moon geometry and local sky brightness.

## Meteor showers

The bundled shower table contains recurring activity periods, approximate peak dates, radiant coordinates, nominal ZHR and geocentric speed for major showers. Target scoring then adds:

- radiant altitude;
- current observing-condition score;
- Moon interference;
- approximate activity strength around the nominal peak.

The bundled CSV schema is covered by a runtime unit test. This matters because a malformed column mapping would otherwise remove every shower from the candidate list without affecting the rest of the app.

Actual annual peak timing, radiant drift and outbursts can differ. For a serious meteor session, check the current International Meteor Organization calendar as well.

## Comets

Current Minor Planet Center orbital elements are propagated locally using a two-body solution. The app estimates total magnitude from the supplied `H` and slope parameters when present, then applies altitude, Moon and deep-sky condition penalties.

Comet photometry and non-gravitational motion can depart substantially from a simple prediction. The recommendation is a planning filter, not a precision ephemeris for acquisition or imaging.

Objects with an `A/` designation are excluded from the comet recommendations because that designation is used for asteroid-like objects rather than an active comet observing recommendation.

## Satellites

CelesTrak visual-group orbital elements are propagated locally with SGP4. The pass score considers:

- cloud score;
- altitude above the local horizon;
- whether the satellite is approximately sunlit while the observer is in darkness.

The visual group is intended to keep the calculation small and relevant. Magnitude is not predicted because reliable optical brightness depends strongly on attitude and geometry. For that reason satellite scores are deliberately reduced before the final Top 10 is selected. Rocket bodies and debris receive the strongest reduction; familiar high-interest spacecraft such as the ISS receive a smaller one. This prevents an accurately predicted high pass with unknown brightness from outranking genuinely strong astronomical targets merely because its altitude is easy to score.

## Final Top 10 selection

The final list is not a raw sort of every candidate class. After each target has its own observing score, Astronomy Observer applies category limits so one class cannot crowd out the observing programme.

Current maximums are:

```text
deep sky        6
planets         3
Moon            1
comet           1
satellite       1
meteor shower   1
Milky Way       1
aurora          1
```

The limits are ceilings, not quotas. The app does not force a weak comet, satellite or any other class into the list. Candidates below the minimum usefulness threshold are dropped, and the list may contain fewer than ten entries when the night or configured equipment does not support ten worthwhile choices.

This diversity stage is intentionally separate from the physical condition score. It exists to answer a practical observing question: "what are the most worthwhile things to spend time on tonight?" rather than "which ten independently scored rows have the largest numbers?"

## Data confidence

Confidence is a weighted measure of how much of the expected input set is present:

```text
cloud quality        20.0%
transparency inputs  25.0%
seeing inputs        15.0%
sky-brightness input 15.0%
Moon data            10.0%
wind                   7.5%
dew                    7.5%
```

A lower confidence score does not automatically mean poor observing. It means the numerical recommendation is based on less complete evidence.
