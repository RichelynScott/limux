//! Slice B: live accept-path entitlement wire-up.
//!
//! These tests call `handle_connection_with_entitlement_config` directly so
//! they do not mutate process-wide `LIMUX_ENTITLEMENT` (which would race other
//! tests). The production `serve` / `handle_connection` paths still resolve
//! mode via `EntitlementConfig::from_env()`.

use limux_control::server::handle_connection_with_entitlement_config;
use limux_control::{Dispatcher, EntitlementConfig, EntitlementMode};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

async fn write_request(writer: &mut tokio::net::unix::OwnedWriteHalf, body: &str) {
    writer
        .write_all(body.as_bytes())
        .await
        .expect("request should write");
    writer.flush().await.expect("request should flush");
}

async fn read_json(reader: &mut BufReader<tokio::net::unix::OwnedReadHalf>) -> Value {
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .await
        .expect("response should read");
    serde_json::from_str(line.trim()).expect("response should be valid json")
}

#[tokio::test]
async fn require_claim_denies_unclaimed_bare_read_then_natural_claim_binds() {
    let (client, server) = UnixStream::pair().expect("unix stream pair");
    let dispatcher = Dispatcher::new();

    let lane_a = dispatcher
        .dispatch(limux_protocol::V2Request {
            id: Some(Value::String("seed-a".into())),
            method: "workspace.create".into(),
            params: serde_json::json!({ "name": "lane-a" }),
        })
        .await;
    let lane_a_id = lane_a.result.expect("a")["workspace"]["id"]
        .as_u64()
        .expect("a id");
    let lane_b = dispatcher
        .dispatch(limux_protocol::V2Request {
            id: Some(Value::String("seed-b".into())),
            method: "workspace.create".into(),
            params: serde_json::json!({ "name": "lane-b" }),
        })
        .await;
    let lane_b_id = lane_b.result.expect("b")["workspace"]["id"]
        .as_u64()
        .expect("b id");
    let surface = dispatcher
        .dispatch(limux_protocol::V2Request {
            id: Some(Value::String("seed-s".into())),
            method: "surface.current".into(),
            params: serde_json::json!({ "workspace_id": lane_b_id }),
        })
        .await;
    let surface_id = surface.result.expect("s")["surface"]["id"]
        .as_u64()
        .expect("s id");
    dispatcher
        .dispatch(limux_protocol::V2Request {
            id: Some(Value::String("seed-t".into())),
            method: "surface.send_text".into(),
            params: serde_json::json!({
                "surface_id": surface_id,
                "text": "WIREUP-TEXT"
            }),
        })
        .await;

    let server_task = tokio::spawn(async move {
        handle_connection_with_entitlement_config(
            server,
            dispatcher,
            EntitlementConfig {
                mode: EntitlementMode::RequireClaim,
            },
        )
        .await
    });

    let (reader_half, mut writer_half) = client.into_split();
    let mut reader = BufReader::new(reader_half);

    write_request(
        &mut writer_half,
        &format!(
            "{{\"id\":\"1\",\"method\":\"surface.read_text\",\"params\":{{\"surface_id\":{surface_id}}}}}\n"
        ),
    )
    .await;
    let denied = read_json(&mut reader).await;
    assert_eq!(denied["id"], "1");
    assert_eq!(
        denied["error"]["code"], -32011,
        "unclaimed bare read must be PermissionDenied; got {denied}"
    );

    write_request(
        &mut writer_half,
        &format!(
            "{{\"id\":\"2\",\"method\":\"surface.read_text\",\"params\":{{\"workspace_id\":{lane_b_id},\"surface_id\":{surface_id}}}}}\n"
        ),
    )
    .await;
    let allowed = read_json(&mut reader).await;
    assert_eq!(allowed["id"], "2");
    assert!(
        allowed["error"].is_null(),
        "natural first claim must succeed; got {allowed}"
    );

    write_request(
        &mut writer_half,
        &format!(
            "{{\"id\":\"3\",\"method\":\"surface.read_text\",\"params\":{{\"workspace_id\":{lane_a_id},\"surface_id\":{surface_id}}}}}\n"
        ),
    )
    .await;
    let sticky = read_json(&mut reader).await;
    assert_eq!(sticky["id"], "3");
    assert_eq!(
        sticky["error"]["code"], -32011,
        "foreign workspace after claim must be PermissionDenied; got {sticky}"
    );

    drop(writer_half);
    drop(reader);
    let _ = server_task.await;
}

#[tokio::test]
async fn off_mode_preserves_pre_patch_cross_workspace_read() {
    let (client, server) = UnixStream::pair().expect("unix stream pair");
    let dispatcher = Dispatcher::new();

    let lane_b = dispatcher
        .dispatch(limux_protocol::V2Request {
            id: Some(Value::String("off-b".into())),
            method: "workspace.create".into(),
            params: serde_json::json!({ "name": "off-lane-b" }),
        })
        .await;
    let lane_b_id = lane_b.result.expect("b")["workspace"]["id"]
        .as_u64()
        .expect("b id");
    let surface = dispatcher
        .dispatch(limux_protocol::V2Request {
            id: Some(Value::String("off-s".into())),
            method: "surface.current".into(),
            params: serde_json::json!({ "workspace_id": lane_b_id }),
        })
        .await;
    let surface_id = surface.result.expect("s")["surface"]["id"]
        .as_u64()
        .expect("s id");
    dispatcher
        .dispatch(limux_protocol::V2Request {
            id: Some(Value::String("off-t".into())),
            method: "surface.send_text".into(),
            params: serde_json::json!({
                "surface_id": surface_id,
                "text": "OFF-MODE-TEXT"
            }),
        })
        .await;

    let server_task = tokio::spawn(async move {
        handle_connection_with_entitlement_config(
            server,
            dispatcher,
            EntitlementConfig {
                mode: EntitlementMode::Off,
            },
        )
        .await
    });

    let (reader_half, mut writer_half) = client.into_split();
    let mut reader = BufReader::new(reader_half);

    write_request(
        &mut writer_half,
        &format!(
            "{{\"id\":\"1\",\"method\":\"surface.read_text\",\"params\":{{\"workspace_id\":{lane_b_id},\"surface_id\":{surface_id}}}}}\n"
        ),
    )
    .await;
    let response = read_json(&mut reader).await;
    assert_eq!(response["id"], "1");
    assert!(
        response["error"].is_null(),
        "Off mode must preserve pre-patch reads; got {response}"
    );

    drop(writer_half);
    drop(reader);
    let _ = server_task.await;
}
