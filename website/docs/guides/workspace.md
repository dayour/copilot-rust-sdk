---
id: workspace
title: Workspace Files
sidebar_label: Workspace
---

# Workspace Files

Sessions - especially [infinite sessions](/docs/guides/infinite-sessions) - have an associated workspace where the agent persists artifacts. The SDK lets you list, read, and create files there.

## The workspace path

```rust
if let Some(path) = session.workspace_path() {
    println!("workspace at {path}");
}
```

The path is present for sessions that have a workspace (such as infinite sessions).

## Listing files

```rust
let files = session.workspace_list_files().await?;
for f in &files {
    println!("{} ({} bytes)", f.path, f.size.unwrap_or(0));
}
```

Each [`WorkspaceFile`](/docs/api/types#workspacefile) carries `path`, optional `size`, and optional `modified_at`.

## Reading a file

```rust
let content = session.workspace_read_file("plan.md").await?;
println!("{content}");
```

## Creating a file

```rust
session.workspace_create_file("notes.md", "# Notes\n\n- first item\n").await?;
```

## Typical pattern

The workspace is a durable scratch space that survives [compaction](/docs/guides/infinite-sessions). A common pattern is to keep a running plan or notes there:

```rust
// Seed a plan file the agent can read and update across turns.
session.workspace_create_file(
    "plan.md",
    "# Plan\n\n1. Investigate\n2. Implement\n3. Verify\n",
).await?;

// Later, read back what the agent recorded.
let plan = session.workspace_read_file("plan.md").await?;
```

## Related

- [Infinite Sessions](/docs/guides/infinite-sessions)
- [Plans](/docs/guides/plans) - structured plan CRUD distinct from raw files.
- [Shell](/docs/guides/shell)
