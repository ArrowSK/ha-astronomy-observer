#!/usr/bin/env python3
"""Apply the small, isolated source changes needed for bundled target thumbnails.

This script exists so the one-time asset-generation workflow can patch large source
files without replacing unrelated working code by hand. Every edit is anchored to a
known-good marker and is idempotent.
"""

from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str, text: str) -> None:
    (ROOT / path).write_text(text, encoding="utf-8")


def replace_once(text: str, old: str, new: str, label: str) -> str:
    if new in text:
        return text
    if old not in text:
        raise SystemExit(f"thumbnail patch marker changed: {label}")
    return text.replace(old, new, 1)


def insert_before(text: str, marker: str, addition: str, label: str) -> str:
    if addition.strip() in text:
        return text
    if marker not in text:
        raise SystemExit(f"thumbnail insertion marker changed: {label}")
    return text.replace(marker, addition + marker, 1)


def patch_index() -> None:
    path = "astronomy_observer/web/index.html"
    text = read(path)
    old_css = """    .target { display: grid; grid-template-columns: 30px minmax(0, 1fr) 58px; gap: 10px; padding: 11px 0; border-top: 1px solid var(--line); }
    .target:first-of-type { border-top: 0; padding-top: 0; }
    .rank { color: var(--muted); font-variant-numeric: tabular-nums; }
    .target-name { font-weight: 620; }
    .target-meta { color: var(--muted); font-size: 12px; margin-top: 2px; }
    .target-score { text-align: right; font-size: 19px; font-variant-numeric: tabular-nums; }
"""
    new_css = """    .target { display: grid; grid-template-columns: 30px 58px minmax(0, 1fr) 58px; gap: 10px; align-items: start; padding: 11px 0; border-top: 1px solid var(--line); }
    .target:first-of-type { border-top: 0; padding-top: 0; }
    .rank { color: var(--muted); font-variant-numeric: tabular-nums; padding-top: 4px; }
    .target-thumb-wrap { width: 54px; height: 54px; position: relative; display: grid; place-items: center; overflow: hidden; border: 1px solid var(--line); border-radius: 9px; background: linear-gradient(145deg, #172238, #0d1422); }
    .target-thumb { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: cover; display: block; }
    .target-thumb-fallback { color: #7183a1; font-size: 25px; line-height: 1; }
    .target-credit { position: absolute; right: 2px; bottom: 2px; width: 23px; height: 23px; display: grid; place-items: center; border-radius: 50%; background: rgba(9,13,22,.84); color: #d8e4f8; border: 1px solid rgba(255,255,255,.28); text-decoration: none; font-size: 12px; font-weight: 700; backdrop-filter: blur(5px); }
    .target-credit:hover { background: #1a2840; color: var(--accent); }
    .target-name { font-weight: 620; }
    .target-meta { color: var(--muted); font-size: 12px; margin-top: 2px; }
    .target-score { text-align: right; font-size: 19px; font-variant-numeric: tabular-nums; padding-top: 2px; }
"""
    text = replace_once(text, old_css, new_css, "target CSS")

    responsive = """
    @media (max-width: 560px) {
      .target { grid-template-columns: 24px 48px minmax(0, 1fr) 42px; gap: 8px; }
      .target-thumb-wrap { width: 46px; height: 46px; border-radius: 8px; }
      .target-credit { width: 21px; height: 21px; font-size: 11px; }
      .target-score { font-size: 17px; }
    }
"""
    text = insert_before(text, "  </style>", responsive, "target mobile CSS")

    old_render = """  $('target-list').innerHTML = snapshot.recommendations.length ? snapshot.recommendations.map((r, i) => `
    <div class=\"target\">
      <div class=\"rank\">${i + 1}</div>
      <div>
        <div class=\"target-name\">${esc(r.name)}</div>
        <div class=\"target-meta\">${esc(r.category)} · ${localTime(r.best_time)} · alt ${n(r.altitude_deg,0,'°')} · az ${n(r.azimuth_deg,0,'°')} · ${esc(r.equipment)}</div>
        <div class=\"target-meta\">${esc(r.note)}${r.moon_separation_deg == null ? '' : ` · Moon ${n(r.moon_separation_deg,0,'°')} away`}</div>
      </div>
      <div class=\"target-score\">${Math.round(r.score)}</div>
    </div>`).join('') : '<div class=\"muted\">No worthwhile targets cleared the current observing limits.</div>';
"""
    new_render = """  function targetImageKey(r) {
    const name = String(r.name || '').trim();
    const category = String(r.category || '').toLowerCase();
    const messier = name.match(/(?:^|[^A-Za-z])M\\s*0*(\\d{1,3})(?=\\b|\\s|\\()/i);
    if (messier) {
      const number = Number(messier[1]);
      if (number >= 1 && number <= 110) return `m${String(number).padStart(3, '0')}`;
    }
    const lower = name.toLowerCase();
    const planets = new Set(['mercury','venus','mars','jupiter','saturn','uranus','neptune']);
    if (category.includes('planet') && planets.has(lower)) return `planet-${lower}`;
    if (lower === 'moon') return 'moon';
    if (lower === 'milky way' || lower === 'galactic centre' || lower === 'galactic center') return 'milky-way';
    const meteors = {
      'perseids':'meteor-perseids', 'geminids':'meteor-geminids', 'quadrantids':'meteor-quadrantids',
      'lyrids':'meteor-lyrids', 'eta aquariids':'meteor-eta-aquariids', 'delta aquariids':'meteor-delta-aquariids',
      'orionids':'meteor-orionids', 'leonids':'meteor-leonids', 'taurids':'meteor-taurids', 'ursids':'meteor-ursids'
    };
    if (category.includes('meteor') && meteors[lower]) return meteors[lower];
    return null;
  }

  function targetThumbnail(r) {
    const key = targetImageKey(r);
    const image = key ? `<img class=\"target-thumb\" src=\"object-images/${key}.webp\" alt=\"${esc(r.name)}\" loading=\"lazy\" decoding=\"async\" onerror=\"this.remove()\">` : '';
    const credit = key ? `<a class=\"target-credit\" href=\"object-images/credits.html#${key}\" title=\"Image credit and licence\" aria-label=\"Image credit and licence for ${esc(r.name)}\">i</a>` : '';
    return `<div class=\"target-thumb-wrap\"><span class=\"target-thumb-fallback\" aria-hidden=\"true\">✦</span>${image}${credit}</div>`;
  }

  $('target-list').innerHTML = snapshot.recommendations.length ? snapshot.recommendations.map((r, i) => `
    <div class=\"target\">
      <div class=\"rank\">${i + 1}</div>
      ${targetThumbnail(r)}
      <div>
        <div class=\"target-name\">${esc(r.name)}</div>
        <div class=\"target-meta\">${esc(r.category)} · ${localTime(r.best_time)} · alt ${n(r.altitude_deg,0,'°')} · az ${n(r.azimuth_deg,0,'°')} · ${esc(r.equipment)}</div>
        <div class=\"target-meta\">${esc(r.note)}${r.moon_separation_deg == null ? '' : ` · Moon ${n(r.moon_separation_deg,0,'°')} away`}</div>
      </div>
      <div class=\"target-score\">${Math.round(r.score)}</div>
    </div>`).join('') : '<div class=\"muted\">No worthwhile targets cleared the current observing limits.</div>';
"""
    text = replace_once(text, old_render, new_render, "target rendering")
    write(path, text)


def asset_helpers(object_dir_expr: str) -> str:
    return f"""
fn object_asset_response(name: &str) -> Option<Response<std::io::Cursor<Vec<u8>>>> {{
    if name.is_empty()
        || name.contains("..")
        || !name.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'))
    {{
        return None;
    }}
    let content_type = if name.ends_with(".webp") {{
        "image/webp"
    }} else if name.ends_with(".html") {{
        "text/html; charset=utf-8"
    }} else if name.ends_with(".json") {{
        "application/json; charset=utf-8"
    }} else if name.ends_with(".txt") {{
        "text/plain; charset=utf-8"
    }} else {{
        return None;
    }};
    let data = fs::read({object_dir_expr}.join(name)).ok()?;
    Some(
        Response::from_data(data)
            .with_header(header("Content-Type", content_type))
            .with_header(header("Cache-Control", "public, max-age=604800"))
            .with_header(header("X-Content-Type-Options", "nosniff"))
            .with_header(header("Referrer-Policy", "no-referrer")),
    )
}}

"""


def patch_ha_server() -> None:
    path = "astronomy_observer/src/web.rs"
    text = read(path)
    text = replace_once(
        text,
        'const DASHBOARD: &str = include_str!("../dashboard/astronomy-dashboard.yaml");\n',
        'const DASHBOARD: &str = include_str!("../dashboard/astronomy-dashboard.yaml");\nconst OBJECT_IMAGE_DIR: &str = "/usr/share/astronomy-observer/object-images";\n',
        "HA object image directory",
    )
    helper = asset_helpers("Path::new(OBJECT_IMAGE_DIR)")
    text = insert_before(text, "fn validate_observation", helper, "HA static asset helper")
    route_marker = """                (Method::Get, \"/\") | (Method::Get, \"/index.html\") => {
                    let _ = request.respond(html_response(INDEX));
                }
"""
    route_new = route_marker + """                (Method::Get, value) if value.starts_with(\"/object-images/\") => {
                    let name = value.trim_start_matches(\"/object-images/\");
                    match object_asset_response(name) {
                        Some(response) => { let _ = request.respond(response); }
                        None => { let _ = request.respond(Response::from_string(\"not found\").with_status_code(StatusCode(404))); }
                    }
                }
"""
    text = replace_once(text, route_marker, route_new, "HA object image route")
    write(path, text)


def patch_web_server() -> None:
    path = "webapp/src/server.rs"
    text = read(path)
    helper = asset_helpers("Path::new(\"/usr/share/astronomy-observer/object-images\")")
    text = insert_before(text, "fn read_body", helper, "web static asset helper")
    route_marker = """        (Method::Get, \"/\") | (Method::Get, \"/index.html\") => {
            let _ = request.respond(html_response(index.to_string()));
        }
"""
    route_new = route_marker + """        (Method::Get, value) if value.starts_with(\"/object-images/\") => {
            let name = value.trim_start_matches(\"/object-images/\");
            match object_asset_response(name) {
                Some(response) => { let _ = request.respond(response); }
                None => { let _ = request.respond(Response::from_string(\"not found\").with_status_code(StatusCode(404))); }
            }
        }
"""
    text = replace_once(text, route_marker, route_new, "web object image route")
    write(path, text)


def patch_dockerfiles() -> None:
    path = "astronomy_observer/Dockerfile"
    text = read(path)
    text = replace_once(text, "ARG BUILD_VERSION=0.3.0", "ARG BUILD_VERSION=0.3.1", "HA version")
    marker = "COPY data/WORLD_ATLAS_NOTICE.md /usr/share/astronomy-observer/WORLD_ATLAS_NOTICE.md\n"
    text = replace_once(text, marker, marker + "COPY data/object_images /usr/share/astronomy-observer/object-images\n", "HA object image copy")
    write(path, text)

    path = "webapp/Dockerfile"
    text = read(path)
    marker = "COPY astronomy_observer/data/WORLD_ATLAS_NOTICE.md /usr/share/astronomy-observer/WORLD_ATLAS_NOTICE.md\n"
    text = replace_once(text, marker, marker + "COPY astronomy_observer/data/object_images /usr/share/astronomy-observer/object-images\n", "web object image copy")
    write(path, text)


def patch_android() -> None:
    path = "android/prepare_assets.py"
    text = read(path)
    text = replace_once(text, "from pathlib import Path\n", "from pathlib import Path\nimport shutil\n", "Android shutil import")
    text = replace_once(
        text,
        'SCRIPT = ANDROID / "ui" / "android.js"\n',
        'SCRIPT = ANDROID / "ui" / "android.js"\nOBJECT_IMAGES = ROOT / "astronomy_observer" / "data" / "object_images"\n',
        "Android object image source",
    )
    marker = '    (OUT / "project-license.txt").write_text((ROOT / "LICENSE").read_text(encoding="utf-8"), encoding="utf-8")\n'
    addition = marker + '    image_out = OUT / "object-images"\n    if image_out.exists():\n        shutil.rmtree(image_out)\n    shutil.copytree(OBJECT_IMAGES, image_out)\n'
    text = replace_once(text, marker, addition, "Android object image assets")
    write(path, text)

    path = "android/app/build.gradle.kts"
    text = read(path).replace("versionCode = 30000", "versionCode = 30100").replace('versionName = "0.3.0"', 'versionName = "0.3.1"')
    write(path, text)

    path = "android/validate.py"
    text = read(path).replace('versionName = "0.3.0"', 'versionName = "0.3.1"').replace('version must match 0.3.0 release', 'version must match 0.3.1 release').replace('version: "0.3.0"', 'version: "0.3.1"').replace('ARG BUILD_VERSION=0.3.0', 'ARG BUILD_VERSION=0.3.1')
    marker = '        ROOT / "docs/ANDROID.md",\n'
    text = replace_once(text, marker, marker + '        HA / "data/object_images/manifest.json",\n        HA / "data/object_images/m031.webp",\n        HA / "data/object_images/credits.html",\n', "Android required image assets")
    generated_marker = '        require("Copy dashboard YAML" in html, "shared dashboard marker unexpectedly disappeared")\n'
    text = replace_once(text, generated_marker, generated_marker + '        require("object-images/" in html, "generated Android UI lacks bundled target thumbnails")\n        require((ANDROID / "generated/assets/object-images/m031.webp").is_file(), "generated Android assets lack M31 thumbnail")\n', "Android generated thumbnail validation")
    write(path, text)

    path = ".github/workflows/android.yaml"
    text = read(path).replace("astronomy-observer-0.3.0-debug.apk", "astronomy-observer-0.3.1-debug.apk").replace("astronomy-observer-0.3.0-android-debug", "astronomy-observer-0.3.1-android-debug").replace("versionName='0.3.0'", "versionName='0.3.1'")
    marker = "          unzip -l \"$APK\" | grep -q 'assets/astronomy-engine-license.txt'\n"
    text = replace_once(text, marker, marker + "          unzip -l \"$APK\" | grep -q 'assets/object-images/m031.webp'\n          unzip -l \"$APK\" | grep -q 'assets/object-images/credits.html'\n", "Android APK image inspection")
    write(path, text)


def patch_config_and_ci() -> None:
    path = "astronomy_observer/config.yaml"
    text = read(path).replace('version: "0.3.0"', 'version: "0.3.1"')
    write(path, text)

    path = ".github/workflows/ci.yaml"
    text = read(path)
    marker = "      - name: Validate bundled light-pollution atlas\n        run: python3 tests/validate_atlas.py\n"
    text = replace_once(text, marker, marker + "\n      - name: Validate licensed object thumbnails\n        run: python3 tests/validate_object_images.py\n", "CI object image validator")
    old_compile = "python3 -m py_compile astronomy_observer/scripts/build_catalog.py astronomy_observer/scripts/build_global_atlas.py tools/light_pollution_tile.py tests/validate_repository.py tests/validate_atlas.py webapp/validate.py android/validate.py android/prepare_assets.py"
    new_compile = old_compile.replace("tools/light_pollution_tile.py", "astronomy_observer/scripts/build_object_thumbnails.py astronomy_observer/scripts/apply_object_thumbnail_feature.py tools/light_pollution_tile.py").replace("tests/validate_atlas.py", "tests/validate_atlas.py tests/validate_object_images.py")
    text = replace_once(text, old_compile, new_compile, "CI Python compilation")
    smoke = "          curl --fail --silent http://127.0.0.1:18080/ | grep -q 'Astronomy Observer'\n"
    text = replace_once(text, smoke, smoke + "          curl --fail --silent --output /tmp/m31.webp http://127.0.0.1:18080/object-images/m031.webp\n          test -s /tmp/m31.webp\n          curl --fail --silent http://127.0.0.1:18080/object-images/credits.html | grep -q 'Object image credits'\n", "web image smoke test")
    write(path, text)


def patch_web_validation() -> None:
    path = "webapp/validate.py"
    text = read(path)
    marker = '        ROOT / "railway.toml",\n'
    if marker in text:
        text = replace_once(text, marker, marker + '        ROOT / "astronomy_observer/data/object_images/manifest.json",\n        ROOT / "astronomy_observer/data/object_images/m031.webp",\n', "web validator image assets")
    write(path, text)


def patch_docs() -> None:
    path = "README.md"
    text = read(path).replace('`0.3.0` remains experimental.', '`0.3.1` remains experimental.')
    highlight = "- **Targets are ranked for your sky and your equipment.** Altitude, local horizon, Moon separation, sky brightness, object type and configured aperture all matter.\n"
    text = replace_once(text, highlight, highlight + "- **Recognisable targets now look recognisable.** Small offline thumbnails are bundled for Messier objects plus familiar planets and major meteor showers; if a target has no licensed image, the list simply falls back to the normal astronomy marker.\n", "README thumbnail highlight")
    feature = "- category-aware Top 10 selection so satellites or comets cannot crowd out the normal observing programme;\n"
    text = replace_once(text, feature, feature + "- compact target thumbnails for licensed Wikimedia Commons images, with in-app per-image credit/licence links and no remote image dependency;\n", "README feature list")
    write(path, text)

    path = "THIRD_PARTY_LICENSES.md"
    text = read(path)
    section = """
## Wikimedia Commons object thumbnails

Astronomy Observer bundles small target thumbnails for the Messier catalogue and a limited set of familiar planets, the Moon, the Milky Way and major meteor showers. The images are selected through the English Wikipedia PageImages API using its `free`-image filter and are then independently checked through the Wikimedia Commons Imageinfo `extmetadata` API before they are accepted.

The builder accepts only files reported as **Public domain, CC0, CC BY, or CC BY-SA**. Fair-use/non-free, NonCommercial, NoDerivatives, GFDL-only and unknown licences are rejected. Each accepted image keeps its own per-file licence and attribution; the image files are not relicensed under Astronomy Observer's PolyForm licence.

The bundled copy is a reduced WebP thumbnail. `astronomy_observer/data/object_images/manifest.json` records the original Commons file, creator, licence, source URL and the thumbnail transformation. `object-images/credits.html` is shipped with Home Assistant, Docker/web and Android so those credits remain accessible from the target image itself.

Wikimedia Commons: https://commons.wikimedia.org/

Wikimedia developer guidance for image licensing: https://foundation.wikimedia.org/wiki/Legal:Wikimedia_Developer_App_Guidelines

"""
    text = insert_before(text, "## Open-Meteo", section, "Wikimedia thumbnail licence section")
    write(path, text)

    path = "docs/DATA_SOURCES.md"
    text = read(path)
    section = """
## Target thumbnails

Small target pictures are an offline presentation aid, not an astronomy-data input. A build-time script asks Wikipedia for the representative free image for each Messier object and a short list of familiar Solar-System/meteor targets, then verifies the actual file and its per-file licence through Wikimedia Commons before including it. The runtime never scrapes Wikipedia and does not contact Wikimedia to draw the target list.

Only Public domain, CC0, CC BY and CC BY-SA files are accepted. Per-image creator/source/licence metadata travels with every installation in `object-images/credits.html` and `manifest.json`. Targets without a verified bundled image keep the same ranking and simply show the neutral fallback marker.

"""
    text = insert_before(text, "##", section, "data source thumbnail section") if "Target thumbnails" not in text else text
    write(path, text)

    path = "docs/ANDROID.md"
    text = read(path)
    marker = "The APK contains"
    if marker in text and "object thumbnails" not in text.lower():
        text = text.replace(marker, "The APK contains the licensed object thumbnails and their per-image credits alongside the other local assets. " + marker, 1)
    write(path, text)

    for path in ["CHANGELOG.md", "astronomy_observer/CHANGELOG.md"]:
        text = read(path)
        entry = """## 0.3.1 - 2026-08-17

- Added small offline target thumbnails for Messier objects plus familiar planets, the Moon, the Milky Way and major meteor showers.
- Images are selected at build time from Wikimedia's representative free images and then licence-checked against Wikimedia Commons metadata; only Public domain, CC0, CC BY and CC BY-SA files are accepted.
- Added per-image creator/source/licence credits inside every installation; third-party images remain under their original licences rather than the project PolyForm licence.
- Targets without a verified bundled image keep the existing ranking/list behaviour and show a neutral fallback marker, so image coverage never changes observing recommendations.
- Kept the thumbnails fully local in Home Assistant, Docker/web and Android; rendering the target list does not depend on Wikipedia, Wikimedia Commons or an Astronomy Observer server.

"""
        text = insert_before(text, "## 0.3.0", entry, f"{path} 0.3.1 changelog")
        write(path, text)


def main() -> None:
    patch_index()
    patch_ha_server()
    patch_web_server()
    patch_dockerfiles()
    patch_android()
    patch_config_and_ci()
    patch_web_validation()
    patch_docs()
    print("Object thumbnail feature patch applied")


if __name__ == "__main__":
    main()
