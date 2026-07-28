// Copyright (c) 2026 Elias Bachaalany
// SPDX-License-Identifier: MIT

//! Demonstrates the `session.metadata.*`, `session.name.*`,
//! `session.usage.*`, `session.instructions.*`, and telemetry-related surfaces
//! using read-only inspection calls where possible.
//!
//! Run with:
//! `cargo run --example rpc_metadata`

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

    println!("Session ID: {}\n", session.session_id());

    let name = session.name().get().await?;
    print_json("Session name", &name);

    let snapshot = session.metadata().snapshot().await?;
    print_json("Metadata snapshot", &snapshot);

    let processing = session.metadata().is_processing().await?;
    print_json("Processing state", &processing);

    let context_info = session
        .metadata()
        .context_info(0, 0, snapshot.selected_model.as_deref())
        .await?;
    print_json("Context window token info", &context_info);

    if let Some(model_id) = snapshot.selected_model.as_deref() {
        let recomputed = session
            .metadata()
            .recompute_context_tokens(model_id)
            .await?;
        print_json("Recomputed token counts", &recomputed);
    } else {
        println!("No selected model was reported in the snapshot, so recompute_context_tokens() was skipped.\n");
    }

    let usage = session.usage().get_metrics().await?;
    print_json("Usage metrics", &usage);

    let instructions = session.instructions().get_sources().await?;
    println!("Instruction sources loaded: {}", instructions.sources.len());
    for source in instructions.sources.iter().take(5) {
        println!(
            "  id={} label={} path={} type={:?}",
            source.id, source.label, source.source_path, source.r#type
        );
    }
    println!();

    println!(
        "Telemetry currently exposes `set_feature_overrides(...)`, which mutates session state."
    );
    println!("This example intentionally leaves that call commented out.\n");

    // Mutating example, intentionally not executed:
    // session.telemetry().set_feature_overrides(std::collections::BTreeMap::new()).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> copilot_sdk::Result<()> {
    println!("=== RPC Metadata Example ===\n");

    let Some(client) = start_authenticated_client().await? else {
        return Ok(());
    };

    let result = run_example(&client).await;
    client.stop().await;
    result
}
