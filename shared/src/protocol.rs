/// 自研 2.4G / 蓝牙 Mesh 通信协议定义
///
/// 帧格式:
/// ┌────────┬────────┬────────┬─────────┬────────────┬─────────┐
/// │ Header │ Flags  │ OpCode │ Payload │   CRC16    │  Footer │
/// │ 1 byte │ 1 byte │ 1 byte │ N bytes │  2 bytes   │ 1 byte  │
/// └────────┴────────┴────────┴─────────┴────────────┴─────────┘
///
/// Header: 0xAA (帧起始)
/// Flags:  bit0=encrypted, bit1=ack_required, bit2=mesh_relay
/// OpCode: 操作码
/// Payload: 变长负载
/// CRC16: CRC-16/CCITT 校验
/// Footer: 0x55 (帧结束)

pub const FRAME_HEADER: u8 = 0xAA;
pub const FRAME_FOOTER: u8 = 0x55;

// Flags
pub const FLAG_ENCRYPTED: u8 = 0x01;
pub const FLAG_ACK_REQUIRED: u8 = 0x02;
pub const FLAG_MESH_RELAY: u8 = 0x04;

// OpCode 定义
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OpCode {
    // --- 基础控制 ---
    Ack             = 0x00,
    Heartbeat       = 0x01,
    Discover        = 0x02,
    DiscoverResp    = 0x03,

    // --- 配网 ---
    ProvisionReq    = 0x10,
    ProvisionResp   = 0x11,
    ProvisionAck    = 0x12,

    // --- 设备控制 ---
    SetState        = 0x20,
    GetState        = 0x21,
    StateReport     = 0x22,

    // --- Mesh 网络 ---
    MeshJoin        = 0x30,
    MeshLeave       = 0x31,
    MeshRelay       = 0x32,
    MeshRoute       = 0x33,

    // --- 安全 ---
    KeyExchange     = 0x40,
    KeyExchangeAck  = 0x41,
}

impl OpCode {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x00 => Some(Self::Ack),
            0x01 => Some(Self::Heartbeat),
            0x02 => Some(Self::Discover),
            0x03 => Some(Self::DiscoverResp),
            0x10 => Some(Self::ProvisionReq),
            0x11 => Some(Self::ProvisionResp),
            0x12 => Some(Self::ProvisionAck),
            0x20 => Some(Self::SetState),
            0x21 => Some(Self::GetState),
            0x22 => Some(Self::StateReport),
            0x30 => Some(Self::MeshJoin),
            0x31 => Some(Self::MeshLeave),
            0x32 => Some(Self::MeshRelay),
            0x33 => Some(Self::MeshRoute),
            0x40 => Some(Self::KeyExchange),
            0x41 => Some(Self::KeyExchangeAck),
            _ => None,
        }
    }
}

/// 协议帧
#[derive(Debug, Clone)]
pub struct Frame {
    pub flags: u8,
    pub opcode: OpCode,
    pub src_addr: u16,
    pub dst_addr: u16,
    pub seq: u16,
    pub payload: Vec<u8>,
}

impl Frame {
    pub fn new(opcode: OpCode, src: u16, dst: u16, payload: Vec<u8>) -> Self {
        Self {
            flags: 0,
            opcode,
            src_addr: src,
            dst_addr: dst,
            seq: 0,
            payload,
        }
    }

    /// 编码为字节流
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(10 + self.payload.len());
        buf.push(FRAME_HEADER);
        buf.push(self.flags);
        buf.push(self.opcode as u8);
        buf.extend_from_slice(&self.src_addr.to_be_bytes());
        buf.extend_from_slice(&self.dst_addr.to_be_bytes());
        buf.extend_from_slice(&self.seq.to_be_bytes());
        buf.extend_from_slice(&self.payload);
        let crc = crc16(&buf);
        buf.extend_from_slice(&crc.to_be_bytes());
        buf.push(FRAME_FOOTER);
        buf
    }

    /// 从字节流解码
    pub fn decode(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() < 11 {
            return Err("frame too short");
        }
        if data[0] != FRAME_HEADER {
            return Err("invalid header");
        }
        if data[data.len() - 1] != FRAME_FOOTER {
            return Err("invalid footer");
        }
        let payload_end = data.len() - 3; // 去掉 CRC(2) + Footer(1)
        let flags = data[1];
        let opcode = OpCode::from_byte(data[2]).ok_or("unknown opcode")?;
        let src_addr = u16::from_be_bytes([data[3], data[4]]);
        let dst_addr = u16::from_be_bytes([data[5], data[6]]);
        let seq = u16::from_be_bytes([data[7], data[8]]);
        let payload = data[9..payload_end].to_vec();

        // CRC 校验
        let expected_crc = u16::from_be_bytes([data[payload_end], data[payload_end + 1]]);
        let actual_crc = crc16(&data[..payload_end]);
        if expected_crc != actual_crc {
            return Err("crc mismatch");
        }

        Ok(Self { flags, opcode, src_addr, dst_addr, seq, payload })
    }

    pub fn with_ack_required(mut self) -> Self {
        self.flags |= FLAG_ACK_REQUIRED;
        self
    }

    pub fn with_encrypted(mut self) -> Self {
        self.flags |= FLAG_ENCRYPTED;
        self
    }

    pub fn with_mesh_relay(mut self) -> Self {
        self.flags |= FLAG_MESH_RELAY;
        self
    }
}

/// CRC-16/CCITT
pub fn crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            if crc & 0x8000 != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

/// 蓝牙 Mesh 网络层地址
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeshAddress(pub u16);

impl MeshAddress {
    pub const BROADCAST: Self = Self(0xFFFF);
    pub const UNASSIGNED: Self = Self(0x0000);

    pub fn is_broadcast(&self) -> bool {
        self.0 == 0xFFFF
    }
}

/// Mesh 网络配置
#[derive(Debug, Clone)]
pub struct MeshConfig {
    pub net_key: [u8; 16],
    pub app_key: [u8; 16],
    pub net_index: u16,
    pub iv_index: u32,
    pub ttl: u8,
}

impl Default for MeshConfig {
    fn default() -> Self {
        Self {
            net_key: [0u8; 16],
            app_key: [0u8; 16],
            net_index: 0,
            iv_index: 0,
            ttl: 7,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_encode_decode() {
        let frame = Frame::new(OpCode::Heartbeat, 0x0001, 0x0002, vec![0x01, 0x02, 0x03]);
        let encoded = frame.encode();
        let decoded = Frame::decode(&encoded).unwrap();
        assert_eq!(decoded.opcode, OpCode::Heartbeat);
        assert_eq!(decoded.src_addr, 0x0001);
        assert_eq!(decoded.dst_addr, 0x0002);
        assert_eq!(decoded.payload, vec![0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_crc16() {
        let data = b"123456789";
        let crc = crc16(data);
        assert_eq!(crc, 0x29B1);
    }
}
