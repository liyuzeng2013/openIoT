#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
OpenIoT 虚拟设备模拟器
模拟 ESP32 设备连接到服务端，上报状态，接收指令
"""

import json
import time
import random
import threading
import uuid
import requests
import websocket
from datetime import datetime

SERVER_URL = "http://localhost:3000"
WS_URL = "ws://localhost:3000/ws"


class VirtualDevice:
    def __init__(self, device_id=None, name="客厅空调"):
        self.device_id = device_id or f"vdev-{uuid.uuid4().hex[:8]}"
        self.name = name
        self.online = True
        self.token = None
        self.user_id = 1
        
        # 模拟设备状态
        self.state = {
            "power": False,
            "temperature": 26.0,      # 当前温度
            "target_temp": 25.0,      # 目标温度
            "humidity": 55.0,
            "light": 70,              # 亮度 0-100
            "fan_speed": "auto",
        }
        
        # WebSocket 连接
        self.ws = None
        self.running = False
        self.heartbeat_thread = None
    
    # ---------- HTTP API 部分 ----------
    
    def register(self):
        """注册到服务端"""
        device = {
            "id": 0,
            "user_id": self.user_id,
            "device_id": self.device_id,
            "name": self.name,
            "device_type": "Sensor",
            "protocol": "Custom2G4",
            "token": self.device_id,
            "online": True,
            "last_seen": datetime.now().isoformat(),
            "created_at": datetime.now().isoformat()
        }
        
        try:
            resp = requests.post(f"{SERVER_URL}/api/devices", json=device)
            data = resp.json()
            if data.get("code") == 0:
                print(f"[√] 设备注册成功: {self.device_id}")
                return True
            print(f"[!] 注册返回: {data}")
            return False
        except Exception as e:
            print(f"[!] 注册失败: {e}")
            return False
    
    def list_devices(self):
        """查看服务器上的设备列表"""
        try:
            resp = requests.get(f"{SERVER_URL}/api/devices")
            data = resp.json()
            if data.get("code") == 0 and data.get("data"):
                print(f"\n=== 服务器设备列表 ===")
                for dev in data["data"]:
                    status = "在线" if dev.get("online") else "离线"
                    print(f"  [{status}] {dev['name']} ({dev['device_id']}) - {dev['device_type']}")
                print()
        except Exception as e:
            print(f"[!] 获取设备列表失败: {e}")
    
    # ---------- WebSocket 部分 ----------
    
    def on_message(self, ws, message):
        """收到消息"""
        try:
            msg = json.loads(message)
            msg_type = msg.get("type", "")
            print(f"\n[收到] {msg}")

            if msg_type == "command" and msg.get("device_id") == self.device_id:
                cmd = msg.get("command", "")
                params = msg.get("params", {})
                self.handle_command(cmd, params)
            elif msg_type == "status":
                # 其他设备状态,忽略
                pass
        except json.JSONDecodeError:
            print(f"[收到非JSON] {message}")
    
    def on_error(self, ws, error):
        print(f"[WebSocket 错误] {error}")
    
    def on_open(self, ws):
        print("[√] WebSocket 连接已建立")
        self.ws = ws
        # 连接成功后立即上报一次状态
        self.report_status()
    
    def on_close(self, ws, close_code, reason):
        print(f"[WebSocket 断开] code={close_code}, reason={reason}")
    
    def connect_websocket(self):
        """连接到 WebSocket 服务器"""
        self.running = True
        
        while self.running:
            try:
                self.ws = websocket.WebSocketApp(
                    WS_URL,
                    on_message=self.on_message,
                    on_error=self.on_error,
                    on_open=self.on_open,
                    on_close=self.on_close
                )
                self.ws.run_forever(ping_interval=30, ping_timeout=10)
            except Exception as e:
                print(f"[WebSocket 异常] {e}")
            
            if self.running:
                print("[重连] 5秒后重新连接...")
                time.sleep(5)
    
    def start(self):
        """启动虚拟设备"""
        print("=" * 50)
        print(f"  OpenIoT 虚拟设备模拟器")
        print(f"  设备ID: {self.device_id}")
        print(f"  设备名称: {self.name}")
        print(f"  服务器: {SERVER_URL}")
        print("=" * 50)
        
        # 注册设备
        self.register()
        time.sleep(0.5)
        
        # 查看设备列表
        self.list_devices()
        
        # 启动 WebSocket (后台线程)
        ws_thread = threading.Thread(target=self.connect_websocket, daemon=True)
        ws_thread.start()
        
        # 启动心跳/状态上报线程
        self.heartbeat_thread = threading.Thread(target=self._report_loop, daemon=True)
        self.heartbeat_thread.start()
        
        # 启动模拟数据变化线程
        sim_thread = threading.Thread(target=self._simulate_environment, daemon=True)
        sim_thread.start()
        
        # 主循环 - 控制台交互
        try:
            while True:
                cmd = input("\n输入指令 (status/on/off/temp/exit, 默认空): ").strip().lower()
                
                if cmd in ["exit", "quit", "q"]:
                    self.stop()
                    break
                elif cmd in ["status", "s"]:
                    self.print_status()
                elif cmd in ["on"]:
                    self.state["power"] = True
                    self.report_status()
                    print("[控制] 设备已开启")
                elif cmd in ["off"]:
                    self.state["power"] = False
                    self.report_status()
                    print("[控制] 设备已关闭")
                elif cmd.startswith("temp"):
                    try:
                        parts = cmd.split()
                        t = float(parts[1])
                        self.state["target_temp"] = t
                        self.report_status()
                        print(f"[控制] 目标温度设置为 {t}°C")
                    except:
                        print("用法: temp 25")
                elif cmd.startswith("light"):
                    try:
                        parts = cmd.split()
                        l = int(parts[1])
                        self.state["light"] = max(0, min(100, l))
                        self.report_status()
                        print(f"[控制] 亮度设置为 {self.state['light']}%")
                    except:
                        print("用法: light 80")
                elif cmd == "":
                    continue
                else:
                    print("未知指令, 支持: status, on, off, temp XX, light XX, exit")
        except KeyboardInterrupt:
            print("\n\n[用户中断] 正在退出...")
            self.stop()
    
    def stop(self):
        """停止虚拟设备"""
        self.running = False
        if self.ws:
            self.ws.close()
        print("[退出] 虚拟设备已停止")
    
    # ---------- 状态上报 ----------
    
    def report_status(self):
        """上报设备状态到服务器"""
        if self.ws and self.ws.sock and self.ws.sock.connected:
            msg = {
                "type": "status",
                "device_id": self.device_id,
                "state": self.state,
                "timestamp": datetime.now().isoformat()
            }
            try:
                self.ws.send(json.dumps(msg, ensure_ascii=False))
            except Exception as e:
                pass
    
    def handle_command(self, command, params):
        """处理来自服务器的指令"""
        print(f"\n[控制指令] 指令={command}, 参数={params}")
        
        if command == "power":
            self.state["power"] = params.get("on", False)
            self.report_status()
        elif command == "brightness":
            self.state["light"] = params.get("value", self.state["light"])
            self.report_status()
        elif command == "set_temp":
            self.state["target_temp"] = params.get("value", self.state["target_temp"])
            self.report_status()
    
    def print_status(self):
        """打印当前状态"""
        print("\n" + "-" * 40)
        print(f"  设备: {self.name} ({self.device_id})")
        print(f"  状态: {'在线' if self.online else '离线'}")
        print(f"  电源: {'开启' if self.state['power'] else '关闭'}")
        print(f"  温度: {self.state['temperature']:.1f}°C (目标 {self.state['target_temp']:.1f}°C)")
        print(f"  湿度: {self.state['humidity']:.1f}%")
        print(f"  亮度: {self.state['light']}%")
        print("-" * 40)
    
    # ---------- 后台循环 ----------
    
    def _report_loop(self):
        """定期上报状态"""
        count = 0
        while self.running:
            try:
                self.report_status()
                count += 1
                if count % 6 == 0:  # 每 30 秒打印一次状态
                    self.print_status()
            except Exception as e:
                pass
            time.sleep(5)
    
    def _simulate_environment(self):
        """模拟环境变化 - 温度湿度波动"""
        while self.running:
            # 温度随机波动
            self.state["temperature"] += random.uniform(-0.2, 0.3)
            self.state["temperature"] = max(10, min(40, self.state["temperature"]))
            
            # 湿度随机波动
            self.state["humidity"] += random.uniform(-1.0, 1.5)
            self.state["humidity"] = max(20, min(90, self.state["humidity"]))
            
            # 如果开启, 缓慢趋近目标温度
            if self.state["power"]:
                diff = self.state["target_temp"] - self.state["temperature"]
                self.state["temperature"] += diff * 0.1
            
            time.sleep(2)


def main():
    import sys
    
    # 可选参数: python virtual_device.py <设备ID> <设备名称>
    device_id = sys.argv[1] if len(sys.argv) > 1 else None
    name = sys.argv[2] if len(sys.argv) > 2 else "客厅空调"
    
    device = VirtualDevice(device_id=device_id, name=name)
    device.start()


if __name__ == "__main__":
    main()
