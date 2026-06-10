---
id: models
title: Models
sidebar_label: Models
---

# Models

You can list available models, choose one at session creation, and switch models (and reasoning effort) mid-session.

## Listing models

```rust
let models = client.list_models().await?;
for m in &models {
    println!("{} - {}", m.id, m.name);
}
```

Each [`ModelInfo`](/docs/api/types#modelinfo) describes capabilities and limits:

```rust
let m = &models[0];
println!("vision: {}", m.capabilities.supports.vision);
println!("max context tokens: {}", m.capabilities.limits.max_context_window_tokens);
if let Some(efforts) = &m.supported_reasoning_efforts {
    println!("reasoning efforts: {efforts:?}");
}
```

The list is cached on the client. Force a refresh with:

```rust
client.clear_models_cache().await;
```

## Selecting a model at creation

```rust
use copilot_sdk::SessionConfig;

let session = client.create_session(SessionConfig {
    model: Some("gpt-4.1".into()),
    ..Default::default()
}).await?;
```

## Inspecting the current model

```rust
let model = session.get_model().await?;
println!("current model: {model}");
```

## Switching models mid-session

[`set_model`](/docs/api/session#model-management) changes the model for subsequent turns. Optionally set the reasoning effort via [`SetModelOptions`](/docs/api/types#setmodeloptions):

```rust
use copilot_sdk::SetModelOptions;

session.set_model("claude-sonnet-4", Some(SetModelOptions {
    reasoning_effort: Some("high".into()),
})).await?;
```

Pass `None` to keep the default reasoning effort:

```rust
session.set_model("gpt-4.1", None).await?;
```

When the model changes you will also observe a [`SessionModelChange`](/docs/core-concepts/events#session-lifecycle) event on the stream.

## Reasoning effort

Models that support reasoning expose `supported_reasoning_efforts` and a `default_reasoning_effort` on [`ModelInfo`](/docs/api/types#modelinfo). You can set the effort:

- at creation, via `SessionConfig.reasoning_effort`, or
- on switch, via `SetModelOptions.reasoning_effort`.

See the `reasoning_effort` and `set_model` examples in the [examples catalog](/docs/examples).

## Related

- [Modes](/docs/guides/modes)
- [BYOK](/docs/guides/byok) - use your own provider and custom model list.
