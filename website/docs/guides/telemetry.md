---
id: telemetry
title: OpenTelemetry
sidebar_label: Telemetry
---

# OpenTelemetry

You can configure the Copilot CLI process to export distributed traces via OpenTelemetry. This is set on the client, before `start`, using [`TelemetryConfig`](/docs/api/types#telemetryconfig).

## Configuring telemetry

```rust
use copilot_sdk::{Client, TelemetryConfig};

let client = Client::builder()
    .telemetry(TelemetryConfig {
        otlp_endpoint: Some("http://localhost:4318".into()),
        exporter_type: Some("otlp-http".into()),
        source_name: Some("my-app".into()),
        capture_content: Some(true),
        file_path: None,
    })
    .build()?;
```

## Configuration fields

| Field | Type | Purpose |
|-------|------|---------|
| `otlp_endpoint` | `Option<String>` | OTLP collector endpoint (for example `http://localhost:4318`). |
| `exporter_type` | `Option<String>` | Exporter to use (for example `otlp-http`). |
| `source_name` | `Option<String>` | Logical source/service name attached to spans. |
| `capture_content` | `Option<bool>` | Whether to include message content in spans. |
| `file_path` | `Option<String>` | Write telemetry to a file instead of (or in addition to) a collector. |

## Exporting to a file

For local debugging without a collector, write spans to a file:

```rust
let client = Client::builder()
    .telemetry(TelemetryConfig {
        file_path: Some("telemetry.jsonl".into()),
        source_name: Some("my-app".into()),
        capture_content: Some(false),
        otlp_endpoint: None,
        exporter_type: None,
    })
    .build()?;
```

## Privacy

`capture_content` controls whether prompts and responses are recorded in spans. Leave it disabled (or `false`) in environments where prompts may contain sensitive data. Avoid exporting content to shared collectors unless you have reviewed the privacy implications. See the [security guidance](/docs/contributing#security).

## A local collector

Point `otlp_endpoint` at any OTLP/HTTP-compatible collector (the OpenTelemetry Collector, Jaeger with OTLP enabled, etc.) listening on the OTLP HTTP port (commonly `4318`).

See the `telemetry` entry in the [examples catalog](/docs/examples).

## Related

- [Client lifecycle](/docs/core-concepts/client-lifecycle)
- [Types API reference](/docs/api/types#telemetryconfig)
