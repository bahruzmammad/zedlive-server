# Security notes

- The server binds to loopback (`127.0.0.1`) by default. Binding to `0.0.0.0` exposes the development site to network interfaces and must be an intentional choice.
- HTTP request paths are normalized and checked against the selected serving root. Existing symlinks are canonicalized and rejected when they resolve outside that root. Configured directory index paths must be relative; index resolution uses the same root containment check.
- This is a local development server, not a hardened public web server. Files may be served to clients that can reach the bound interface. Keep LAN exposure, directory listings, proxy rules, mounts, and CORS settings deliberate.
- The reload WebSocket accepts an absent `Origin` for non-browser clients, but when browsers send `Origin`, the scheme and authority must match the request host. CORS wildcard origins are not accepted together with credentialed CORS.
- The filesystem check and later file read are separate operations; concurrent filesystem changes can create a time-of-check/time-of-use race. Do not serve untrusted directories to untrusted users.
- Server state is stored in `.live-server/` inside the project root. Process identity/PID-reuse handling and atomic duplicate-start protection still need hardening before a stable release.
- Do not put credentials, private keys, or sensitive files in a served project root. Use HTTPS only with intentionally configured certificates; TLS behavior is not covered by the current integration tests.
