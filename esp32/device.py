"""
设备管理模块
负责控制 ESP32 的 GPIO、PWM、ADC 等
"""

from machine import Pin, ADC, PWM
import json

class DeviceManager:
    """设备管理器"""
    
    def __init__(self):
        # 设备能力
        self.capabilities = {
            'light': False,
            'switch': False,
            'sensor': False,
            'plug': False
        }
        
        # 硬件引脚
        self.pins = {}
        self.pwm_channels = {}
        self.adc_channels = {}
        
        # 设备状态
        self.state = {
            'power': False,
            'brightness': 100,
            'temperature': 0,
            'humidity': 0
        }
        
    def start(self):
        """启动设备管理"""
        # 初始化默认引脚
        self._init_default_pins()
        
        # 加载保存的状态
        self._load_state()
        
        print("[Device] 设备管理已启动")
        
    def _init_default_pins(self):
        """初始化默认引脚"""
        # LED 指示灯
        self.pins['led'] = Pin(2, Pin.OUT)
        
        # 继电器控制（插座/开关）
        self.pins['relay'] = Pin(5, Pin.OUT)
        
        # PWM 调光（灯光）
        self.pwm_channels['light'] = PWM(Pin(4))
        self.pwm_channels['light'].freq(1000)
        
        # 温湿度传感器（DHT11 或 ADC）
        self.adc_channels['temp'] = ADC(Pin(34))
        self.adc_channels['temp'].atten(ADC.ATTN_11DB)
        
        self.adc_channels['humidity'] = ADC(Pin(35))
        self.adc_channels['humidity'].atten(ADC.ATTN_11DB)
        
    def execute_command(self, cmd_type, params):
        """执行设备指令"""
        result = {'success': False, 'state': {}}
        
        if cmd_type == 'power':
            self.set_power(params.get('on', False))
            result['success'] = True
            
        elif cmd_type == 'brightness':
            brightness = params.get('value', 100)
            self.set_brightness(brightness)
            result['success'] = True
            
        elif cmd_type == 'read_sensor':
            self.read_sensors()
            result['success'] = True
            
        elif cmd_type == 'toggle':
            self.toggle()
            result['success'] = True
            
        # 获取当前状态
        result['state'] = self.get_state()
        
        # 保存状态
        self._save_state()
        
        return result
        
    def set_power(self, on):
        """设置电源开关"""
        self.state['power'] = on
        
        # 控制继电器
        if 'relay' in self.pins:
            self.pins['relay'].value(1 if on else 0)
            
        # 控制灯光
        if on:
            self.set_brightness(self.state['brightness'])
        else:
            if 'light' in self.pwm_channels:
                self.pwm_channels['light'].duty(0)
                
        # 更新 LED 指示
        if 'led' in self.pins:
            self.pins['led'].value(1 if on else 0)
            
        print(f"[Device] 电源: {'开' if on else '关'}")
        
    def set_brightness(self, brightness):
        """设置亮度（0-100）"""
        brightness = max(0, min(100, brightness))
        self.state['brightness'] = brightness
        
        # PWM 占空比 (0-1023)
        duty = int(brightness * 10.23)
        
        if 'light' in self.pwm_channels:
            self.pwm_channels['light'].duty(duty)
            
        print(f"[Device] 亮度: {brightness}%")
        
    def toggle(self):
        """切换状态"""
        self.set_power(not self.state['power'])
        
    def read_sensors(self):
        """读取传感器数据"""
        # 读取温度（ADC 值转温度，需要校准）
        if 'temp' in self.adc_channels:
            adc_val = self.adc_channels['temp'].read()
            # 简单转换公式（需要实际校准）
            self.state['temperature'] = round(adc_val / 100.0, 1)
            
        # 读取湿度
        if 'humidity' in self.adc_channels:
            adc_val = self.adc_channels['humidity'].read()
            self.state['humidity'] = round(adc_val / 100.0, 1)
            
        print(f"[Device] 温度: {self.state['temperature']}°C, 湿度: {self.state['humidity']}%")
        
    def get_state(self):
        """获取设备状态"""
        return self.state.copy()
        
    def _save_state(self):
        """保存状态到 NVS"""
        try:
            from config import Config
            config = Config()
            config.save_device_state(self.state)
        except:
            pass
            
    def _load_state(self):
        """从 NVS 加载状态"""
        try:
            from config import Config
            config = Config()
            saved_state = config.load_device_state()
            if saved_state:
                self.state.update(saved_state)
                # 应用状态
                self.set_power(self.state['power'])
        except:
            pass
