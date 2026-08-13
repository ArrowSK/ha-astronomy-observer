let webLocation = null;

function webLocationFromForm() {
  return {
    latitude: Number(document.getElementById('web-location-lat').value),
    longitude: Number(document.getElementById('web-location-lon').value),
    elevation_m: Number(document.getElementById('web-location-elevation').value || 0),
    label: document.getElementById('web-location-label').value.trim() || 'Observing site',
    timezone: document.getElementById('web-location-timezone').value.trim() || 'UTC'
  };
}
