# OpenIoT

> 类似米家的开源 IoT 平台，自研 2.4G + 蓝牙 Mesh 协议，支持 ESP32 设备接入

**版本：alpha2612** · 2026年 Alpha 测试分支，当前仅 Bug 修复迭代

## 项目架构

```
openIoT/
├── shared/            # 共享代码库：协议定义、数据结构
│   └── src/
│       ├── types.rs   # Device / ApiResponse / WsMessage
│       └── protocol.rs # 2.4G & BLE Mesh 协议帧
├── server/            # Rust Web 服务端
│   ├── src/
│   │   ├── main.rs    # Axum 路由 + 广播通道
│   │   ├── db.rs      # SQLite 持久化
│   │   ├── handlers.rs # API & WebSocket 处理
│   │   └── auth.rs    # JWT 认证中间件
│   └── static/        # 前端页面 (HTML/CSS/JS)
├── client/            # Windows 桌面客户端 (egui)
│   └── src/
│       ├── main.rs
│       ├── api.rs     # 服务端 HTTP 调用
│       └── views/     # 登录页 / 控制台
├── esp32/             # MicroPython ESP32 固件
│   ├── main.py        # 主循环
│   ├── config.py      # NVS 非易失配置
│   ├── provision.py   # WiFi 配网 (热点模式)
│   ├── protocol.py    # 协议处理
│   ├── mesh.py        # 蓝牙 Mesh 组网
│   └── device.py      # 设备控制逻辑
├── virtual_device.py  # Python 虚拟设备 (调试/演示用)
├── Makefile
├── build.bat          # Windows 一键构建脚本
└── Cargo.toml         # Workspace 定义
```

## 技术栈

| 层级 | 技术 |
|---|---|
| 服务端 | Rust + Axum + SQLite (rusqlite) + WebSocket + JWT |
| Web 前端 | 原生 HTML / CSS / JavaScript (无需构建) |
| 桌面端 | Rust + egui + reqwest |
| 硬件端 | MicroPython (ESP32 / ESP32-C3) |
| 通信协议 | 自研 2.4G、蓝牙 Mesh 1.0/2.0、WiFi WebSocket |

## 版本命名规则

格式：`[版本类型][年份后两位][分级编号][更新类型]`

| 段落 | 取值 | 含义 |
|---|---|---|
| 版本类型 | `alpha` / `beta` / `stable` | 内部测试 / 公开测试 / 正式发行 |
| 年份 | `26` `27` ... | 年份后两位 |
| 分级编号 | `1` | Alpha 测试 |
| | `2` | Patch 补丁 (仅修复bug) |
| | `3` | Beta 测试 |
| | `4` | 普通 Stable |
| | `5` | LTS 长期支持 |
| 更新类型 | `1` | 新功能迭代 |
| | `2` | 仅 Bug 修复 |

示例：`stable26h52` → stable / 2026年 / LTS / 纯修复

本分支当前版本：**alpha2612**

## 快速开始

### 前置条件

- **Rust** 工具链 (1.70+)
- **Python** 3.10+ (仅虚拟设备需要)

### 1. 编译与启动服务端

**Windows**（推荐）：
```
build.bat build
build.bat run-server
```

**Linux / macOS** 或有 Make：
```
make build
make run-server
```

**手动 Cargo**：
```
cargo build --workspace
cargo run -p openiot-server
```

服务启动后访问：http://localhost:3000

### 2. 启动虚拟设备（可选）

在新终端运行：
```
python virtual_device.py
```
或：
```
run_device.bat
```

虚拟设备会自动注册为「客厅空调」并每 5 秒上报温湿度/亮度，可通过 Web 页面实时查看。

### 3. 启动桌面客户端（可选）

```
cargo run -p openiot-client
```

## 功能特性

- 用户注册 / 登录，JWT 鉴权
- 多设备管理（增删查改，按用户隔离）
- WebSocket 实时通信：设备状态上报、服务器指令下发
- 广播通道：一条消息自动推送给所有在线客户端
- 虚拟设备模拟器：无需硬件即可演示完整链路
- 自研协议帧：`Header(4B) + Flags(1B) + OpCode(1B) + Payload + CRC16(2B) + Footer(2B)`
- 蓝牙 Mesh 1.0 / 2.0 兼容
- SQLite 本地持久化（`openiot.db`）

## API 概览

| Method | Path | 功能 |
|---|---|---|
| POST | `/api/register` | 注册用户 |
| POST | `/api/login` | 登录（返回 JWT） |
| GET | `/api/devices` | 获取设备列表 |
| POST | `/api/devices` | 新增设备 |
| GET | `/api/devices/:id` | 获取单个设备详情 |
| DELETE | `/api/devices/:id` | 删除设备 |
| POST | `/api/devices/:id/command` | 向设备下发指令 |
| POST | `/api/provision` | 设备配网请求 |
| GET | `/ws` | WebSocket 连接点（实时通信） |

### WebSocket 消息格式

设备上报：
```json
{
  "type": "status",
  "device_id": "vdev-xxx",
  "state": {
    "power": true,
    "temperature": 26.5,
    "humidity": 55.2,
    "light": 70
  }
}
```

指令下发：
```json
{
  "type": "command",
  "device_id": "vdev-xxx",
  "command": "brightness",
  "params": { "value": 80 }
}
```

## 目录说明

```
openIoT/
├── server/static/       前端资源 (index.html / style.css / app.js)
├── esp32/               刷入 ESP32 的 MicroPython 代码
│   └── 配网流程：上电 → 热点 openIoT-Setup → 手机连接 → 访问 192.168.4.1 → 填 WiFi
├── virtual_device.py    Python 版设备模拟器
└── openiot.db           运行后生成的 SQLite 数据库（服务端目录）
```

## 编译故障排查

- **`futures_util` 未解析**：确认 `server/Cargo.toml` 中依赖存在，执行 `cargo update`
- **`debug_handler` 报错**：确认 `axum` 已启用 `macros` feature：`axum = { version = "0.7", features = ["ws", "multipart", "macros"] }`
- **bat 脚本报乱码**：确认文件为 UTF-8 编码，脚本中已使用 `chcp 65001`

## 开发路线图

- [x] 基础用户系统 & JWT
- [x] 设备 CRUD
- [x] WebSocket 实时通信
- [x] 虚拟设备模拟器
- [x] Web 前端控制台
- [x] egui 桌面客户端框架
- [x] ESP32 MicroPython 固件 (配网/协议/蓝牙 Mesh)
- [ ] 真实 2.4G 射频模块驱动
- [ ] 固件 OTA 升级
- [ ] 场景联动 / 自动化规则
- [ ] 移动端 App

## License

MIT
