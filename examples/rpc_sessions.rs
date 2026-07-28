// Copyright (c) 2026 Elias Bachaalany
// SPDX-License-Identifier: MIT

//! Demonstrates the server-surface `sessions.*` APIs for listing persisted
//! sessions and inspecting a few session-level metadata views without changing
//! or deleting any saved state.
//!
//! Run with:
//! `cargo run --example rpc_sessions`

use copilot_sdk::rpc::sessions::{
    SessionsCheckInUseRequest, SessionsFindByPrefixRequest, SessionsFindByTaskIdRequest,
    SessionsGetEventFilePathRequest, SessionsGetLastForContextRequest,
    SessionsGetPersistedRemoteSteerableRequest, SessionsListRequest,
};
use copilot_sdk::{find_copilot_cli, Client};
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
    let sessions = client.sessions();

    let listed = sessions
        .list(Some(SessionsListRequest {
            metadata_limit: Some(10),
            filter: None,
            include_detached: Some(false),
        }))
        .await?;
    println!("Persisted sessions found: {}\n", listed.sessions.len());
    for session in listed.sessions.iter().take(5) {
        println!(
            "  session_id={} name={} remote={} summary={}",
            session.session_id,
            session.name.as_deref().unwrap_or("<unnamed>"),
            session.is_remote,
            session.summary.as_deref().unwrap_or("<none>")
        );
    }
    println!();

    let sizes = sessions.get_sizes().await?;
    print_json("Persisted session workspace sizes", &sizes);

    if let Some(first_session) = listed.sessions.first() {
        let prefix: String = first_session.session_id.chars().take(8).collect();
        let by_prefix = sessions
            .find_by_prefix(SessionsFindByPrefixRequest { prefix })
            .await?;
        print_json("Lookup by session ID prefix", &by_prefix);

        let in_use = sessions
            .check_in_use(SessionsCheckInUseRequest {
                session_ids: vec![first_session.session_id.clone()],
            })
            .await?;
        print_json("In-use check for the first session", &in_use);

        if !first_session.is_remote {
            let event_file = sessions
                .get_event_file_path(SessionsGetEventFilePathRequest {
                    session_id: first_session.session_id.clone(),
                })
                .await?;
            print_json("Event file path for the first session", &event_file);
        }

        let remote_steerable = sessions
            .get_persisted_remote_steerable(SessionsGetPersistedRemoteSteerableRequest {
                session_id: first_session.session_id.clone(),
            })
            .await?;
        print_json("Persisted remote-steerable flag", &remote_steerable);

        if let Some(context) = first_session.context.clone() {
            let last_for_context = sessions
                .get_last_for_context(SessionsGetLastForContextRequest {
                    context: Some(context),
                })
                .await?;
            print_json(
                "Best prior session for the first session's context",
                &last_for_context,
            );
        }

        if let Some(task_id) = first_session.mc_task_id.clone() {
            let by_task = sessions
                .find_by_task_id(SessionsFindByTaskIdRequest { task_id })
                .await?;
            print_json("Lookup by GitHub task ID", &by_task);
        }
    } else {
        println!("No persisted sessions were available to inspect more deeply.\n");
    }

    println!("Mutating APIs such as close, fork, prune_old, bulk_delete, release_lock,");
    println!("reload_plugin_hooks, save, and set_additional_plugins are intentionally left as comments.\n");

    // Mutating examples, intentionally not executed:
    // let _ = client.sessions().fork(SessionsForkRequest { session_id: "abc".into(), to_event_id: None, name: None }).await?;
    // let _ = client.sessions().prune_old(SessionsPruneOldRequest { older_than_days: 30, dry_run: Some(true), include_named: Some(false), exclude_session_ids: None }).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> copilot_sdk::Result<()> {
    println!("=== RPC Sessions Example ===\n");

    let Some(client) = start_authenticated_client().await? else {
        return Ok(());
    };

    let result = run_example(&client).await;
    client.stop().await;
    result
}
