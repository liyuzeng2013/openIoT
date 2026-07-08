"""
配置管理模块
负责 NVS 存储和读取
"""

import json
import machine
from esp32 import NVS

class Config:
    """设备配置管理"""
    
    def __init__(self):
        self.nvs = NVS("openiot")
        self.device_id = self._generate_device_id()
        
    def _generate_device_id(self):
        """生成设备唯一 ID"""
        import ubinascii
        import network
        mac = network.WLAN().config('mac')
        return ubinascii.hexlify(mac).decode()[:12]
        
    def get_device_id(self):
        """获取设备 ID"""
        return self.device_id
        
    def get_mac(self):
        """获取 MAC 地址"""
        import network
        return network.WLAN().config('mac')
        
    def is_provisioned(self):
        """检查是否已配网"""
        try:
            ssid = self.nvs.get_str("wifi_ssid")
            return len(ssid) > 0
        except:
            return False
            
    def save_wifi_config(self, ssid, password):
        """保存 WiFi 配置"""
        self.nvs.set_str("wifi_ssid", ssid)
        self.nvs.set_str("wifi_password", password)
        self.nvs.commit()
        print("[Config] WiFi 配置已保存")
        
    def load_wifi_config(self):
        """加载 WiFi 配置"""
        return {
            'ssid': self.nvs.get_str("wifi_ssid"),
            'password': self.nvs.get_str("wifi_password")
        }
        
    def save_server_config(self, server_ip, server_port, device_token):
        """保存服务器配置"""
        self.nvs.set_str("server_ip", server_ip)
        self.nvs.set_i32("server_port", server_port)
        self.nvs.set_str("device_token", device_token)
        self.nvs.commit()
        print("[Config] 服务器配置已保存")
        
    def load_server_config(self):
        """加载服务器配置"""
        return {
            'server_ip': self.nvs.get_str("server_ip"),
            'server_port': self.nvs.get_i32("server_port"),
            'device_token': self.nvs.get_str("device_token"),
            'device_id': self.device_id
        }
        
    def save_mesh_config(self, net_key, app_key, device_addr):
        """保存 Mesh 网络配置"""
        # 将字节数组转为 JSON 字符串存储
        self.nvs.set_str("mesh_net_key", json.dumps(net_key))
        self.nvs.set_str("mesh_app_key", json.dumps(app_key))
        self.nvs.set_i32("mesh_device_addr", device_addr)
        self.nvs.commit()
        print("[Config] Mesh 配置已保存")
        
    def load_mesh_config(self):
        """加载 Mesh 配置"""
        try:
            net_key = json.loads(self.nvs.get_str("mesh_net_key"))
            app_key = json.loads(self.nvs.get_str("mesh_app_key"))
            device_addr = self.nvs.get_i32("mesh_device_addr")
            return {
                'net_key': bytes(net_key),
                'app_key': bytes(app_key),
                'device_addr': device_addr
            }
        except:
            return {
                'net_key': bytes(16),
                'app_key': bytes(16),
                'device_addr': 0
            }
            
    def save_device_state(self, state):
        """保存设备状态"""
        self.nvs.set_str("device_state", json.dumps(state))
        self.nvs.commit()
        
    def load_device_state(self):
        """加载设备状态"""
        try:
            return json.loads(self.nvs.get_str("device_state"))
        except:
            return {}
            
    def clear_config(self):
        """清除所有配置（恢复出厂设置）"""
        self.nvs.erase_key("wifi_ssid")
        self.nvs.erase_key("wifi_password")
        self.nvs.erase_key("server_ip")
        self.nvs.erase_key("server_port")
        self.nvs.erase_key("device_token")
        self.nvs.erase_key("mesh_net_key")
        self.nvs.erase_key("mesh_app_key")
        self.nvs.erase_key("mesh_device_addr")
        self.nvs.erase_key("device_state")
        self.nvs.commit()
        print("[Config] 配置已清除")
