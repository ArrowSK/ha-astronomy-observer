const ANDROID_SETTINGS_KEY = 'astronomy-observer.android.settings.v1';
const ANDROID_OBSERVATIONS_KEY = 'astronomy-observer.android.observations.v1';
let androidLocation = null;
let androidMinimumAltitude = 20;
let androidHorizonMask = '0:0,90:0,180:0,270:0';
let androidCalculating = false;

function androidJsonResponse(value, status = 200) {
  return new Response(JSON.stringify(value), {
    status,
    headers: {'Content-Type': 'application/json; charset=utf-8'}
  });
}

function androidReadObservations() {
  try {
    const value = JSON.parse(localStorage.getItem(ANDROID_OBSERVATIONS_KEY) || '[]');
    return Array.isArray(value) ? value : [];
  } catch (_) {
    return [];
  }
}

function androidWriteObservations(rows) {
  localStorage.setItem(ANDROID_OBSERVATIONS_KEY, JSON.stringify(rows.slice(0, 500)));
}

window.fetch = async function(input, init = {}) {
  const raw = typeof input === 'string' ? input : String(input?.url || '');
  const path = raw.replace(/^\.\//, '').replace(/^\//, '').split('?')[0];
  const method = String(init.method || 'GET').toUpperCase();

  if (path === 'api/observations' && method === 'GET') {
    return androidJsonResponse(androidReadObservations());
  }
  if (path === 'api/observations' && method === 'DELETE') {
    let payload = {};
    try { payload = JSON.parse(init.body || '{}'); } catch (_) {}
    const remove = new Set(Array.isArray(payload.recorded_at) ? payload.recorded_at : []);
    const before = androidReadObservations();
    const after = before.filter(row => !remove.has(row.recorded_at));
    androidWriteObservations(after);
    return androidJsonResponse({deleted: before.length - after.length});
  }
  if (path === 'api/observation' && method === 'POST') {
    let payload;
    try { payload = JSON.parse(init.body || '{}'); }
    catch (_) { return androidJsonResponse({error: 'invalid observation'}, 400); }
    const now = new Date().toISOString();
    const row = {
      recorded_at: now,
      location: currentSnapshot?.location?.label || androidLocation?.label || 'Observing site',
      sqm: payload.sqm ?? null,
      seeing_arcsec: payload.seeing_arcsec ?? null,
      transparency: payload.transparency ?? null,
      limiting_magnitude: payload.limiting_magnitude ?? null,
      notes: String(payload.notes || '').slice(0, 1000),
      forecast_score: currentSnapshot?.conditions?.overall ?? null
    };
    const rows = androidReadObservations();
    rows.unshift(row);
    androidWriteObservations(rows);
    return androidJsonResponse({saved: true, recorded_at: now});
  }
  if (path === 'api/refresh' && method === 'POST') {
    return androidJsonResponse({accepted: true}, 202);
  }
  if (path === 'api/people' && method === 'GET') return androidJsonResponse([]);
  if (path === 'api/settings' && method === 'GET') {
    return androidJsonResponse({
      primary_person: '',
      minimum_target_altitude: androidMinimumAltitude,
      horizon_mask: androidHorizonMask
    });
  }
  if (path.startsWith('api/')) {
    return androidJsonResponse({error: 'This Home Assistant-only endpoint is not used by the Android app.'}, 404);
  }
  throw new Error('Network requests from the embedded interface are disabled.');
};

function androidLoadSavedSettings() {
  const timezone = Intl.DateTimeFormat().resolvedOptions().timeZone || 'UTC';
  $('web-location-timezone').value = timezone;
  try {
    const saved = JSON.parse(localStorage.getItem(ANDROID_SETTINGS_KEY) || 'null');
    if (!saved || !saved.location) return;
    androidLocation = saved.location;
    androidMinimumAltitude = Number(saved.minimum_target_altitude ?? 20);
    androidHorizonMask = saved.horizon_mask || '0:0,90:0,180:0,270:0';
    $('web-location-label').value = androidLocation.label || 'Observing site';
    $('web-location-timezone').value = androidLocation.timezone || timezone;
    $('web-location-lat').value = androidLocation.latitude;
    $('web-location-lon').value = androidLocation.longitude;
    $('web-location-elevation').value = androidLocation.elevation_m || 0;
  } catch (_) {
    localStorage.removeItem(ANDROID_SETTINGS_KEY);
  }
}

function androidLocationFromForm() {
  return {
    latitude: Number($('web-location-lat').value),
    longitude: Number($('web-location-lon').value),
    elevation_m: Number($('web-location-elevation').value || 0),
    label: $('web-location-label').value.trim() || 'Observing site',
    timezone: $('web-location-timezone').value.trim() || 'UTC'
  };
}

loadSetup = async function() {
  const altitude = String(Math.round(androidMinimumAltitude));
  if (![...$('minimum-altitude').options].some(option => option.value === altitude)) {
    $('minimum-altitude').insertAdjacentHTML('beforeend', `<option value="${esc(altitude)}">${esc(altitude)}° — current value</option>`);
  }
  $('minimum-altitude').value = altitude;
  $('horizon-mask').value = androidHorizonMask;
  $('setup-status').textContent = '';
  setupLoaded = true;
};

load = async function() {
  if (!androidLocation) {
    $('content').hidden = true;
    $('loading').hidden = false;
    $('loading').textContent = 'Choose the observing location in Setup. You can use the phone location or enter a site manually.';
    return;
  }
  if (androidCalculating) return;
  androidCalculating = true;
  $('loading').hidden = false;
  $('loading').textContent = 'Calculating tonight on this phone…';
  try {
    AstronomyAndroid.calculate(JSON.stringify({
      location: androidLocation,
      minimum_target_altitude: androidMinimumAltitude,
      horizon_mask: androidHorizonMask
    }));
  } catch (error) {
    androidCalculating = false;
    $('content').hidden = true;
    $('loading').textContent = `Unable to start the calculation: ${error.message}`;
  }
};

window.androidCalculationResult = function(text) {
  androidCalculating = false;
  try {
    const result = JSON.parse(text);
    if (!result.ok) throw new Error(result.error || 'Unknown native calculation error');
    render(result.snapshot);
  } catch (error) {
    $('content').hidden = true;
    $('loading').hidden = false;
    $('loading').textContent = `Unable to calculate the current result: ${error.message}`;
  }
};

window.androidLocationResult = function(result) {
  const status = $('android-location-status');
  if (!result?.ok) {
    status.textContent = result?.error || 'Location is unavailable.';
    return;
  }
  $('web-location-lat').value = Number(result.latitude).toFixed(6);
  $('web-location-lon').value = Number(result.longitude).toFixed(6);
  $('web-location-elevation').value = Number(result.elevation_m || 0).toFixed(0);
  $('web-location-label').value = result.label || 'Current location';
  $('web-location-timezone').value = result.timezone || $('web-location-timezone').value || 'UTC';
  status.textContent = 'Location ready. Save to calculate.';
};

document.addEventListener('click', async event => {
  const button = event.target.closest('#save-setup');
  if (!button) return;
  event.preventDefault();
  event.stopImmediatePropagation();

  const location = androidLocationFromForm();
  if (!Number.isFinite(location.latitude) || location.latitude < -90 || location.latitude > 90) {
    $('setup-status').textContent = 'Latitude must be between -90 and 90.';
    return;
  }
  if (!Number.isFinite(location.longitude) || location.longitude < -180 || location.longitude > 180) {
    $('setup-status').textContent = 'Longitude must be between -180 and 180.';
    return;
  }
  if (!Number.isFinite(location.elevation_m) || location.elevation_m < -500 || location.elevation_m > 9000) {
    $('setup-status').textContent = 'Elevation must be between -500 and 9000 metres.';
    return;
  }

  androidLocation = location;
  androidMinimumAltitude = Number($('minimum-altitude').value);
  androidHorizonMask = $('horizon-mask').value.trim() || '0:0,90:0,180:0,270:0';
  localStorage.setItem(ANDROID_SETTINGS_KEY, JSON.stringify({
    location: androidLocation,
    minimum_target_altitude: androidMinimumAltitude,
    horizon_mask: androidHorizonMask
  }));
  button.disabled = true;
  $('setup-status').textContent = 'Saved. Calculating…';
  $('setup-card').hidden = true;
  $('setup-button').setAttribute('aria-expanded', 'false');
  try {
    await load();
  } finally {
    button.disabled = false;
    $('setup-status').textContent = '';
  }
}, true);

$('android-current-location').addEventListener('click', () => {
  $('android-location-status').textContent = 'Getting location…';
  AstronomyAndroid.requestLocation();
});

$('about-button').addEventListener('click', () => {
  closeAppMenu();
  AstronomyAndroid.showLicences();
});

async function androidBootstrap() {
  androidLoadSavedSettings();
  await loadSetup();
  updateBottomNav();
  if (androidLocation) {
    await load();
  } else {
    $('setup-card').hidden = false;
    $('setup-button').setAttribute('aria-expanded', 'true');
    $('loading').textContent = 'Choose an observing location to begin. Nothing needs to be hosted or connected to Home Assistant.';
  }
  setInterval(load, 30 * 60 * 1000);
}

androidBootstrap();
