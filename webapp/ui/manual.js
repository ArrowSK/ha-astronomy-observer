let webLocation = null;
let webMinimumAltitude = 20;
let webHorizonMask = '0:0,90:0,180:0,270:0';

function webLocationFromForm() {
  return {
    latitude: Number(document.getElementById('web-location-lat').value),
    longitude: Number(document.getElementById('web-location-lon').value),
    elevation_m: Number(document.getElementById('web-location-elevation').value || 0),
    label: document.getElementById('web-location-label').value.trim() || 'Observing site',
    timezone: document.getElementById('web-location-timezone').value.trim() || 'UTC'
  };
}

loadSetup = async function() {
  const altitude = String(Math.round(webMinimumAltitude));
  if (![...$('minimum-altitude').options].some(option => option.value === altitude)) {
    $('minimum-altitude').insertAdjacentHTML('beforeend', `<option value="${esc(altitude)}">${esc(altitude)}° — current value</option>`);
  }
  $('minimum-altitude').value = altitude;
  $('horizon-mask').value = webHorizonMask;
  $('setup-status').textContent = 'The standalone web service uses the observing site entered here.';
  setupLoaded = true;
};

load = async function() {
  if (!webLocation) {
    $('content').hidden = true;
    $('loading').hidden = false;
    $('loading').textContent = 'Enter an observing location in Setup to calculate tonight.';
    return;
  }
  $('loading').hidden = false;
  $('loading').textContent = 'Calculating conditions for this location…';
  try {
    const response = await fetch('api/web/snapshot', {
      method: 'POST',
      headers: {'Content-Type':'application/json'},
      body: JSON.stringify({
        location: webLocation,
        minimum_target_altitude: webMinimumAltitude,
        horizon_mask: webHorizonMask
      })
    });
    const result = await response.json();
    if (!response.ok) throw new Error(result.error || `HTTP ${response.status}`);
    render(result);
  } catch (error) {
    $('content').hidden = true;
    $('loading').hidden = false;
    $('loading').textContent = `Unable to calculate the current result: ${error.message}`;
  }
};

document.addEventListener('click', async event => {
  const button = event.target.closest('#save-setup');
  if (!button) return;
  event.preventDefault();
  event.stopImmediatePropagation();

  const location = webLocationFromForm();
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

  webLocation = location;
  webMinimumAltitude = Number($('minimum-altitude').value);
  webHorizonMask = $('horizon-mask').value.trim() || '0:0,90:0,180:0,270:0';
  button.disabled = true;
  $('setup-status').textContent = 'Calculating…';
  $('setup-card').hidden = true;
  $('setup-button').setAttribute('aria-expanded', 'false');
  try {
    await load();
    $('setup-status').textContent = '';
  } finally {
    button.disabled = false;
  }
}, true);

async function webBootstrap() {
  await loadSetup();
  $('setup-card').hidden = false;
  $('setup-button').setAttribute('aria-expanded', 'true');
  $('loading').textContent = 'Enter an observing location in Setup to calculate tonight.';
  updateBottomNav();
}

webBootstrap();
