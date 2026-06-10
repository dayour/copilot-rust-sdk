---
id: plans
title: Plans
sidebar_label: Plans
---

# Plans

A session can maintain a structured plan - typically a checklist of steps the agent intends to follow. The SDK exposes simple CRUD over the plan via [`PlanData`](/docs/api/types#plandata).

## The plan shape

```rust
pub struct PlanData {
    pub content: Option<String>,  // the plan body (often markdown)
    pub title: Option<String>,
}
```

## Reading the plan

```rust
if let Some(plan) = session.read_plan().await? {
    if let Some(title) = &plan.title {
        println!("# {title}");
    }
    if let Some(content) = &plan.content {
        println!("{content}");
    }
} else {
    println!("no plan yet");
}
```

`read_plan` returns `Option<PlanData>` - `None` when no plan exists.

## Updating the plan

```rust
use copilot_sdk::PlanData;

session.update_plan(&PlanData {
    title: Some("Implementation Plan".into()),
    content: Some("Step 1: Implement\nStep 2: Test\nStep 3: Document".into()),
}).await?;
```

Updating replaces the stored plan with the values you provide.

## Deleting the plan

```rust
session.delete_plan().await?;
```

## Typical workflow

Plans shine in [`Plan` mode](/docs/guides/modes):

```rust
use copilot_sdk::SessionMode;

// 1. Switch to plan mode and let the agent draft a plan.
session.set_mode(SessionMode::Plan).await?;
session.send_and_collect("Plan how to add CSV export.", None).await?;

// 2. Review what the agent produced.
let plan = session.read_plan().await?;

// 3. Optionally refine it yourself, then execute in autopilot.
session.set_mode(SessionMode::Autopilot).await?;
session.send("Execute the plan.").await?;
```

See the `plan_ops` entry in the [examples catalog](/docs/examples) for a complete CRUD demo.

## Related

- [Modes](/docs/guides/modes)
- [Workspace](/docs/guides/workspace) - the agent often writes a `plan.md` into the workspace.
