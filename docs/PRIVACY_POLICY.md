# LocaGate Privacy Policy

**Last updated:** June 8, 2026  
**Publisher:** ChinhDEV  
**Contact:** phamchinh347@gmail.com  
**Website:** https://chinh.dev

---

## 1. Introduction

LocaGate ("the App", "we", "our") is a desktop tunnel proxy application that helps you expose local web servers to the internet through a relay server you configure (typically a self-hosted Cloudflare Worker).

This Privacy Policy explains what information the App processes, how it is stored, and what choices you have. By installing or using LocaGate, you agree to this policy.

---

## 2. Summary

| Topic | LocaGate Practice |
|-------|-------------------|
| Account registration | Not required |
| Analytics / telemetry | Not collected by the App |
| Data sold to third parties | No |
| Local storage | Yes — settings and request logs |
| Internet transmission | Yes — only to your configured relay and local servers |
| Children | Not intended for users under 13 |

---

## 3. Information We Do Not Collect

LocaGate does **not**:

- Require you to create an account
- Collect your name, email, phone number, or payment information
- Run analytics, crash reporting, or advertising SDKs
- Send usage data to ChinhDEV servers

The App works primarily on your device and connects only to endpoints **you** configure.

---

## 4. Information Stored Locally on Your Device

When you use LocaGate, the following data may be stored on your computer in the application data folder:

### 4.1 Application Settings

- Cloudflare Worker URL (relay server address)
- Authentication token for your relay server
- Tunnel configurations (name, local address, host header, tunnel type)
- Maximum number of request logs to keep in memory

This data is saved in a local `config.json` file and remains on your device unless you delete it or uninstall the App.

### 4.2 Request Logs (In-Memory)

While a tunnel is active, LocaGate may temporarily store:

- HTTP method, path, status code, duration, and timestamp
- Request and response headers
- Request and response body content (cached up to 512 KB per request for inspection in the App UI)

Request logs are kept in memory during the App session. Older entries are removed automatically based on your configured log limit. You can clear logs at any time using the **Clear Logs** button.

When you close the App, in-memory logs are discarded. Settings in `config.json` are retained.

---

## 5. Information Transmitted Over the Network

When you start a tunnel, LocaGate:

1. Opens a **WebSocket connection** to the Cloudflare Worker URL you configured
2. Sends your **authentication token** to authenticate with your relay server
3. Relays **HTTP/HTTPS traffic** between public visitors and your local server

Traffic relayed through the tunnel may include any data sent to your local application (for example: URLs, headers, cookies, form data, API payloads). **You control what data flows through the tunnel** based on how your local server and public endpoints are used.

LocaGate does not add third-party tracking to relayed traffic.

---

## 6. Self-Hosted Relay Server (Cloudflare Worker)

If you deploy the included Cloudflare Worker relay (or a modified version), that server may temporarily store:

- Request metadata: method, path, status code, duration, and request ID
- Up to 100 recent log entries in Cloudflare Durable Object storage

That relay is **operated by you** (or your organization), not by ChinhDEV. Its data handling is subject to:

- Your Cloudflare account settings and policies
- [Cloudflare's Privacy Policy](https://www.cloudflare.com/privacypolicy/)

If you use a relay operated by someone else, their privacy practices apply to traffic passing through that relay.

---

## 7. Third-Party Services

LocaGate may interact with the following third-party components:

| Service | Purpose | Data Shared |
|---------|---------|-------------|
| **Your Cloudflare Worker** | Tunnel relay | Traffic and auth token you configure |
| **Your local web server** | Tunnel destination | Relayed HTTP requests |
| **Microsoft WebView2** | App UI runtime (Windows) | Standard WebView2 behavior per Microsoft |
| **System browser** | Opening external links (if used) | URL you choose to open |

LocaGate does not integrate with social networks, ad networks, or cloud analytics platforms.

---

## 8. Permissions and System Access

On Windows, LocaGate may request capabilities required for a desktop networking tool, including:

- Internet access (to connect to your relay server)
- Local network access (to reach your local servers)
- Full trust desktop execution (standard for non-sandboxed desktop apps)

The App does not access your camera, microphone, contacts, or SMS.

---

## 9. Data Security

- Authentication tokens and settings are stored locally on your device. Protect your computer and do not share your relay token.
- Relay traffic is transmitted over encrypted connections (WSS/HTTPS) when supported by your Worker configuration.
- You are responsible for securing your relay server, auth token, and local applications exposed through the tunnel.

No method of transmission or storage is 100% secure. Use LocaGate only in environments you trust.

---

## 10. Data Retention and Deletion

| Data | Retention |
|------|-----------|
| App settings (`config.json`) | Until you change, delete, or uninstall the App |
| In-memory request logs | Until cleared, log limit reached, or App closed |
| Relay server logs (Worker) | Up to 100 entries; controlled by your Worker deployment |

**To delete local App data:**

1. Clear logs in the App, or  
2. Uninstall LocaGate and remove its application data folder

**To delete relay server data:** Redeploy or clear storage on your Cloudflare Worker / Durable Object.

---

## 11. Your Responsibilities

Because LocaGate exposes local services to the internet, you are responsible for:

- Choosing what local applications and data are exposed
- Keeping your auth token and Worker URL confidential
- Complying with applicable laws regarding data you process through tunnels
- Informing **your** users if you expose services that handle their personal data

LocaGate is a developer tool. You act as the data controller for any personal data processed through tunnels you create.

---

## 12. Children's Privacy

LocaGate is not directed at children under 13 (or the minimum age required in your jurisdiction). We do not knowingly collect personal information from children.

---

## 13. International Users

LocaGate is available worldwide. Data is processed on your local device and on infrastructure you configure (for example, Cloudflare's global network). By using the App, you understand that relayed data may cross international borders according to your relay provider's infrastructure.

---

## 14. Changes to This Policy

We may update this Privacy Policy from time to time. The "Last updated" date at the top will reflect the latest version. Continued use of the App after changes constitutes acceptance of the updated policy.

Material changes may also be noted in release notes or the Microsoft Store listing.

---

## 15. Contact Us

If you have questions about this Privacy Policy or LocaGate's data practices:

- **Email:** phamchinh347@gmail.com  
- **Publisher:** ChinhDEV  
- **GitHub:** https://github.com/chinhps/LocaGate

---

## 16. Microsoft Store Disclosure Summary

For Microsoft Partner Center privacy questionnaire:

- **Personal data collected by publisher:** None directly
- **Personal data processed by app functionality:** User-configured network traffic and local settings (stored locally)
- **Data shared with third parties:** Only to user-configured relay infrastructure (e.g., Cloudflare Worker)
- **Data used for tracking:** No
- **Data linked to user identity:** No (no accounts)
- **Data encrypted in transit:** Yes (WSS/HTTPS for relay connections)
- **Users can request data deletion:** Yes — clear logs in App or uninstall; relay data managed by user's own server
