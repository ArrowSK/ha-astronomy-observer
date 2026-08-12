# Bundled World Atlas data notice

Astronomy Observer includes a compact location-lookup derivative of the World Atlas of Artificial Night Sky Brightness so that light pollution can be estimated automatically from the selected Home Assistant location.

Source:

Falchi, F. et al. (2016), *The New World Atlas of Artificial Night Sky Brightness*, GFZ Data Services, DOI 10.5880/GFZ.1.4.2016.001.

Dataset licence: Creative Commons Attribution-NonCommercial 4.0 International (CC BY-NC 4.0).

Dataset page: https://dataservices.gfz-potsdam.de/contact/showshort.php?id=escidoc:1541893

Licence text: https://creativecommons.org/licenses/by-nc/4.0/

The bundled file `world_atlas_3min.bin` was made from the official 2015 GeoTIFF. The source grid is approximately 30 arcseconds. Astronomy Observer averages it to approximately 3 arcminutes for the zero-configuration global lookup, then logarithmically quantises artificial zenith luminance into unsigned 16-bit values. The stored quantity remains artificial zenith luminance in mcd/m². Natural sky brightness is not stored in the atlas derivative.

The corresponding `world_atlas_3min.json` records the grid geometry, encoding, source citation, file size and SHA-256 digest. The transformation is reproducible with `scripts/build_global_atlas.py` and the repository workflow that builds the bundled atlas from the official GFZ download.

The PolyForm Noncommercial licence covering original Astronomy Observer code does not replace the World Atlas data licence. The bundled atlas derivative remains subject to CC BY-NC 4.0 and the attribution above.
