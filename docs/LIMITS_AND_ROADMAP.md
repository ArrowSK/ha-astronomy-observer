# Known limits and roadmap

The first release covers the full observing-decision path, but several areas are intentionally conservative rather than pretending to a precision the inputs cannot support.

## Known limits

### Seeing is a proxy

General weather products do not provide a site-calibrated optical-turbulence profile. The current seeing dimension uses upper-air and surface wind as a proxy and stays on a 0–100 scale. It does not report arcseconds.

### Static light-pollution atlas

The Falchi atlas is a baseline model of artificial zenith brightness. It cannot know about recent lighting changes, snow reflection, local floodlights or tonight's aerosols. A real SQM sensor is preferable when available.

### Generic imaging score

The imaging score does not yet distinguish broadband, narrowband, lunar/planetary video, focal length or image scale.

### Equipment model

Aperture is used as a broad feasibility gate. Focal length, eyepiece field stop, camera sensor, filters and mount limits are not yet modelled.

### Comet precision

Comets use a local two-body propagation of MPC elements. This is sufficient for ranking likely opportunities, not for high-precision pointing of rapidly changing or very close objects.

### Satellite brightness

Visible satellite candidates are selected from CelesTrak's visual group and checked for approximate illumination and altitude. Optical magnitude is not predicted.

### Meteor annual timing

The built-in shower table uses recurring date ranges and nominal peaks. Exact annual maxima and unusual activity should be checked against the current IMO calendar.

### Local horizon is manual

The horizon mask must currently be entered as azimuth/altitude points. There is no terrain-horizon download or graphical editor yet.

## Planned work

The following additions fit the current architecture without changing the basic score philosophy:

1. Site calibration history: compare forecast components against the observer's own notes/SQM measurements without sending calibration data away.
2. Graphical horizon editor in the Ingress page.
3. Multiple named equipment profiles with focal length, eyepieces, cameras and filters.
4. Field-of-view and framing checks for extended targets.
5. Broadband vs narrowband vs planetary-imaging condition profiles.
6. More rigorous lunar sky-brightness calculation exposed separately from the current interference index.
7. Optional terrain horizon generated from a user-supplied elevation model.
8. Better annual meteor-shower data update tooling, including radiant drift and exact maximum times.
9. Higher-precision comet ephemeris option for users who explicitly enable it.
10. Calibration summaries that compare saved observing notes with forecast bias by site and season.
11. Lunar-feature recommendations based on colongitude and terminator geometry.
12. Configurable target-category preferences while retaining diversity in the default Top 10.

A roadmap item should not be added to the runtime merely because data exist for it. It should have a clear observing use, a documented source, a bounded resource cost and a failure mode that does not compromise the rest of the app.
