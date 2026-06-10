---
id: user-input
title: User Input
sidebar_label: User Input
---

# User Input

Sometimes the agent needs to ask the human a question mid-task - to disambiguate, confirm, or choose between options. The SDK surfaces these as user-input requests you answer with a handler.

## Enabling user input

Set `request_user_input` on the session config so requests are routed to your handler:

```rust
use copilot_sdk::SessionConfig;

let session = client.create_session(SessionConfig {
    request_user_input: Some(true),
    ..Default::default()
}).await?;
```

## Registering a handler

A [`UserInputHandler`](/docs/api/session#type-aliases) receives a [`UserInputRequest`](/docs/api/types#userinputrequest) and a [`UserInputInvocation`](/docs/api/types#userinputinvocation), and returns a [`UserInputResponse`](/docs/api/types#userinputresponse).

```rust
use copilot_sdk::UserInputResponse;

session.register_user_input_handler(|request, _invocation| {
    println!("Agent asks: {}", request.question);

    if let Some(choices) = &request.choices {
        // Present choices; here we just pick the first.
        UserInputResponse {
            answer: choices[0].clone(),
            was_freeform: Some(false),
        }
    } else {
        // Freeform answer.
        UserInputResponse {
            answer: "yes".into(),
            was_freeform: Some(true),
        }
    }
}).await;
```

### The request

```rust
pub struct UserInputRequest {
    pub question: String,
    pub choices: Option<Vec<String>>,   // present for multiple-choice
    pub allow_freeform: Option<bool>,   // whether a custom answer is allowed
}
```

- If `choices` is present, return one of them (or a freeform answer if `allow_freeform` is true).
- If `choices` is absent, return a freeform `answer`.

### The response

```rust
pub struct UserInputResponse {
    pub answer: String,
    pub was_freeform: Option<bool>,
}
```

## Checking for a handler

```rust
if session.has_user_input_handler().await {
    println!("user input will be handled programmatically");
}
```

## Wiring to a real UI

In an interactive app, your handler would block on actual user input (a prompt, a dialog, a web form). Keep it responsive: the agent's turn is paused until you return a response. For terminal apps, read from stdin; for GUIs, await a UI event.

See the `user_input` entry in the [examples catalog](/docs/examples).

## Related

- [Permissions](/docs/guides/permissions) - approve/deny actions (distinct from answering questions).
- [Hooks](/docs/guides/hooks) - intercept lifecycle points.
