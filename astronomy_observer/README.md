# Astronomy Observer

Astronomy Observer evaluates observing conditions for a Home Assistant location and ranks useful targets for the night ahead.

It is built for observers who want to see why a night scores well or badly. The Ingress view separates observing-quality scores from raw forecast conditions, and every condition row can be expanded in place for a concise explanation of how that quantity is used. Target recommendations continue to account for altitude, horizon obstruction, sky brightness, Moon separation and configured equipment.

After the first successful refresh, open the Astronomy Observer panel and open the hamburger menu, choose **Setup**, and select the observer and a simple lowest-useful-altitude horizon. Saving closes Setup automatically and starts a recalculation. The advanced directional horizon remains available for sites that genuinely need it.

Light pollution is automatic. Astronomy Observer follows the selected Home Assistant person or Home location and reads the bundled approximately 3-arcminute World Atlas grid locally. That sky-brightness estimate is included in the main condition score without any CSV or external light-pollution service. A fixed SQM value, Home Assistant SQM sensor or higher-resolution local CSV can still override the built-in estimate.

The header keeps the refresh icon directly available and places **Setup** and the **Observation journal** in a hamburger menu with generous touch spacing. The journal uses a document icon. A persistent bottom navigation bar jumps between Tonight, Conditions, Targets, Outlook and Sources. Observation history is collapsed by default and can be searched or filtered when opened. The dashboard YAML copy action lives inside Setup because it is normally only needed during initial configuration.

Open the app's **Documentation** tab for the full setup guide, entity names, dashboard preset, light-pollution details and scoring method.
