# Astronomy Observer

Astronomy Observer evaluates observing conditions for a Home Assistant location and ranks useful targets for the night ahead.

It is built for observers who want to see why a night scores well or badly. Cloud, transparency, estimated seeing, darkness, Moon interference, wind, dew risk and confidence remain separate, while target recommendations account for altitude, horizon obstruction, sky brightness, Moon separation and configured equipment.

After the first successful refresh, open the Astronomy Observer panel and press **Setup** to choose the observer and a simple lowest-useful-altitude horizon. The advanced directional horizon remains available for sites that genuinely need it.

Light pollution is automatic. Astronomy Observer follows the selected Home Assistant person or Home location and reads the bundled approximately 3-arcminute World Atlas grid locally. That sky-brightness estimate is included in the main condition score without any CSV or external light-pollution service. A fixed SQM value, Home Assistant SQM sensor or higher-resolution local CSV can still override the built-in estimate.

Open the app's **Documentation** tab for the full setup guide, entity names, dashboard preset, light-pollution details and scoring method.
