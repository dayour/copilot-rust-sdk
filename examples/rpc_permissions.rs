// Copyright (c) 2026 Elias Bachaalany
// SPDX-License-Identifier: MIT

//! Demonstrates the read-focused `session.permissions.*` APIs for inspecting
//! permission state, allowed paths, location-scoped rules, folder trust, and
//! the URL-permission surface.
//!
//! Run with:
//! `cargo run --example rpc_permissions`

use copilot_sdk::{find_copilot_cli, Client, SessionConfig};
use serde::Serialize;

fn print_json<T: Serialize>(label: &str, value: &T) {
    match serde_json::to_string_pretty(value) {
        Ok(json) => println!("{label}:\n{json}\n"),
        Err(err) => println!("{label}: <failed to render JSON: {err}>"),
    }
}

async fn start_authenticated_client() -> copilot_sdk::Result<Option<Client>> {
    if find_copilot_cli().is_none() {
        println!(
            "Copilot CLI not found. Set `COPILOT_CLI_PATH` or install/authenticate `copilot`."
        );
        return Ok(None);
    }

    let client = Client::builder().use_stdio(true).build()?;
    if let Err(err) = client.start().await {
        eprintln!("Failed to start the Copilot CLI: {err}");
        return Ok(None);
    }

    match client.get_auth_status().await {
        Ok(status) if status.is_authenticated => Ok(Some(client)),
        Ok(status) => {
            let detail = status
                .status_message
                .unwrap_or_else(|| "Run `copilot auth login` and try again.".to_string());
            println!("Copilot CLI is installed but not authenticated. {detail}");
            client.stop().await;
            Ok(None)
        }
        Err(err) => {
            eprintln!("Failed to read Copilot authentication status: {err}");
            client.stop().await;
            Ok(None)
        }
    }
}

async fn run_example(client: &Client) -> copilot_sdk::Result<()> {
    let session = client.create_session(SessionConfig::default()).await?;
    let permissions = session.permissions();
    let paths = permissions.paths();
    let locations = permissions.locations();
    let folder_trust = permissions.folder_trust();

    println!("Session ID: {}\n", session.session_id());

    let allow_all = permissions.get_allow_all().await?;
    print_json("Allow-all state", &allow_all);

    let pending = permissions.pending_requests().await?;
    print_json("Pending permission requests", &pending);

    let cwd = std::env::current_dir()?;
    let cwd_text = cwd.to_string_lossy().into_owned();
    let examples_dir_text = cwd.join("examples").to_string_lossy().into_owned();

    let allowed_dirs = paths.list().await?;
    print_json("Allowed directories", &allowed_dirs);

    let within_allowed = paths
        .is_path_within_allowed_directories(&examples_dir_text)
        .await?;
    println!(
        "Is `{examples_dir_text}` within an allowed directory? {}\n",
        within_allowed.allowed
    );

    let within_workspace = paths.is_path_within_workspace(&examples_dir_text).await?;
    println!(
        "Is `{examples_dir_text}` within the session workspace? {}\n",
        within_workspace.allowed
    );

    let location = locations.resolve(&cwd_text).await?;
    print_json("Resolved permission location", &location);

    let applied = locations.apply(&cwd_text).await?;
    print_json("Applied location-scoped permissions", &applied);

    let trusted = folder_trust.is_trusted(&cwd_text).await?;
    println!("Is `{cwd_text}` trusted? {}\n", trusted.trusted);

    println!("URL permissions are exposed through `session.permissions().urls()`.");
    println!(
        "This example avoids calling `set_unrestricted_mode(...)` because it mutates user state.\n"
    );

    // Mutating example, intentionally not executed:
    // let _ = session.permissions().urls().set_unrestricted_mode(true).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> copilot_sdk::Result<()> {
    println!("=== RPC Permissions Example ===\n");

    let Some(client) = start_authenticated_client().await? else {
        return Ok(());
    };

    let result = run_example(&client).await;
    client.stop().await;
    result
}
