// OpenIoT 前端应用

const API_BASE = 'http://localhost:3000';
const WS_URL = 'ws://localhost:3000/ws';

class OpenIoTApp {
    constructor() {
        this.token = localStorage.getItem('token');
        this.username = localStorage.getItem('username');
        this.devices = [];
        this.selectedDevice = null;
        this.ws = null;
        this.init();
    }

    init() {
        if (this.token) {
            this.showDashboard();
        } else {
            this.showLogin();
        }
        this.bindEvents();
    }

    bindEvents() {
        document.getElementById('login-form').addEventListener('submit', (e) => {
            e.preventDefault();
            this.login();
        });
        document.getElementById('register-btn').addEventListener('click', () => {
            this.register();
        });
        document.getElementById('logout-btn').addEventListener('click', () => {
            this.logout();
        });
        document.getElementById('add-device-btn').addEventListener('click', () => {
            this.showModal('add-device-modal');
        });
        document.getElementById('add-device-form').addEventListener('submit', (e) => {
            e.preventDefault();
            this.addDevice();
        });
        document.querySelectorAll('.modal-close').forEach(btn => {
            btn.addEventListener('click', (e) => {
                const modal = e.target.closest('.modal');
                if (modal) this.hideModal(modal.id);
            });
        });
        this.setupModalDrag();
    }

    setupModalDrag() {
        document.querySelectorAll('.modal').forEach(modal => {
            const header = modal.querySelector('.modal-header');
            if (!header) return;
            let isDragging = false;
            let startX, startY, initLeft, initTop;

            header.style.cursor = 'move';
            header.style.userSelect = 'none';

            header.addEventListener('mousedown', (e) => {
                isDragging = true;
                const content = modal.querySelector('.modal-content');
                const rect = content.getBoundingClientRect();
                startX = e.clientX;
                startY = e.clientY;
                initLeft = rect.left;
                initTop = rect.top;
                e.preventDefault();
            });

            document.addEventListener('mousemove', (e) => {
                if (!isDragging) return;
                const dx = e.clientX - startX;
                const dy = e.clientY - startY;
                const content = modal.querySelector('.modal-content');
                content.style.position = 'fixed';
                content.style.margin = '0';
                content.style.left = (initLeft + dx) + 'px';
                content.style.top = (initTop + dy) + 'px';
            });

            document.addEventListener('mouseup', () => {
                isDragging = false;
            });
        });
    }

    connectWebSocket() {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) return;
        try {
            this.ws = new WebSocket(WS_URL);
            this.ws.onopen = () => {
                console.log('[WebSocket] Connected');
            };
            this.ws.onmessage = (event) => {
                try {
                    const msg = JSON.parse(event.data);
                    if (msg.type === 'status' && msg.state) {
                        this.updateDeviceState(msg.device_id, msg.state);
                    }
                } catch (e) {
                    console.error('Parse error:', e);
                }
            };
            this.ws.onerror = (err) => {
                console.log('[WebSocket] Error:', err);
            };
            this.ws.onclose = () => {
                console.log('[WebSocket] Closed, reconnecting...');
                setTimeout(() => this.connectWebSocket(), 3000);
            };
        } catch (e) {
            console.error('WS init error:', e);
        }
    }

    updateDeviceState(deviceId, state) {
        if (this.selectedDevice && this.selectedDevice.device_id === deviceId) {
            const tempEl = document.getElementById('temp-value');
            const humEl = document.getElementById('humidity-value');
            const lightEl = document.getElementById('light-value');
            const powerEl = document.getElementById('power-switch');

            if (tempEl && state.temperature !== undefined) {
                tempEl.textContent = state.temperature.toFixed(1);
            }
            if (humEl && state.humidity !== undefined) {
                humEl.textContent = state.humidity.toFixed(1);
            }
            if (lightEl && state.light !== undefined) {
                lightEl.textContent = state.light;
            }
            if (powerEl && state.power !== undefined) {
                powerEl.checked = state.power;
            }

            const targetTempEl = document.getElementById('target-temp-value');
            if (targetTempEl && state.target_temp !== undefined) {
                targetTempEl.textContent = state.target_temp.toFixed(1);
            }

            const statusBadge = document.getElementById('device-status');
            if (statusBadge) {
                statusBadge.textContent = '在线';
                statusBadge.className = 'status-badge online';
            }
        }

        const device = this.devices.find(d => d.device_id === deviceId);
        if (device) {
            device.online = true;
            if (state) device.state = state;
        }
        this.renderDeviceList();
    }

    showLogin() {
        document.getElementById('login-page').classList.add('active');
        document.getElementById('dashboard-page').classList.remove('active');
    }

    showDashboard() {
        document.getElementById('login-page').classList.remove('active');
        document.getElementById('dashboard-page').classList.add('active');
        document.getElementById('current-user').textContent = this.username;
        this.connectWebSocket();
        this.loadDevices();
    }

    async login() {
        const username = document.getElementById('username').value;
        const password = document.getElementById('password').value;
        try {
            const response = await fetch(`${API_BASE}/api/login`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ username, password })
            });
            const data = await response.json();
            if (data.code === 0 && data.data) {
                this.token = data.data.token;
                this.username = data.data.username;
                localStorage.setItem('token', this.token);
                localStorage.setItem('username', this.username);
                this.showMessage('login-message', '登录成功', 'success');
                setTimeout(() => this.showDashboard(), 500);
            } else {
                this.showMessage('login-message', data.message || '登录失败', 'error');
            }
        } catch (error) {
            this.showMessage('login-message', '网络错误', 'error');
        }
    }

    async register() {
        const username = document.getElementById('username').value;
        const password = document.getElementById('password').value;
        if (!username || !password) {
            this.showMessage('login-message', '请填写用户名和密码', 'error');
            return;
        }
        try {
            const response = await fetch(`${API_BASE}/api/register`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ username, password })
            });
            const data = await response.json();
            if (data.code === 0 && data.data) {
                this.token = data.data.token;
                this.username = data.data.username;
                localStorage.setItem('token', this.token);
                localStorage.setItem('username', this.username);
                this.showMessage('login-message', '注册成功', 'success');
                setTimeout(() => this.showDashboard(), 500);
            } else {
                this.showMessage('login-message', data.message || '注册失败', 'error');
            }
        } catch (error) {
            this.showMessage('login-message', '网络错误', 'error');
        }
    }

    logout() {
        localStorage.removeItem('token');
        localStorage.removeItem('username');
        this.token = null;
        this.username = null;
        this.devices = [];
        this.selectedDevice = null;
        if (this.ws) {
            this.ws.close();
            this.ws = null;
        }
        this.showLogin();
    }

    async loadDevices() {
        try {
            const response = await fetch(`${API_BASE}/api/devices`, {
                headers: { 'Authorization': `Bearer ${this.token}` }
            });
            const data = await response.json();
            if (data.code === 0 && data.data) {
                this.devices = data.data;
                this.renderDeviceList();
            }
        } catch (error) {
            console.error('Failed to load devices:', error);
        }
    }

    renderDeviceList() {
        const container = document.getElementById('device-list');
        if (this.devices.length === 0) {
            container.innerHTML = '<p style="text-align: center; color: #999; padding: 20px;">暂无设备</p>';
            return;
        }
        container.innerHTML = this.devices.map(device => {
            const st = device.state || {};
            let extra = '';
            if (st.temperature !== undefined) {
                extra = `<div style="color:#2196F3; font-size: 11px; margin-top: 4px;">温度 ${st.temperature.toFixed(1)}°C / 湿度 ${st.humidity?.toFixed(1) || '--'}%</div>`;
            }
            return `
            <div class="device-card ${this.selectedDevice?.device_id === device.device_id ? 'selected' : ''}" 
                 data-device-id="${device.device_id}">
                <div class="device-card-header">
                    <span class="device-card-name">${device.name}</span>
                    <span class="device-card-status ${device.online ? 'online' : ''}"></span>
                </div>
                <div class="device-card-info">
                    <div>${this.getDeviceTypeName(device.device_type)} / ${this.getProtocolName(device.protocol)}</div>
                    ${extra}
                </div>
            </div>`;
        }).join('');

        container.querySelectorAll('.device-card').forEach(card => {
            card.addEventListener('click', () => {
                const deviceId = card.dataset.deviceId;
                const device = this.devices.find(d => d.device_id === deviceId);
                this.selectDevice(device);
            });
        });
    }

    selectDevice(device) {
        this.selectedDevice = device;
        this.renderDeviceList();
        this.renderDeviceControl();
    }

    renderDeviceControl() {
        if (!this.selectedDevice) {
            document.getElementById('no-device-selected').classList.remove('hidden');
            document.getElementById('device-control').classList.add('hidden');
            return;
        }
        document.getElementById('no-device-selected').classList.add('hidden');
        document.getElementById('device-control').classList.remove('hidden');

        const device = this.selectedDevice;
        const st = device.state || {};

        document.getElementById('device-name').textContent = device.name;
        document.getElementById('device-id').textContent = device.device_id;
        document.getElementById('device-type').textContent = this.getDeviceTypeName(device.device_type);
        document.getElementById('device-protocol').textContent = this.getProtocolName(device.protocol);

        const statusBadge = document.getElementById('device-status');
        statusBadge.textContent = device.online ? '在线' : '离线';
        statusBadge.className = `status-badge ${device.online ? 'online' : ''}`;

        const controlsContainer = document.querySelector('.device-controls');
        controlsContainer.innerHTML = this.getDeviceControls(device, st);
        this.bindDeviceControlEvents();
    }

    getDeviceControls(device, st) {
        st = st || {};
        const temp = st.temperature !== undefined ? st.temperature.toFixed(1) : '--';
        const hum = st.humidity !== undefined ? st.humidity.toFixed(1) : '--';
        const light = st.light !== undefined ? st.light : 50;
        const target = st.target_temp !== undefined ? st.target_temp.toFixed(1) : '--';
        const power = st.power ? 'checked' : '';

        switch (device.device_type) {
            case 'Light':
                return `
                    <div class="control-item">
                        <label>电源开关</label>
                        <label class="switch">
                            <input type="checkbox" id="power-switch" ${power}>
                            <span class="slider"></span>
                        </label>
                    </div>
                    <div class="control-item">
                        <label>亮度: <span id="light-value">${light}</span>%</label>
                        <input type="range" id="brightness-slider" min="0" max="100" value="${light}">
                    </div>
                `;
            case 'Plug':
                return `
                    <div class="control-item">
                        <label>电源开关</label>
                        <label class="switch">
                            <input type="checkbox" id="power-switch" ${power}>
                            <span class="slider"></span>
                        </label>
                    </div>
                `;
            case 'Sensor':
            case 'Thermostat':
                return `
                    <div class="control-item sensor-panel">
                        <label class="panel-label">实时传感器数据</label>
                        <div class="sensor-grid">
                            <div class="sensor-item">
                                <span class="sensor-icon">🌡️</span>
                                <span class="sensor-label">当前温度</span>
                                <span class="sensor-value"><span id="temp-value">${temp}</span>°C</span>
                            </div>
                            <div class="sensor-item">
                                <span class="sensor-icon">💧</span>
                                <span class="sensor-label">相对湿度</span>
                                <span class="sensor-value"><span id="humidity-value">${hum}</span>%</span>
                            </div>
                            <div class="sensor-item">
                                <span class="sensor-icon">🎯</span>
                                <span class="sensor-label">目标温度</span>
                                <span class="sensor-value"><span id="target-temp-value">${target}</span>°C</span>
                            </div>
                            <div class="sensor-item">
                                <span class="sensor-icon">💡</span>
                                <span class="sensor-label">环境亮度</span>
                                <span class="sensor-value"><span id="light-value">${light}</span>%</span>
                            </div>
                        </div>
                        <button id="refresh-sensor" class="btn btn-small btn-primary" style="margin-top: 16px;">手动刷新</button>
                    </div>
                    <div class="control-item">
                        <label>电源控制</label>
                        <label class="switch">
                            <input type="checkbox" id="power-switch" ${power}>
                            <span class="slider"></span>
                        </label>
                    </div>
                `;
            case 'Switch':
                return `
                    <div class="control-item">
                        <label>开关状态</label>
                        <label class="switch">
                            <input type="checkbox" id="power-switch" ${power}>
                            <span class="slider"></span>
                        </label>
                    </div>
                `;
            default:
                return `
                    <div class="control-item sensor-panel">
                        <label class="panel-label">设备状态</label>
                        <div class="sensor-grid">
                            <div class="sensor-item">
                                <span class="sensor-icon">🌡️</span>
                                <span class="sensor-label">温度</span>
                                <span class="sensor-value"><span id="temp-value">${temp}</span>°C</span>
                            </div>
                            <div class="sensor-item">
                                <span class="sensor-icon">💧</span>
                                <span class="sensor-label">湿度</span>
                                <span class="sensor-value"><span id="humidity-value">${hum}</span>%</span>
                            </div>
                        </div>
                    </div>
                    <div class="control-item">
                        <button id="send-command" class="btn btn-primary">发送指令</button>
                    </div>
                `;
        }
    }

    bindDeviceControlEvents() {
        const powerSwitch = document.getElementById('power-switch');
        if (powerSwitch) {
            powerSwitch.addEventListener('change', (e) => {
                this.sendCommand('power', { on: e.target.checked });
            });
        }
        const brightnessSlider = document.getElementById('brightness-slider');
        if (brightnessSlider) {
            brightnessSlider.addEventListener('input', (e) => {
                const lv = document.getElementById('light-value');
                if (lv) lv.textContent = e.target.value;
            });
            brightnessSlider.addEventListener('change', (e) => {
                this.sendCommand('brightness', { value: parseInt(e.target.value) });
            });
        }
        const refreshSensor = document.getElementById('refresh-sensor');
        if (refreshSensor) {
            refreshSensor.addEventListener('click', () => {
                this.sendCommand('read_sensor', {});
            });
        }
        const sendCmd = document.getElementById('send-command');
        if (sendCmd) {
            sendCmd.addEventListener('click', () => {
                this.sendCommand('ping', {});
            });
        }
    }

    async sendCommand(command, params) {
        if (!this.selectedDevice) return;
        try {
            const response = await fetch(`${API_BASE}/api/devices/${this.selectedDevice.device_id}/command`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${this.token}`
                },
                body: JSON.stringify({
                    device_id: this.selectedDevice.device_id,
                    command: command,
                    params: params
                })
            });
            const data = await response.json();
            if (data.code !== 0) {
                console.error('Command failed:', data.message);
            }
        } catch (error) {
            console.error('Network error:', error);
        }
    }

    async addDevice() {
        const name = document.getElementById('device-name-input').value;
        const deviceType = document.getElementById('device-type-select').value;
        const protocol = document.getElementById('protocol-select').value;
        const deviceId = `device-${Date.now()}`;
        const device = {
            user_id: 1,
            device_id: deviceId,
            name: name,
            device_type: deviceType,
            protocol: protocol,
            token: `token-${Date.now()}`,
            online: false,
            last_seen: new Date().toISOString(),
            created_at: new Date().toISOString()
        };
        try {
            const response = await fetch(`${API_BASE}/api/devices`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${this.token}`
                },
                body: JSON.stringify(device)
            });
            const data = await response.json();
            if (data.code === 0) {
                this.hideModal('add-device-modal');
                this.loadDevices();
            } else {
                alert(data.message || '添加失败');
            }
        } catch (error) {
            alert('网络错误');
        }
    }

    showModal(modalId) {
        const modal = document.getElementById(modalId);
        if (!modal) return;
        const content = modal.querySelector('.modal-content');
        if (content) {
            content.style.position = '';
            content.style.left = '';
            content.style.top = '';
            content.style.margin = '';
        }
        modal.classList.remove('hidden');
    }

    hideModal(modalId) {
        document.getElementById(modalId).classList.add('hidden');
    }

    showMessage(elementId, message, type) {
        const element = document.getElementById(elementId);
        element.textContent = message;
        element.className = `message ${type}`;
    }

    getDeviceTypeName(type) {
        const names = {
            'Light': '灯',
            'Switch': '开关',
            'Sensor': '传感器',
            'Plug': '插座',
            'Thermostat': '温控器',
            'Unknown': '未知'
        };
        return names[type] || type;
    }

    getProtocolName(protocol) {
        const names = {
            'BluetoothMesh1': 'BLE Mesh 1.0',
            'BluetoothMesh2': 'BLE Mesh 2.0',
            'Custom2G4': '自研 2.4G'
        };
        return names[protocol] || protocol;
    }
}

document.addEventListener('DOMContentLoaded', () => {
    new OpenIoTApp();
});
