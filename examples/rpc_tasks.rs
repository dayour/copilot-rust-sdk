// Copyright (c) 2026 Elias Bachaalany
// SPDX-License-Identifier: MIT

//! Demonstrates the `session.tasks.*` namespace for listing tracked background
//! work, checking promotable tasks, and inspecting task progress without
//! changing task state.
//!
//! Run with:
//! `cargo run --example rpc_tasks`

use copilot_sdk::rpc::tasks::TaskInfo;
use copilot_sdk::{find_copilot_cli, Client, SessionConfig};
use serde::Serialize;

fn print_json<T: Serialize>(label: &str, value: &T) {
    match serde_json::to_string_pretty(value) {
        Ok(json) => println!("{label}:\n{json}\n"),
        Err(err) => println!("{label}: <failed to render JSON: {err}>"),
    }
}

fn task_id(task: &TaskInfo) -> &str {
    match task {
        TaskInfo::Agent(info) => &info.id,
        TaskInfo::Shell(info) => &info.id,
    }
}

fn print_task_summary(task: &TaskInfo) {
    match task {
        TaskInfo::Agent(info) => println!(
            "Agent task: id={} status={:?} agent_type={} description={}",
            info.id, info.status, info.agent_type, info.description
        ),
        TaskInfo::Shell(info) => println!(
            "Shell task: id={} status={:?} command={} description={}",
            info.id, info.status, info.command, info.description
        ),
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
    let tasks = session.tasks();

    println!("Session ID: {}\n", session.session_id());

    tasks.refresh().await?;
    println!("Refreshed background-task metadata.\n");

    let task_list = tasks.list().await?;
    println!("Tracked tasks: {}\n", task_list.tasks.len());
    for task in &task_list.tasks {
        print_task_summary(task);
    }
    println!();

    let promotable = tasks.get_current_promotable().await?;
    print_json("Current promotable task", &promotable);

    if let Some(first_task) = task_list.tasks.first() {
        let progress = tasks.get_progress(task_id(first_task)).await?;
        print_json("Progress for the first tracked task", &progress);
    } else {
        println!("No tracked tasks were available, so there was nothing to inspect with get_progress().\n");
    }

    println!("Mutating task APIs such as start_agent, promote_to_background, send_message, cancel, and remove");
    println!("are intentionally left as comments to keep this example read-only.\n");

    // Mutating examples, intentionally not executed:
    // let started = session.tasks().start_agent("explore", "Investigate the codebase", "example-agent", None, None).await?;
    // let _ = session.tasks().send_message(&started.agent_id, "Hello from the host", None).await?;
    // let _ = session.tasks().cancel(&started.agent_id).await?;
    // let _ = session.tasks().remove(&started.agent_id).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> copilot_sdk::Result<()> {
    println!("=== RPC Tasks Example ===\n");

    let Some(client) = start_authenticated_client().await? else {
        return Ok(());
    };

    let result = run_example(&client).await;
    client.stop().await;
    result
}
