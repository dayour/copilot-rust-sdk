---
id: byok
title: BYOK (Bring Your Own Key)
sidebar_label: BYOK
---

# BYOK (Bring Your Own Key)

BYOK lets you point the agent at an OpenAI-compatible provider using your own API key, and optionally supply a custom list of models. This is useful for self-hosted models, alternative providers, or controlling cost and data residency.

## Provider configuration

Set a [`ProviderConfig`](/docs/api/types#providerconfig) on the session config:

```rust
use copilot_sdk::{SessionConfig, ProviderConfig};

let session = client.create_session(SessionConfig {
    provider: Some(ProviderConfig {
        base_url: "https://api.openai.com/v1".into(),
        api_key: Some(std::env::var("OPENAI_API_KEY")?),
        ..Default::default()
    }),
    ..Default::default()
}).await?;
```

`ProviderConfig` fields:

| Field | Purpose |
|-------|---------|
| `base_url` | The provider's API base URL. |
| `provider_type` | Optional provider type hint. |
| `wire_api` | Optional wire protocol selector. |
| `api_key` | API key for the provider. |
| `bearer_token` | Bearer token alternative to `api_key`. |
| `azure` | Azure-specific options (see below). |

## Auto-BYOK from environment

Set `auto_byok_from_env` to let the SDK pick up provider credentials from the environment automatically:

```rust
let session = client.create_session(SessionConfig {
    auto_byok_from_env: true,
    ..Default::default()
}).await?;
```

## Azure OpenAI

Provide [`AzureOptions`](/docs/api/types#azureoptions) for Azure-hosted models:

```rust
use copilot_sdk::{ProviderConfig, AzureOptions};

let provider = ProviderConfig {
    base_url: "https://my-resource.openai.azure.com".into(),
    api_key: Some(std::env::var("AZURE_OPENAI_KEY")?),
    azure: Some(AzureOptions {
        api_version: Some("2024-06-01".into()),
    }),
    ..Default::default()
};
```

## Custom model list

When using BYOK you often want to advertise your own models. Register a callback on the [client builder](/docs/core-concepts/client-lifecycle#building-a-client) with [`on_list_models`](/docs/api/client#builder-methods). It is an async function returning `Vec<ModelInfo>`:

```rust
use copilot_sdk::{Client, ModelInfo};

let client = Client::builder()
    .on_list_models(|| async {
        Ok(vec![
            // Construct ModelInfo values describing your models.
        ])
    })
    .build()?;
```

This callback feeds [`client.list_models()`](/docs/guides/models) and the model picker. See [`ModelInfo`](/docs/api/types#modelinfo) for the full structure (capabilities, limits, billing, reasoning efforts).

## Putting it together

```rust
use copilot_sdk::{Client, SessionConfig, ProviderConfig};

let client = Client::builder()
    .on_list_models(|| async { Ok(vec![/* your ModelInfo list */]) })
    .build()?;
client.start().await?;

let session = client.create_session(SessionConfig {
    provider: Some(ProviderConfig {
        base_url: "https://api.openai.com/v1".into(),
        api_key: Some(std::env::var("OPENAI_API_KEY")?),
        ..Default::default()
    }),
    auto_byok_from_env: true,
    ..Default::default()
}).await?;
```

## Security

Never hardcode keys. Read them from the environment or a secrets manager, and avoid logging them. See the [security guidance](/docs/contributing#security).

See the `byok` and `list_models` entries in the [examples catalog](/docs/examples).

## Related

- [Models](/docs/guides/models)
- [Types API reference](/docs/api/types#providerconfig)
