<div align="center">
<p align="center">
  <a href="https://www.edgee.ai">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="https://cdn.edgee.ai/img/component-dark.svg">
      <img src="https://cdn.edgee.ai/img/component.svg" height="100" alt="Edgee">
    </picture>
  </a>
</p>
</div>

<h1 align="center">Matomo component for Edgee</h1>

[![Coverage Status](https://coveralls.io/repos/github/edgee-ai/matomo-component/badge.svg)](https://coveralls.io/github/edgee-ai/matomo-component)
[![GitHub issues](https://img.shields.io/github/issues/edgee-ai/matomo-component.svg)](https://github.com/edgee-ai/matomo-component/issues)
[![Edgee Component Registry](https://img.shields.io/badge/Edgee_Component_Registry-Public-green.svg)](https://www.edgee.ai/edgee/matomo)

This is a Rust-based Edgee component that integrates Matomo analytics using the Edgee Data Collection protocol. It allows you to send user events, page views, and user identity data directly to your Matomo instance from the edge.

---

## ✨ Features

- ✅ Track custom user events (`track`)
- ✅ Track page views (`page`)
- ✅ Identify users and send user properties (`user`)
- ✅ Built for edge execution: fast, secure, serverless
- ✅ Supports `_cvar` custom variables
- ✅ Automatic enrichment with context (campaign, session, client, etc.)

---

## 🔧 Settings

This component requires the following settings:

| Key                   | Type   | Required | Description                                                        |
|-----------------------|--------|----------|--------------------------------------------------------------------|
| `site_id`             | string | ✅       | Your Matomo site ID                                                |
| `endpoint_url`        | string | ✅       | Full URL of your Matomo instance (e.g. `https://matomo.example.com`) |
| `authentication_token` | string | ❌       | Optional `token_auth` if needed for enhanced tracking              |

---

## 🧪 Testing Locally

### 🛠️ Build the component

```bash
edgee component build
```

### ✅ Run unit tests

```bash
cargo test
```

### 🔍 Run a live test with simulated events

```bash
edgee components test \
  --event-type page \
  --settings site_id=your_site_id,endpoint_url=https://your-matomo-instance.com,authentication_token=YOUR_TOKEN \
  --make-http-request
```

Replace `event-type` with `track` or `user` to test other event types.

---

### 📂 Project Structure

```text
matomo-component/
├── src/
│   └── lib.rs                 # Main component logic
├── target/
│   └── wasm32-wasip2/
│       └── release/
│           └── matomo.wasm    # Built WebAssembly output
├── matomo.png                 # Component icon
├── Cargo.toml                 # Rust dependencies
└── edgee-component.toml       # Edgee manifest
```

---

### 📚 Learn More

- [Matomo Tracking API](https://matomo.org/docs/tracking-api/)
- [Edgee Developer Guide](https://www.edgee.ai/docs/services/registry/developer-guide)
