# PSLinkB Dashboard

A responsive dark control-room UI for the PSLinkB multi-platform streaming work.

## Current status

This commit adds the dashboard frontend only. It is not yet served by the PSLinkB binary, and the status/control API is not implemented. The UI intentionally keeps control buttons disabled until the backend can safely dispatch commands to the live state machine.

## Planned local API

- `GET /api/status`: live snapshot of PS5 input, per-platform RTMP output, danmaku, uptime, bitrate/FPS, and host resource metrics.
- `POST /api/control`: authenticated actions such as `start`, `stop`, `reconnect-bilibili`, and `reconnect-douyin`.
- Optional WebSocket endpoint for log streaming; polling is used by this first UI draft.

## LAN security requirements

Before enabling LAN access:
- Bind only to the selected private LAN interface, not `0.0.0.0` by default.
- Require an unpredictable session token and validate it for every control request.
- Protect against CSRF and reject untrusted Origin/Host headers.
- Never return RTMP stream keys, Bilibili cookies, or other credentials from status endpoints.
- Do not expose the dashboard to the public internet.

## Development

Open `dashboard/index.html` through a local HTTP server once the backend is implemented. Opening it directly as a `file://` URL will not make the API available.
