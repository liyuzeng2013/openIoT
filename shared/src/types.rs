use serde::{Deserialize, Serialize};

/// 用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub created_at: String,
}

/// 设备信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: i64,
    pub user_id: i64,
    pub device_id: String,       // 设备唯一标识 (MAC 或自定义 ID)
    pub name: String,
    pub device_type: DeviceType,
    pub protocol: ProtocolType,
    pub token: String,           // 设备令牌
    pub online: bool,
    pub last_seen: String,
    pub created_at: String,
}

/// 设备类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceType {
    Light,
    Switch,
    Sensor,
    Plug,
    Thermostat,
    Unknown,
}

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceType::Light => write!(f, "灯"),
            DeviceType::Switch => write!(f, "开关"),
            DeviceType::Sensor => write!(f, "传感器"),
            DeviceType::Plug => write!(f, "插座"),
            DeviceType::Thermostat => write!(f, "温控器"),
            DeviceType::Unknown => write!(f, "未知"),
        }
    }
}

/// 协议类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProtocolType {
    BluetoothMesh1,    // 蓝牙 Mesh 1.0
    BluetoothMesh2,    // 蓝牙 Mesh 2.0
    Custom2G4,         // 自研 2.4G
}

impl std::fmt::Display for ProtocolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolType::BluetoothMesh1 => write!(f, "BLE Mesh 1.0"),
            ProtocolType::BluetoothMesh2 => write!(f, "BLE Mesh 2.0"),
            ProtocolType::Custom2G4 => write!(f, "自研 2.4G"),
        }
    }
}

/// 设备指令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCommand {
    pub device_id: String,
    pub command: String,
    pub params: serde_json::Value,
}

/// 设备状态上报
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStatus {
    pub device_id: String,
    pub state: serde_json::Value,
    pub timestamp: String,
}

/// API 响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T: Serialize> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self { code: 0, message: "ok".into(), data: Some(data) }
    }
    pub fn error(code: i32, msg: impl Into<String>) -> Self {
        Self { code, message: msg.into(), data: None }
    }
}

/// WebSocket 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WsMessage {
    /// 设备状态更新
    DeviceStatusUpdate(DeviceStatus),
    /// 发送设备指令
    DeviceCommand(DeviceCommand),
    /// 配网请求
    ProvisionRequest(ProvisionInfo),
    /// 配网结果
    ProvisionResult { success: bool, device_id: String, message: String },
    /// 心跳
    Ping,
    Pong,
}

/// 配网信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisionInfo {
    pub ssid: String,
    pub password: String,
    pub device_type: DeviceType,
    pub protocol: ProtocolType,
}

/// 登录请求
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

/// 注册请求
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}

/// Token 响应
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub token: String,
    pub username: String,
}
