"""
OpenIoT ESP32 MicroPython 固件
支持蓝牙 Mesh 1.0/2.0 和自研 2.4G 协议
"""

import network
import json
import time
from machine import Pin, ADC, PWM
import ntptime
import ubinascii

# 导入自定义模块
from config import Config
from provision import ProvisionManager
from protocol import ProtocolHandler
from mesh import MeshNetwork
from device import DeviceManager

class OpenIoTDevice:
    """OpenIoT 设备主类"""
    
    def __init__(self):
        self.config = Config()
        self.provision = ProvisionManager()
        self.protocol = ProtocolHandler()
        self.mesh = MeshNetwork()
        self.device_mgr = DeviceManager()
        
        # 设备状态
        self.connected = False
        self.server_ip = None
        
    def boot(self):
        """设备启动流程"""
        print("[OpenIoT] 设备启动中...")
        
        # 1. 检查是否已配网
        if not self.config.is_provisioned():
            print("[OpenIoT] 未配网，进入配网模式")
            self.enter_provision_mode()
        else:
            print("[OpenIoT] 已配网，连接到网络")
            self.connect_to_network()
            
    def enter_provision_mode(self):
        """进入配网模式"""
        # 创建 WiFi 热点
        ap = network.WLAN(network.AP_IF)
        ap.active(True)
        
        # 生成设备唯一 ID
        device_id = ubinascii.hexlify(self.config.get_mac()).decode()[:8]
        ssid = f"OpenIoT-{device_id}"
        
        ap.config(ssid=ssid, password="12345678", authmode=network.WLAN.WEP)
        print(f"[Provision] AP 已启动: {ssid}")
        
        # 启动配网服务
        self.provision.start_provision_server(self.on_provision_complete)
        
    def on_provision_complete(self, config_data):
        """配网完成回调"""
        print("[Provision] 配网完成，保存配置")
        
        # 保存配置到 NVS
        self.config.save_wifi_config(
            config_data['ssid'],
            config_data['password']
        )
        
        # 保存服务器信息
        self.config.save_server_config(
            config_data['server_ip'],
            config_data['server_port'],
            config_data['device_token']
        )
        
        # 保存 Mesh 网络配置
        self.config.save_mesh_config(
            config_data['net_key'],
            config_data['app_key'],
            config_data['device_addr']
        )
        
        # 重启设备
        print("[Provision] 重启设备...")
        time.sleep(1)
        import machine
        machine.reset()
        
    def connect_to_network(self):
        """连接到 WiFi 网络"""
        wifi_config = self.config.load_wifi_config()
        
        sta = network.WLAN(network.STA_IF)
        sta.active(True)
        sta.connect(wifi_config['ssid'], wifi_config['password'])
        
        print(f"[WiFi] 正在连接到 {wifi_config['ssid']}...")
        
        # 等待连接
        timeout = 30
        while not sta.isconnected() and timeout > 0:
            time.sleep(1)
            timeout -= 1
            print(".", end="")
            
        if sta.isconnected():
            print("\n[WiFi] 连接成功")
            ip = sta.ifconfig()[0]
            print(f"[WiFi] IP 地址: {ip}")
            self.connected = True
            
            # 同步时间
            try:
                ntptime.settime()
                print("[NTP] 时间同步成功")
            except:
                print("[NTP] 时间同步失败")
                
            # 启动设备服务
            self.start_device_services()
        else:
            print("\n[WiFi] 连接失败，进入配网模式")
            self.enter_provision_mode()
            
    def start_device_services(self):
        """启动设备服务"""
        # 加载设备配置
        server_config = self.config.load_server_config()
        mesh_config = self.config.load_mesh_config()
        
        # 初始化 Mesh 网络
        self.mesh.init(mesh_config)
        
        # 启动协议处理器
        self.protocol.start(server_config, self.on_command_received)
        
        # 启动心跳
        self.start_heartbeat()
        
        # 启动设备管理
        self.device_mgr.start()
        
    def on_command_received(self, command):
        """处理收到的指令"""
        print(f"[Command] 收到指令: {command}")
        
        # 解析指令
        cmd_type = command.get('type')
        params = command.get('params', {})
        
        # 执行设备控制
        result = self.device_mgr.execute_command(cmd_type, params)
        
        # 上报状态
        self.report_status(result)
        
    def report_status(self, status):
        """上报设备状态"""
        status_msg = {
            'device_id': self.config.get_device_id(),
            'state': status,
            'timestamp': time.time()
        }
        
        # 通过协议发送
        self.protocol.send_status(status_msg)
        
    def start_heartbeat(self):
        """启动心跳"""
        import _thread
        
        def heartbeat_task():
            while True:
                if self.connected:
                    self.protocol.send_heartbeat()
                time.sleep(30)  # 30秒心跳
                
        _thread.start_new_thread(heartbeat_task, ())

def main():
    """主函数"""
    device = OpenIoTDevice()
    device.boot()
    
    # 主循环
    while True:
        time.sleep(1)

if __name__ == "__main__":
    main()
