// Copyright (c) 2026 Elias Bachaalany
// SPDX-License-Identifier: MIT

//! Demonstrates the `session.skills.*`, `session.extensions.*`,
//! `session.plugins.*`, and `session.commands.*` namespaces with non-destructive
//! discovery calls.
//!
//! Run with:
//! `cargo run --example rpc_skills`

use copilot_sdk::rpc::skills::CommandsListRequest;
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

    session.skills().ensure_loaded().await?;

    let skills = session.skills().list().await?;
    println!("Discovered skills: {}", skills.skills.len());
    for skill in skills.skills.iter().take(5) {
        println!(
            "  skill={} enabled={} source={:?}",
            skill.name, skill.enabled, skill.source
        );
    }
    println!();

    let invoked = session.skills().get_invoked().await?;
    print_json("Skills invoked in this session", &invoked);

    let extensions = session.extensions().list().await?;
    println!("Discovered extensions: {}", extensions.extensions.len());
    for extension in extensions.extensions.iter().take(5) {
        println!(
            "  extension={} status={:?} source={:?}",
            extension.id, extension.status, extension.source
        );
    }
    println!();

    let plugins = session.plugins().list().await?;
    println!("Installed plugins: {}", plugins.plugins.len());
    for plugin in plugins.plugins.iter().take(5) {
        println!(
            "  plugin={} enabled={} version={}",
            plugin.name,
            plugin.enabled,
            plugin.version.as_deref().unwrap_or("<unknown>")
        );
    }
    println!();

    let commands = session
        .commands()
        .list(Some(CommandsListRequest {
            include_builtins: Some(true),
            include_skills: Some(true),
            include_client_commands: Some(true),
        }))
        .await?;
    println!("Available slash commands: {}", commands.commands.len());
    for command in commands.commands.iter().take(10) {
        println!(
            "  /{} kind={:?} allow_during_agent_execution={}",
            command.name, command.kind, command.allow_during_agent_execution
        );
    }
    println!();

    println!("Mutating APIs such as skill enable/disable, extension enable/disable,");
    println!("and command invoke/execute are intentionally left as comments.\n");

    // Mutating examples, intentionally not executed:
    // session.skills().enable("my-skill").await?;
    // session.extensions().disable("user:sample-extension").await?;
    // let _ = session.commands().invoke("help", None).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> copilot_sdk::Result<()> {
    println!("=== RPC Skills Example ===\n");

    let Some(client) = start_authenticated_client().await? else {
        return Ok(());
    };

    let result = run_example(&client).await;
    client.stop().await;
    result
}
