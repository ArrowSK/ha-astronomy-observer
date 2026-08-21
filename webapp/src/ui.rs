use crate::error::{err, AppResult};

const BASE_INDEX: &str = include_str!("../../astronomy_observer/web/index.html");
const WEB_SCRIPT: &str = include_str!("../ui/manual.js");

pub fn web_index() -> AppResult<String> {
    let old_location = r#"      <div class="setup-box">
        <h3>Location</h3>
        <label>Observe for
          <select id="person-select"><option value="">Home</option></select>
        </label>
        <p class="foot">People come directly from Home Assistant. If a selected person has no current coordinates, Home is used as the fallback.</p>
      </div>"#;
    let new_location = r#"      <div class="setup-box">
        <h3>Location</h3>
        <select id="person-select" hidden><option value="">Web location</option></select>
        <p class="muted">Enter the observing site used for this browser session. Reloading the page clears it.</p>
        <div class="web-location-grid">
          <label>Site name<input id="web-location-label" maxlength="100" placeholder="Observing site"></label>
          <label>Time zone<input id="web-location-timezone" spellcheck="false" value="UTC" placeholder="Europe/Budapest"></label>
          <label>Latitude<input id="web-location-lat" inputmode="decimal" type="number" min="-90" max="90" step="0.000001" placeholder="47.497900"></label>
          <label>Longitude<input id="web-location-lon" inputmode="decimal" type="number" min="-180" max="180" step="0.000001" placeholder="19.040200"></label>
          <label>Elevation (m)<input id="web-location-elevation" inputmode="decimal" type="number" min="-500" max="9000" step="1" value="0"></label>
        </div>
      </div>"#;

    if !BASE_INDEX.contains(old_location) {
        return Err(err(
            "shared interface location marker changed; update the web adapter",
        ));
    }
    let mut html = BASE_INDEX.replacen(old_location, new_location, 1);
    html = html.replacen(
        "    .setup-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px; }",
        "    .setup-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 12px; }\n    .web-location-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 8px; margin-top: 8px; }",
        1,
    );
    html = html.replacen(
        "      .condition-groups, .note-form, .setup-grid, .history-controls { grid-template-columns: 1fr; }",
        "      .condition-groups, .note-form, .setup-grid, .history-controls, .web-location-grid { grid-template-columns: 1fr; }",
        1,
    );
    html = html.replacen(
        "          <button id=\"notes-button\"",
        "          <button id=\"notes-button\" hidden",
        1,
    );
    html = html.replacen(
        "<div class=\"setup-box\" style=\"grid-column:1/-1\">\n        <h3>Dashboard preset</h3>",
        "<div class=\"setup-box\" hidden style=\"grid-column:1/-1\">\n        <h3>Dashboard preset</h3>",
        1,
    );
    html = html.replacen(
        "<span><strong>Setup</strong><small>Observer, horizon and dashboard</small></span>",
        "<span><strong>Setup</strong><small>Observer location and horizon</small></span>",
        1,
    );

    let old_tail = "load();\nupdateBottomNav();\nsetInterval(load, 60000);";
    if !html.contains(old_tail) {
        return Err(err(
            "shared interface startup marker changed; update the web adapter",
        ));
    }
    html = html.replacen(old_tail, WEB_SCRIPT, 1);

    let body_end = "</body>";
    if !html.contains(body_end) {
        return Err(err(
            "shared interface body marker changed; update the target image adapter",
        ));
    }
    html = html.replacen(
        body_end,
        "<script src=\"target-images.js?v=0.3.4\"></script>\n</body>",
        1,
    );
    Ok(html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_interface_transforms_for_web() {
        let html = web_index().unwrap();
        assert!(html.contains("web-location-lat"));
        assert!(html.contains("api/web/snapshot"));
        assert!(html.contains("<span>Forecast</span>"));
        assert!(html.contains("target-images.js?v=0.3.4"));
    }
}
