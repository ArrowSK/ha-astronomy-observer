# Security

Astronomy Observer runs as a Home Assistant app with Ingress and Home Assistant's default AppArmor restrictions. It does not request privileged mode, host networking, Docker access or write access to Home Assistant configuration.

The Ingress HTTP service accepts requests only from Home Assistant's Ingress proxy address or loopback. The app panel is admin-only because the in-memory result includes the selected observing location.

Do not publish Home Assistant access tokens, exact private location data or a working exploit in a public issue. For a security problem, contact the repository owner privately through GitHub first so the issue can be reproduced and fixed before detailed public disclosure.

Please include the app version, Home Assistant version, host architecture and a minimal description of the affected endpoint or data path. Remove tokens and private coordinates from logs before sending them.
