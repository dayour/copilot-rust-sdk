// Copyright (c) 2026 Elias Bachaalany
// SPDX-License-Identifier: MIT

//! Demonstrates the session-scoped `session.mcp.*`, `session.mcp.apps.*`, and
//! `session.mcp.oauth.*` namespaces with safe discovery-oriented calls.
//!
//! Run with:
//! `cargo run --example rpc_mcp`

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
    let mcp = session.mcp();

    println!("Session ID: {}\n", session.session_id());

    let servers = mcp.list().await?;
    println!("Configured MCP servers: {}", servers.servers.len());
    for server in &servers.servers {
        println!(
            "  server={} status={:?} source={:?} error={}",
            server.name,
            server.status,
            server.source,
            server.error.as_deref().unwrap_or("<none>")
        );
    }
    println!();

    match mcp.apps().get_host_context().await {
        Ok(host_context) => print_json("Current MCP App host context", &host_context),
        Err(err) => eprintln!("Could not read MCP App host context: {err}\n"),
    }

    if let Some(first_server) = servers.servers.first() {
        match mcp.apps().diagnose(&first_server.name).await {
            Ok(diagnostics) => print_json("MCP App diagnostics for the first server", &diagnostics),
            Err(err) => eprintln!(
                "Could not diagnose MCP App wiring for `{}`: {err}\n",
                first_server.name
            ),
        }
    } else {
        println!("No MCP servers were configured, so there was nothing to diagnose.\n");
    }

    println!("Mutating APIs such as enable/disable, reload, set_env_value_mode,");
    println!("read_resource, call_tool, and oauth.login are intentionally left as comments.\n");

    // Mutating or server-specific examples, intentionally not executed:
    // let _ = session.mcp().enable("github").await?;
    // let _ = session.mcp().apps().call_tool("github", "search_code", None, "github").await?;
    // let _ = session.mcp().oauth().login("some-remote-server", None, None, None).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> copilot_sdk::Result<()> {
    println!("=== RPC MCP Example ===\n");

    let Some(client) = start_authenticated_client().await? else {
        return Ok(());
    };

    let result = run_example(&client).await;
    client.stop().await;
    result
}
