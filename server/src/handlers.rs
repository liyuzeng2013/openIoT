use axum::{
    extract::{Path, State, WebSocketUpgrade},
    response::IntoResponse,
    Json,
};
use serde_json::json;
use shared::types::*;
use crate::AppState;

#[axum::debug_handler]
pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Json<ApiResponse<TokenResponse>> {
    if let Ok(Some(_)) = state.db.get_user_by_username(&req.username) {
        return Json(ApiResponse::<TokenResponse>::error(1, "Username already exists"));
    }

    let hash = match bcrypt::hash(&req.password, bcrypt::DEFAULT_COST) {
        Ok(h) => h,
        Err(_) => return Json(ApiResponse::<TokenResponse>::error(2, "Password hash failed")),
    };

    match state.db.create_user(&req.username, &hash) {
        Ok(user_id) => {
            let token = crate::auth::create_token(user_id, &req.username, &state.jwt_secret)
                .unwrap_or_default();
            Json(ApiResponse::<TokenResponse>::success(TokenResponse {
                token,
                username: req.username,
            }))
        }
        Err(_) => Json(ApiResponse::<TokenResponse>::error(3, "Registration failed")),
    }
}

#[axum::debug_handler]
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Json<ApiResponse<TokenResponse>> {
    let user = match state.db.get_user_by_username(&req.username) {
        Ok(Some(u)) => u,
        _ => return Json(ApiResponse::<TokenResponse>::error(1, "Invalid username or password")),
    };

    if !bcrypt::verify(&req.password, &user.password_hash).unwrap_or(false) {
        return Json(ApiResponse::<TokenResponse>::error(1, "Invalid username or password"));
    }

    let token = crate::auth::create_token(user.id, &user.username, &state.jwt_secret)
        .unwrap_or_default();

    Json(ApiResponse::<TokenResponse>::success(TokenResponse {
        token,
        username: user.username,
    }))
}

#[axum::debug_handler]
pub async fn list_devices(
    State(state): State<AppState>,
) -> Json<ApiResponse<Vec<Device>>> {
    let user_id = 1;
    match state.db.get_user_devices(user_id) {
        Ok(devices) => Json(ApiResponse::<Vec<Device>>::success(devices)),
        Err(_) => Json(ApiResponse::<Vec<Device>>::error(1, "Failed to get devices")),
    }
}

#[axum::debug_handler]
pub async fn get_device(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<ApiResponse<Device>> {
    match state.db.get_device_by_id(&id) {
        Ok(Some(device)) => Json(ApiResponse::<Device>::success(device)),
        Ok(None) => Json(ApiResponse::<Device>::error(1, "Device not found")),
        Err(_) => Json(ApiResponse::<Device>::error(2, "Query failed")),
    }
}

#[axum::debug_handler]
pub async fn add_device(
    State(state): State<AppState>,
    Json(device): Json<Device>,
) -> Json<ApiResponse<String>> {
    match state.db.create_device(&device) {
        Ok(_) => Json(ApiResponse::<String>::success("Device added".to_string())),
        Err(_) => Json(ApiResponse::<String>::error(1, "Add failed".to_string())),
    }
}

#[axum::debug_handler]
pub async fn delete_device(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Json<ApiResponse<String>> {
    match state.db.delete_device(&id) {
        Ok(_) => Json(ApiResponse::<String>::success("Device deleted".to_string())),
        Err(_) => Json(ApiResponse::<String>::error(1, "Delete failed".to_string())),
    }
}

#[axum::debug_handler]
pub async fn send_command(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(cmd): Json<DeviceCommand>,
) -> Json<ApiResponse<String>> {
    tracing::info!("Sending command to device {}: {:?}", id, cmd);
    let cmd_msg = json!({
        "type": "command",
        "device_id": id,
        "command": cmd.command,
        "params": cmd.params
    });
    let _ = state.broadcast_tx.send(cmd_msg.to_string());
    Json(ApiResponse::<String>::success("Command sent".to_string()))
}

#[axum::debug_handler]
pub async fn provision_device(
    State(_state): State<AppState>,
    Json(_info): Json<ProvisionInfo>,
) -> Json<ApiResponse<serde_json::Value>> {
    let device_id = format!("esp32-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap());
    let token = uuid::Uuid::new_v4().to_string();

    Json(ApiResponse::<serde_json::Value>::success(json!({
        "device_id": device_id,
        "token": token,
        "message": "Provisioning info sent"
    })))
}

#[axum::debug_handler]
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

async fn handle_ws(
    socket: axum::extract::ws::WebSocket,
    state: AppState,
) {
    use axum::extract::ws::Message;
    use futures_util::{SinkExt, StreamExt};
    use serde_json::Value;

    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.broadcast_tx.subscribe();

    let (client_tx, mut client_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    // Task 1: Forward from broadcast channel to this WS client
    let tx_clone = client_tx.clone();
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if tx_clone.send(msg).is_err() {
                break;
            }
        }
    });

    // Task 2: Receive from WS client, parse, broadcast, process
    let process_tx = state.broadcast_tx.clone();
    let process_db = state.db.clone();
    tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    // Try WsMessage first
                    if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                        match ws_msg {
                            WsMessage::Ping => {
                                let _ = client_tx.send(
                                    serde_json::to_string(&WsMessage::Pong).unwrap()
                                );
                            }
                            WsMessage::DeviceStatusUpdate(status) => {
                                let _ = process_db.save_device_state(&status.device_id, &status.state);
                                let _ = process_db.update_device_online(&status.device_id, true);
                                // Broadcast to all
                                let outgoing = json!({
                                    "type": "status",
                                    "device_id": status.device_id,
                                    "state": status.state
                                });
                                let _ = process_tx.send(outgoing.to_string());
                            }
                            WsMessage::DeviceCommand(_cmd) => {
                                let _ = process_tx.send(text.clone());
                            }
                            _ => {}
                        }
                    } else if let Ok(val) = serde_json::from_str::<Value>(&text) {
                        // Fallback: accept simple format {type: "status", device_id: "...", state: {...}}
                        if let Some(msg_type) = val.get("type").and_then(|v| v.as_str()) {
                            if msg_type == "status" {
                                if let (Some(device_id), Some(state_val)) = (
                                    val.get("device_id").and_then(|v| v.as_str()),
                                    val.get("state")
                                ) {
                                    let _ = process_db.save_device_state(device_id, state_val);
                                    let _ = process_db.update_device_online(device_id, true);
                                    let _ = process_tx.send(text.clone());
                                }
                            } else {
                                // Just forward
                                let _ = process_tx.send(text.clone());
                            }
                        }
                    }
                }
                Ok(Message::Close(_)) => break,
                Err(_) => break,
                _ => {}
            }
        }
        // Signal task 3 to stop
        drop(client_tx);
    });

    // Task 3: Forward queued messages to WS
    while let Some(msg) = client_rx.recv().await {
        if sender.send(Message::Text(msg)).await.is_err() {
            break;
        }
    }
}
