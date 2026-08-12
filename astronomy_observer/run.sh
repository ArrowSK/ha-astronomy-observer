#!/usr/bin/with-contenv bashio
set -euo pipefail

bashio::log.info "Starting Astronomy Observer"
exec /usr/local/bin/astronomy-observer --options /data/options.json --data-dir /data --config-dir /config
