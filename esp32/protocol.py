"""
协议处理模块
实现自研 2.4G 和蓝牙 Mesh 协议
"""

import socket
import json
import struct
import time

# 协议常量
FRAME_HEADER = 0xAA
FRAME_FOOTER = 0x55

# OpCode
OPCODE_ACK = 0x00
OPCODE_HEARTBEAT = 0x01
OPCODE_DISCOVER = 0x02
OPCODE_DISCOVER_RESP = 0x03
OPCODE_PROVISION_REQ = 0x10
OPCODE_PROVISION_RESP = 0x11
OPCODE_SET_STATE = 0x20
OPCODE_GET_STATE = 0x21
OPCODE_STATE_REPORT = 0x22
OPCODE_MESH_JOIN = 0x30
OPCODE_MESH_LEAVE = 0x31

# Flags
FLAG_ENCRYPTED = 0x01
FLAG_ACK_REQUIRED = 0x02
FLAG_MESH_RELAY = 0x04

class ProtocolHandler:
    """协议处理器"""
    
    def __init__(self):
        self.server_socket = None
        self.server_config = None
        self.command_callback = None
        self.seq_num = 0
        self.running = False
        
    def start(self, server_config, callback):
        """启动协议处理器"""
        self.server_config = server_config
        self.command_callback = callback
        self.running = True
        
        # 创建 TCP 连接到服务器
        self._connect_to_server()
        
        # 启动接收线程
        import _thread
        _thread.start_new_thread(self._receive_loop, ())
        
    def _connect_to_server(self):
        """连接到服务器"""
        try:
            self.server_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            self.server_socket.connect((
                self.server_config['server_ip'],
                self.server_config['server_port']
            ))
            print(f"[Protocol] 已连接到服务器 {self.server_config['server_ip']}:{self.server_config['server_port']}")
            
            # 发送设备注册
            self._send_register()
            
        except Exception as e:
            print(f"[Protocol] 连接服务器失败: {e}")
            
    def _send_register(self):
        """发送设备注册"""
        register_msg = {
            'type': 'register',
            'device_id': self.server_config['device_id'],
            'token': self.server_config['device_token']
        }
        
        frame = self._encode_frame(
            opcode=OPCODE_DISCOVER,
            payload=json.dumps(register_msg).encode()
        )
        
        self.server_socket.send(frame)
        
    def _receive_loop(self):
        """接收消息循环"""
        buffer = b""
        
        while self.running:
            try:
                data = self.server_socket.recv(1024)
                if not data:
                    break
                    
                buffer += data
                
                # 解析帧
                while len(buffer) >= 11:
                    # 查找帧头
                    header_pos = buffer.find(bytes([FRAME_HEADER]))
                    if header_pos == -1:
                        buffer = b""
                        break
                        
                    if header_pos > 0:
                        buffer = buffer[header_pos:]
                        
                    # 检查是否有完整帧
                    footer_pos = buffer.find(bytes([FRAME_FOOTER]))
                    if footer_pos == -1:
                        break
                        
                    # 提取帧
                    frame_data = buffer[:footer_pos + 1]
                    buffer = buffer[footer_pos + 1:]
                    
                    # 解码帧
                    frame = self._decode_frame(frame_data)
                    if frame:
                        self._handle_frame(frame)
                        
            except Exception as e:
                print(f"[Protocol] 接收错误: {e}")
                break
                
        print("[Protocol] 接收线程退出")
        
    def _handle_frame(self, frame):
        """处理接收到的帧"""
        opcode = frame['opcode']
        payload = frame['payload']
        
        if opcode == OPCODE_HEARTBEAT:
            # 心跳响应
            pass
        elif opcode == OPCODE_SET_STATE:
            # 设置状态指令
            command = json.loads(payload.decode())
            if self.command_callback:
                self.command_callback(command)
        elif opcode == OPCODE_GET_STATE:
            # 获取状态请求
            pass
        elif opcode == OPCODE_ACK:
            # ACK 确认
            pass
            
    def _encode_frame(self, opcode, payload, flags=0):
        """编码协议帧"""
        self.seq_num = (self.seq_num + 1) & 0xFFFF
        
        # 构建帧数据
        frame = bytearray()
        frame.append(FRAME_HEADER)
        frame.append(flags)
        frame.append(opcode)
        
        # 源地址（设备地址）
        src_addr = 0x0001  # 临时固定
        frame.extend(struct.pack('>H', src_addr))
        
        # 目标地址（服务器地址）
        dst_addr = 0x0000  # 服务器
        frame.extend(struct.pack('>H', dst_addr))
        
        # 序列号
        frame.extend(struct.pack('>H', self.seq_num))
        
        # 负载
        frame.extend(payload)
        
        # CRC16
        crc = self._crc16(frame)
        frame.extend(struct.pack('>H', crc))
        
        # 帧尾
        frame.append(FRAME_FOOTER)
        
        return bytes(frame)
        
    def _decode_frame(self, data):
        """解码协议帧"""
        if len(data) < 11:
            return None
            
        if data[0] != FRAME_HEADER or data[-1] != FRAME_FOOTER:
            return None
            
        # 验证 CRC
        payload_end = len(data) - 3
        expected_crc = struct.unpack('>H', data[payload_end:payload_end + 2])[0]
        actual_crc = self._crc16(data[:payload_end])
        
        if expected_crc != actual_crc:
            print("[Protocol] CRC 校验失败")
            return None
            
        # 解析帧
        flags = data[1]
        opcode = data[2]
        src_addr = struct.unpack('>H', data[3:5])[0]
        dst_addr = struct.unpack('>H', data[5:7])[0]
        seq = struct.unpack('>H', data[7:9])[0]
        payload = data[9:payload_end]
        
        return {
            'flags': flags,
            'opcode': opcode,
            'src_addr': src_addr,
            'dst_addr': dst_addr,
            'seq': seq,
            'payload': payload
        }
        
    def _crc16(self, data):
        """CRC-16/CCITT 计算"""
        crc = 0xFFFF
        for byte in data:
            crc ^= byte << 8
            for _ in range(8):
                if crc & 0x8000:
                    crc = (crc << 1) ^ 0x1021
                else:
                    crc <<= 1
                crc &= 0xFFFF
        return crc
        
    def send_heartbeat(self):
        """发送心跳"""
        if self.server_socket:
            frame = self._encode_frame(OPCODE_HEARTBEAT, b"")
            try:
                self.server_socket.send(frame)
            except:
                pass
                
    def send_status(self, status_msg):
        """发送状态上报"""
        if self.server_socket:
            payload = json.dumps(status_msg).encode()
            frame = self._encode_frame(OPCODE_STATE_REPORT, payload)
            try:
                self.server_socket.send(frame)
            except:
                pass
                
    def stop(self):
        """停止协议处理器"""
        self.running = False
        if self.server_socket:
            self.server_socket.close()
