"""
蓝牙 Mesh 网络模块
支持 Mesh 1.0 和 Mesh 2.0 协议
"""

import json
import time
import struct

class MeshNetwork:
    """蓝牙 Mesh 网络管理"""
    
    def __init__(self):
        self.net_key = None
        self.app_key = None
        self.device_addr = 0
        self.iv_index = 0
        self.seq_num = 0
        self.neighbors = []  # 邻居节点列表
        self.routing_table = {}  # 路由表
        
    def init(self, config):
        """初始化 Mesh 网络"""
        self.net_key = config['net_key']
        self.app_key = config['app_key']
        self.device_addr = config['device_addr']
        
        print(f"[Mesh] 初始化完成，设备地址: 0x{self.device_addr:04X}")
        
    def send_mesh_message(self, dst_addr, opcode, payload):
        """发送 Mesh 消息"""
        self.seq_num = (self.seq_num + 1) & 0xFFFFFF
        
        # 构建 Mesh PDU
        pdu = self._build_mesh_pdu(dst_addr, opcode, payload)
        
        # 加密
        encrypted_pdu = self._encrypt_pdu(pdu)
        
        # 通过蓝牙发送
        self._bluetooth_send(encrypted_pdu)
        
    def _build_mesh_pdu(self, dst_addr, opcode, payload):
        """构建 Mesh PDU"""
        pdu = bytearray()
        
        # 控制字节
        ctl = 0x00  # 访问消息
        pdu.append(ctl)
        
        # 源地址
        pdu.extend(struct.pack('>H', self.device_addr))
        
        # 目标地址
        pdu.extend(struct.pack('>H', dst_addr))
        
        # IV Index
        pdu.extend(struct.pack('>I', self.iv_index))
        
        # 序列号
        pdu.extend(struct.pack('>I', self.seq_num)[1:4])  # 3字节
        
        # OpCode
        pdu.append(opcode)
        
        # 负载
        pdu.extend(payload)
        
        return pdu
        
    def _encrypt_pdu(self, pdu):
        """加密 PDU（简化版）"""
        # TODO: 实现真正的 AES-CCM 加密
        # 这里只是示例，实际需要使用 ubluecrypto 库
        encrypted = bytearray()
        for i, byte in enumerate(pdu):
            key_byte = self.net_key[i % len(self.net_key)]
            encrypted.append(byte ^ key_byte)
        return encrypted
        
    def _bluetooth_send(self, data):
        """通过蓝牙发送数据"""
        # TODO: 实现蓝牙广播或 GATT 发送
        print(f"[Mesh] 发送数据: {len(data)} 字节")
        
    def receive_mesh_message(self, data):
        """接收 Mesh 消息"""
        # 解密
        decrypted = self._decrypt_pdu(data)
        
        # 解析 PDU
        if len(decrypted) < 11:
            return
            
        ctl = decrypted[0]
        src_addr = struct.unpack('>H', decrypted[1:3])[0]
        dst_addr = struct.unpack('>H', decrypted[3:5])[0]
        iv_index = struct.unpack('>I', decrypted[5:9])[0]
        seq = struct.unpack('>I', b'\x00' + decrypted[9:12])[0]
        opcode = decrypted[12]
        payload = decrypted[13:]
        
        # 检查是否发给自己的
        if dst_addr == self.device_addr or dst_addr == 0xFFFF:
            self._handle_mesh_message(src_addr, opcode, payload)
            
        # 如果是中继消息，转发
        if dst_addr != self.device_addr and dst_addr != 0xFFFF:
            self._relay_message(data)
            
    def _decrypt_pdu(self, data):
        """解密 PDU"""
        # TODO: 实现真正的 AES-CCM 解密
        decrypted = bytearray()
        for i, byte in enumerate(data):
            key_byte = self.net_key[i % len(self.net_key)]
            decrypted.append(byte ^ key_byte)
        return decrypted
        
    def _handle_mesh_message(self, src_addr, opcode, payload):
        """处理 Mesh 消息"""
        print(f"[Mesh] 收到消息来自 0x{src_addr:04X}, OpCode: 0x{opcode:02X}")
        
        # 更新邻居列表
        if src_addr not in self.neighbors:
            self.neighbors.append(src_addr)
            print(f"[Mesh] 发现新邻居: 0x{src_addr:04X}")
            
        # 更新路由表
        self.routing_table[src_addr] = src_addr  # 直接邻居
        
    def _relay_message(self, data):
        """中继消息"""
        # TTL 减 1
        # TODO: 实现 TTL 处理
        self._bluetooth_send(data)
        
    def join_mesh(self):
        """加入 Mesh 网络"""
        # 发送 Join 请求
        self.send_mesh_message(0xFFFF, 0x30, b"")
        print("[Mesh] 发送加入网络请求")
        
    def leave_mesh(self):
        """离开 Mesh 网络"""
        self.send_mesh_message(0xFFFF, 0x31, b"")
        print("[Mesh] 离开网络")
        
    def get_neighbors(self):
        """获取邻居列表"""
        return self.neighbors
        
    def get_routing_table(self):
        """获取路由表"""
        return self.routing_table
