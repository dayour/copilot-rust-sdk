---
id: attachments
title: Attachments
sidebar_label: Attachments
---

# Attachments

Messages can carry attachments that give the agent additional context: whole files, directories, or a specific text selection within a file.

## Attachment types

The [`AttachmentType`](/docs/api/types#attachmenttype) enum has three variants:

| Variant | Meaning |
|---------|---------|
| `File` | Attach a single file. |
| `Directory` | Attach a directory. |
| `Selection` | Attach a text selection (a range within a file). |

## Attaching files and directories

Build [`UserMessageAttachment`](/docs/api/types#usermessageattachment) values and pass them in [`MessageOptions`](/docs/api/types#messageoptions):

```rust
use copilot_sdk::{MessageOptions, UserMessageAttachment, AttachmentType};

let options = MessageOptions {
    prompt: "Review these for bugs.".into(),
    attachments: Some(vec![
        UserMessageAttachment {
            attachment_type: AttachmentType::File,
            path: "src/main.rs".into(),
            display_name: "main.rs".into(),
        },
        UserMessageAttachment {
            attachment_type: AttachmentType::Directory,
            path: "src/handlers".into(),
            display_name: "handlers".into(),
        },
    ]),
    mode: None,
};

session.send(options).await?;
```

## Attaching a text selection

For selections, the SDK provides [`SelectionAttachment`](/docs/api/types#selectionattachment) with a [`SelectionRange`](/docs/api/types#selectionrange) of [`SelectionPosition`](/docs/api/types#selectionposition) (line/character) values - the same shape editors use:

```rust
use copilot_sdk::{SelectionAttachment, SelectionRange, SelectionPosition};

let selection = SelectionAttachment {
    file_path: "src/lib.rs".into(),
    display_name: "lib.rs".into(),
    text: "fn add(a: i32, b: i32) -> i32 { a + b }".into(),
    selection: SelectionRange {
        start: SelectionPosition { line: 10.0, character: 0.0 },
        end:   SelectionPosition { line: 10.0, character: 39.0 },
    },
};
```

This mirrors how an editor integration would attach the user's current selection so the agent can reason about a precise region of code.

## Observing attachments on the stream

Inbound user messages echo their attachments via [`UserMessageData`](/docs/api/events) (`attachments: Option<Vec<UserMessageAttachmentItem>>`), so tooling can display what was attached.

See the `attachments` entry in the [examples catalog](/docs/examples).

## Related

- [Messaging](/docs/guides/messaging)
- [Workspace](/docs/guides/workspace)
