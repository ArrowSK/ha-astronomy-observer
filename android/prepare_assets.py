#!/usr/bin/env python3
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ANDROID = ROOT / "android"
BASE = ROOT / "astronomy_observer" / "web" / "index.html"
OUT = ANDROID / "generated" / "assets"
SCRIPT = ANDROID / "ui" / "android.js"


def replace_once(text: str, old: str, new: str, label: str) -> str:
    if old not in text:
        raise SystemExit(f"Android UI adapter marker changed: {label}")
    return text.replace(old, new, 1)


def main() -> None:
    html = BASE.read_text(encoding="utf-8")
    android_js = SCRIPT.read_text(encoding="utf-8").rstrip()

    html = replace_once(
        html,
        '<meta name="color-scheme" content="dark light">',
        '<meta name="color-scheme" content="dark light">\n'
        '  <meta http-equiv="Content-Security-Policy" content="default-src \'self\' data:; script-src \'self\' \'unsafe-inline\'; style-src \'self\' \'unsafe-inline\'; img-src \'self\' data:; connect-src \'none\'; object-src \'none\'; frame-src \'none\'; base-uri \'none\'">',
        "head CSP",
    )

    old_location = '''      <div class="setup-box">
        <h3>Location</h3>
        <label>Observe for
          <select id="person-select"><option value="">Home</option></select>
        </label>
        <p class="foot">People come directly from Home Assistant. If a selected person has no current coordinates, Home is used as the fallback.</p>
      </div>'''
    new_location = '''      <div class="setup-box">
        <h3>Location</h3>
        <select id="person-select" hidden><option value="">Android location</option></select>
        <div class="setup-actions" style="margin-top:0"><button id="android-current-location" type="button">Use current location</button><span id="android-location-status" class="muted"></span></div>
        <div class="android-location-grid">
          <label>Site name<input id="web-location-label" maxlength="100" placeholder="Observing site"></label>
          <label>Time zone<input id="web-location-timezone" spellcheck="false" placeholder="Europe/Budapest"></label>
          <label>Latitude<input id="web-location-lat" inputmode="decimal" type="number" min="-90" max="90" step="0.000001" placeholder="47.497900"></label>
          <label>Longitude<input id="web-location-lon" inputmode="decimal" type="number" min="-180" max="180" step="0.000001" placeholder="19.040200"></label>
          <label>Elevation (m)<input id="web-location-elevation" inputmode="decimal" type="number" min="-500" max="9000" step="1" value="0"></label>
        </div>
        <p class="foot">The exact coordinates stay on this phone for local astronomy and light-pollution calculations. Weather requests use the same reduced coordinate precision as the other Astronomy Observer editions.</p>
      </div>'''
    html = replace_once(html, old_location, new_location, "location box")

    html = replace_once(
        html,
        "    .setup-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px; }",
        "    .setup-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px; }\n"
        "    .android-location-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 8px; margin-top: 10px; }",
        "location CSS",
    )
    html = replace_once(
        html,
        "      .condition-groups, .note-form, .setup-grid, .history-controls { grid-template-columns: 1fr; }",
        "      .condition-groups, .note-form, .setup-grid, .history-controls, .android-location-grid { grid-template-columns: 1fr; }",
        "mobile location CSS",
    )
    html = replace_once(
        html,
        "<span><strong>Setup</strong><small>Observer, horizon and dashboard</small></span>",
        "<span><strong>Setup</strong><small>Location and horizon</small></span>",
        "setup menu subtitle",
    )

    notes_tail = '''            <span><strong>Observation journal</strong><small>Notes, history, search and filters</small></span>
          </button>
        </div>'''
    notes_with_about = '''            <span><strong>Observation journal</strong><small>Notes, history, search and filters</small></span>
          </button>
          <button id="about-button" class="menu-item" type="button" role="menuitem">
            <svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="9"/><path d="M12 11v6M12 7h.01"/></svg>
            <span><strong>About &amp; licences</strong><small>Data sources, attribution and licences</small></span>
          </button>
        </div>'''
    html = replace_once(html, notes_tail, notes_with_about, "about menu")

    html = replace_once(
        html,
        '      <div id="source-list"></div>',
        '      <div id="source-list"></div>\n'
        '      <div class="foot"><a href="https://open-meteo.com/" rel="external noreferrer">Weather data by Open-Meteo.com</a> · transformed into Astronomy Observer condition scores.</div>',
        "Open-Meteo attribution",
    )

    dashboard_marker = '<div class="setup-box" style="grid-column:1/-1">\n        <h3>Dashboard preset</h3>'
    html = replace_once(
        html,
        dashboard_marker,
        '<div class="setup-box" hidden style="grid-column:1/-1">\n        <h3>Dashboard preset</h3>',
        "Home Assistant dashboard box",
    )

    startup = "load();\nupdateBottomNav();\nsetInterval(load, 60000);"
    html = replace_once(html, startup, android_js, "startup script")

    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "index.html").write_text(html, encoding="utf-8")
    (OUT / "project-license.txt").write_text((ROOT / "LICENSE").read_text(encoding="utf-8"), encoding="utf-8")
    print("Android WebView assets prepared from the shared interface")


if __name__ == "__main__":
    main()
