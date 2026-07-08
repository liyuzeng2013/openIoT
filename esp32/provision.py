"""
配网管理模块
负责 WiFi 配网和设备注册
"""

import socket
import json
import network

class ProvisionManager:
    """配网管理器"""
    
    def __init__(self):
        self.server_socket = None
        self.provision_callback = None
        
    def start_provision_server(self, callback):
        """启动配网服务器"""
        self.provision_callback = callback
        
        # 创建 UDP 服务器
        self.server_socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self.server_socket.bind(('0.0.0.0', 8266))
        self.server_socket.settimeout(1.0)
        
        print("[Provision] 配网服务器已启动，端口 8266")
        
        # 启动接收线程
        import _thread
        _thread.start_new_thread(self._receive_loop, ())
        
    def _receive_loop(self):
        """接收配网请求"""
        while True:
            try:
                data, addr = self.server_socket.recvfrom(1024)
                print(f"[Provision] 收到配网请求来自 {addr}")
                
                # 解析配网数据
                provision_data = json.loads(data.decode())
                
                # 发送确认
                response = json.dumps({'status': 'ok', 'message': '配网信息已接收'})
                self.server_socket.sendto(response.encode(), addr)
                
                # 调用回调
                if self.provision_callback:
                    self.provision_callback(provision_data)
                    
                break  # 配网完成，退出循环
                
            except socket.timeout:
                continue
            except Exception as e:
                print(f"[Provision] 错误: {e}")
                
    def stop(self):
        """停止配网服务器"""
        if self.server_socket:
            self.server_socket.close()
            self.server_socket = None
