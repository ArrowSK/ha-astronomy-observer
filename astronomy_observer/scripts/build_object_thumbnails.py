#!/usr/bin/env python3
"""Build a small, redistributable thumbnail set for common observing targets.

The builder intentionally uses Wikimedia APIs rather than scraping pages. It asks
English Wikipedia for representative *free* page images in batches, then verifies
that the actual files exist on Wikimedia Commons and that each per-file licence is
in a deliberately small allow-list before downloading anything.

The output is a compact set of WebP thumbnails plus machine- and human-readable
attribution. Third-party images keep their own licences; they are never relicensed
under Astronomy Observer's project licence.
"""

from __future__ import annotations

import html
import json
import re
import time
import urllib.error
import urllib.parse
import urllib.request
from html.parser import HTMLParser
from pathlib import Path
from typing import Any, Iterable, TypeVar

from PIL import Image

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "astronomy_observer" / "data" / "object_images"
USER_AGENT = "AstronomyObserverThumbnailBuilder/1.0 (+https://github.com/ArrowSK/ha-astronomy-observer)"
WIKIPEDIA_API = "https://en.wikipedia.org/w/api.php"
COMMONS_API = "https://commons.wikimedia.org/w/api.php"
THUMB_MAX = 192
API_BATCH = 40
T = TypeVar("T")

# Messier coverage is the useful baseline because those identifiers already appear
# in OpenNGC-derived target names. A small set of familiar Solar-System/meteor
# targets gives the UI useful photos outside the Messier catalogue without trying
# to bundle thousands of NGC/IC images.
TARGETS: list[tuple[str, str]] = [
    *((f"m{number:03d}", f"Messier {number}") for number in range(1, 111)),
    ("planet-mercury", "Mercury (planet)"),
    ("planet-venus", "Venus"),
    ("planet-mars", "Mars"),
    ("planet-jupiter", "Jupiter"),
    ("planet-saturn", "Saturn"),
    ("planet-uranus", "Uranus"),
    ("planet-neptune", "Neptune"),
    ("moon", "Moon"),
    ("milky-way", "Milky Way"),
    ("meteor-perseids", "Perseids"),
    ("meteor-geminids", "Geminids"),
    ("meteor-quadrantids", "Quadrantids"),
    ("meteor-lyrids", "Lyrids"),
    ("meteor-eta-aquariids", "Eta Aquariids"),
    ("meteor-delta-aquariids", "Delta Aquariids"),
    ("meteor-orionids", "Orionids"),
    ("meteor-leonids", "Leonids"),
    ("meteor-taurids", "Taurids"),
    ("meteor-ursids", "Ursids"),
]


class _TextExtractor(HTMLParser):
    def __init__(self) -> None:
        super().__init__()
        self.parts: list[str] = []

    def handle_data(self, data: str) -> None:
        self.parts.append(data)


def clean_html(value: str | None) -> str:
    if not value:
        return ""
    parser = _TextExtractor()
    parser.feed(html.unescape(value))
    return re.sub(r"\s+", " ", " ".join(parser.parts)).strip()


def chunks(values: list[T], size: int = API_BATCH) -> Iterable[list[T]]:
    for offset in range(0, len(values), size):
        yield values[offset : offset + size]


def read_url(request: urllib.request.Request, timeout: int) -> bytes:
    delay = 2.0
    for attempt in range(6):
        try:
            with urllib.request.urlopen(request, timeout=timeout) as response:
                return response.read()
        except urllib.error.HTTPError as error:
            if error.code not in {429, 500, 502, 503, 504} or attempt == 5:
                raise
            retry_after = error.headers.get("Retry-After", "").strip()
            wait = float(retry_after) if retry_after.isdigit() else delay
            print(f"Wikimedia returned HTTP {error.code}; retrying in {wait:.0f}s")
            time.sleep(wait)
            delay = min(delay * 2, 30.0)
        except urllib.error.URLError:
            if attempt == 5:
                raise
            time.sleep(delay)
            delay = min(delay * 2, 30.0)
    raise RuntimeError("unreachable retry loop")


def request_json(url: str, params: dict[str, str]) -> dict[str, Any]:
    query = urllib.parse.urlencode(params)
    request = urllib.request.Request(f"{url}?{query}", headers={"User-Agent": USER_AGENT})
    return json.loads(read_url(request, 35).decode("utf-8"))


def resolve_title(title: str, aliases: dict[str, str]) -> str:
    value = title
    seen: set[str] = set()
    while value in aliases and value not in seen:
        seen.add(value)
        value = aliases[value]
    return value


def page_images() -> dict[str, tuple[str, str | None]]:
    result: dict[str, tuple[str, str | None]] = {}
    for batch in chunks(TARGETS):
        requested_titles = [page_title for _, page_title in batch]
        payload = request_json(
            WIKIPEDIA_API,
            {
                "action": "query",
                "format": "json",
                "formatversion": "2",
                "redirects": "1",
                "prop": "pageimages",
                "piprop": "name",
                "pilicense": "free",
                "titles": "|".join(requested_titles),
            },
        )
        query = payload.get("query", {})
        aliases: dict[str, str] = {}
        for pair in query.get("normalized", []):
            aliases[pair.get("from", "")] = pair.get("to", "")
        for pair in query.get("redirects", []):
            aliases[pair.get("from", "")] = pair.get("to", "")
        pages = {
            page.get("title", ""): page
            for page in query.get("pages", [])
            if isinstance(page, dict)
        }
        for key, page_title in batch:
            resolved = resolve_title(page_title, aliases)
            result[key] = (page_title, pages.get(resolved, {}).get("pageimage"))
        time.sleep(0.8)
    return result


def filename_key(value: str) -> str:
    return value.replace("_", " ").strip().casefold()


def commons_infos(filenames: list[str]) -> dict[str, dict[str, Any]]:
    result: dict[str, dict[str, Any]] = {}
    unique = list(dict.fromkeys(filename for filename in filenames if filename))
    for batch in chunks(unique):
        payload = request_json(
            COMMONS_API,
            {
                "action": "query",
                "format": "json",
                "formatversion": "2",
                "prop": "imageinfo",
                "iiprop": "url|extmetadata",
                "iiurlwidth": str(THUMB_MAX),
                "titles": "|".join(f"File:{filename}" for filename in batch),
            },
        )
        for page in payload.get("query", {}).get("pages", []):
            if not isinstance(page, dict) or page.get("missing"):
                continue
            info = page.get("imageinfo", [])
            if not info:
                continue
            page_title = page.get("title", "")
            if page_title.startswith("File:"):
                page_title = page_title[5:]
            result[filename_key(page_title)] = info[0]
        time.sleep(0.8)
    return result


def metadata_value(metadata: dict[str, Any], key: str) -> str:
    value = metadata.get(key, {})
    return clean_html(value.get("value") if isinstance(value, dict) else "")


def allowed_license(short_name: str) -> bool:
    normal = short_name.strip().lower()
    return (
        normal == "public domain"
        or normal.startswith("cc0")
        or normal.startswith("cc by ")
        or normal.startswith("cc by-sa ")
    )


def commons_page(filename: str) -> str:
    title = "File:" + filename.replace(" ", "_")
    return "https://commons.wikimedia.org/wiki/" + urllib.parse.quote(title, safe=":()_-.,")


def download(url: str, path: Path) -> None:
    request = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    path.write_bytes(read_url(request, 45))


def build_thumbnail(source: Path, target: Path) -> tuple[int, int]:
    with Image.open(source) as image:
        image.thumbnail((THUMB_MAX, THUMB_MAX), Image.Resampling.LANCZOS)
        if image.mode not in ("RGB", "RGBA"):
            image = image.convert("RGBA" if "transparency" in image.info else "RGB")
        image.save(target, "WEBP", quality=82, method=6)
        return image.width, image.height


def credits_html(items: list[dict[str, Any]]) -> str:
    rows = []
    for item in items:
        creator = html.escape(item["creator"] or "See source file page")
        source = html.escape(item["source_url"], quote=True)
        label = html.escape(item["label"])
        licence_name = html.escape(item["license"])
        licence_url = html.escape(item["license_url"], quote=True)
        licence = (
            f'<a href="{licence_url}" rel="external noreferrer">{licence_name}</a>'
            if licence_url
            else licence_name
        )
        rows.append(
            f'<article id="{item["key"]}"><h2>{label}</h2>'
            f'<p>{creator} · <a href="{source}" rel="external noreferrer">Wikimedia Commons source</a> · {licence}</p>'
            '<p class="muted">Astronomy Observer distributes a reduced WebP thumbnail; the image remains under the licence shown above.</p></article>'
        )
    return """<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Object image credits · Astronomy Observer</title><base target="_blank"><style>
:root{color-scheme:dark}body{margin:0;background:#090d16;color:#eef3ff;font:15px/1.55 system-ui,-apple-system,BlinkMacSystemFont,"Segoe UI",sans-serif}main{max-width:820px;margin:auto;padding:24px 18px 60px}h1{font-size:24px}h2{font-size:16px;margin:0 0 4px}article{padding:14px 0;border-top:1px solid #28344a}a{color:#9ec5ff}.muted{color:#9eabc1;font-size:13px}.return-bar{position:fixed;z-index:20;top:0;left:0;right:0;display:flex;align-items:center;gap:14px;min-height:64px;padding:calc(10px + env(safe-area-inset-top)) 18px 10px;background:rgba(9,13,22,.96);border-bottom:1px solid #28344a;backdrop-filter:blur(12px)}.return-bar a{display:inline-flex;align-items:center;justify-content:center;width:44px;height:44px;padding:0;border:1px solid #28344a;border-radius:9px;background:#172033;color:#eef3ff;text-decoration:none;font-size:22px;font-weight:650}.return-bar a:hover{border-color:#4d607f;background:#1a2840}.return-bar span{color:#9eabc1;font-size:13px}main{padding-top:calc(96px + env(safe-area-inset-top))}article{scroll-margin-top:calc(92px + env(safe-area-inset-top))}@media(max-width:520px){.return-bar span{display:none}}</style></head>
<body><nav class="return-bar" aria-label="Credit page navigation"><a id="credits-back" target="_self" href="../#targets" aria-label="Back to Astronomy Observer" title="Back to Astronomy Observer" onclick="if(location.protocol==='file:'){this.href='../index.html#targets';}">←</a><span>Image credits and licences</span></nav><main><h1>Object image credits</h1><p>Small target images in Astronomy Observer come from Wikimedia Commons. Each file was selected through Wikimedia APIs only after its per-file licence was checked. The images below keep their original licences and are not relicensed under the Astronomy Observer project licence.</p>
""" + "\n".join(rows) + "\n</main></body></html>\n"


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    temporary = OUT / ".download"
    temporary.mkdir(exist_ok=True)
    items: list[dict[str, Any]] = []
    skipped: list[dict[str, str]] = []

    page_data = page_images()
    image_info = commons_infos([filename for _, filename in page_data.values() if filename])

    for index, (key, page_title) in enumerate(TARGETS, start=1):
        filename = page_data.get(key, (page_title, None))[1]
        if not filename:
            skipped.append({"key": key, "page": page_title, "reason": "no free representative page image"})
            print(f"[{index:03d}/{len(TARGETS)}] {key}: no free representative page image")
            continue
        info = image_info.get(filename_key(filename))
        if not info:
            skipped.append({"key": key, "page": page_title, "reason": "representative image is not available from Wikimedia Commons imageinfo"})
            print(f"[{index:03d}/{len(TARGETS)}] {key}: Commons metadata unavailable")
            continue

        metadata = info.get("extmetadata", {})
        licence = metadata_value(metadata, "LicenseShortName")
        if not allowed_license(licence):
            skipped.append({"key": key, "page": page_title, "reason": f"licence not allow-listed: {licence or 'unknown'}"})
            print(f"[{index:03d}/{len(TARGETS)}] {key}: rejected licence {licence or 'unknown'}")
            continue
        thumb_url = info.get("thumburl") or info.get("url")
        if not thumb_url:
            skipped.append({"key": key, "page": page_title, "reason": "no downloadable image URL"})
            continue

        try:
            raw = temporary / f"{key}.source"
            local = OUT / f"{key}.webp"
            download(thumb_url, raw)
            width, height = build_thumbnail(raw, local)
            raw.unlink(missing_ok=True)

            creator = metadata_value(metadata, "Artist") or metadata_value(metadata, "Credit")
            item = {
                "key": key,
                "label": page_title,
                "local_path": f"{key}.webp",
                "width": width,
                "height": height,
                "source_file": filename,
                "source_url": commons_page(filename),
                "creator": creator,
                "license": licence,
                "license_url": metadata_value(metadata, "LicenseUrl"),
                "selection": "English Wikipedia representative page image restricted to free images, then verified on Wikimedia Commons",
                "transformation": f"scaled/re-encoded WebP thumbnail, maximum {THUMB_MAX}px per side",
            }
            items.append(item)
            print(f"[{index:03d}/{len(TARGETS)}] {key}: {filename} ({licence})")
        except Exception as error:
            skipped.append({"key": key, "page": page_title, "reason": f"download/thumbnail error: {error}"})
            print(f"[{index:03d}/{len(TARGETS)}] {key}: skipped: {error}")
        time.sleep(0.18)

    try:
        temporary.rmdir()
    except OSError:
        pass

    manifest = {
        "format": "Astronomy Observer object thumbnails v1",
        "source": "Wikimedia Commons via English Wikipedia PageImages and Wikimedia Commons Imageinfo APIs",
        "policy": "Only Public domain, CC0, CC BY, and CC BY-SA images are accepted; fair-use/non-free/NC/ND/GFDL-only/unknown files are excluded.",
        "thumbnail_max_px": THUMB_MAX,
        "items": items,
        "skipped": skipped,
    }
    (OUT / "manifest.json").write_text(json.dumps(manifest, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    (OUT / "credits.html").write_text(credits_html(items), encoding="utf-8")
    (OUT / "NOTICE.txt").write_text(
        "Astronomy Observer object thumbnails\n\n"
        "These small target images are redistributed from Wikimedia Commons. Each image keeps its own per-file licence and attribution; none is relicensed under the Astronomy Observer PolyForm licence.\n\n"
        "The build accepts only files whose Wikimedia Commons metadata reports Public domain, CC0, CC BY, or CC BY-SA. Fair-use/non-free, NC, ND, GFDL-only, and unknown licences are excluded.\n\n"
        "See object-images/credits.html in the application or astronomy_observer/data/object_images/manifest.json in the source tree for per-image creator, source and licence details.\n",
        encoding="utf-8",
    )
    print(f"Wrote {len(items)} licensed thumbnails; skipped {len(skipped)} targets")
    if len(items) < 80:
        raise SystemExit("Too few licensed thumbnails were produced; refusing to publish an unexpectedly sparse set")
    for required in ("m031.webp", "planet-saturn.webp"):
        if not (OUT / required).is_file():
            raise SystemExit(f"Required packaging smoke-test asset is missing: {required}")


if __name__ == "__main__":
    main()
