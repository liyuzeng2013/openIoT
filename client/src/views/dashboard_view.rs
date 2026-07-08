use eframe::egui;
use crate::IoTApp;
use crate::api::ApiClient;
use shared::types::*;

pub fn show(app: &mut IoTApp, ctx: &egui::Context) {
    // 顶部菜单栏
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("OpenIoT 控制台");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(username) = &app.username {
                    ui.label(format!("欢迎, {}", username));
                }
                if ui.button("退出登录").clicked() {
                    app.token = None;
                    app.username = None;
                    app.devices.clear();
                    app.current_view = crate::AppView::Login;
                }
            });
        });
    });

    // 左侧设备列表
    egui::SidePanel::left("device_list").show(ctx, |ui| {
        ui.heading("我的设备");
        ui.separator();

        if ui.button("刷新设备列表").clicked() {
            if let Some(token) = &app.token {
                let client = ApiClient::new();
                let rt = tokio::runtime::Runtime::new().unwrap();
                match rt.block_on(client.get_devices(token)) {
                    Ok(devices) => {
                        app.devices = devices;
                    }
                    Err(e) => {
                        app.status_message = Some(format!("获取设备失败: {}", e));
                    }
                }
            }
        }

        ui.separator();

        if app.devices.is_empty() {
            ui.label("暂无设备");
        } else {
            for device in &app.devices {
                let device_name = &device.name;
                let status = if device.online { "🟢 在线" } else { "🔴 离线" };
                
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.strong(device_name);
                            ui.label(format!("类型: {}", device.device_type));
                            ui.label(format!("协议: {}", device.protocol));
                            ui.label(status);
                        });
                    });
                    
                    if ui.button("控制").clicked() {
                        // TODO: 打开设备控制界面
                    }
                });
                
                ui.add_space(5.0);
            }
        }
    });

    // 主内容区
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("设备控制面板");
        ui.separator();

        if app.devices.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.label("请选择一个设备或添加新设备");
            });
        } else {
            // 显示第一个设备的控制界面作为示例
            if let Some(device) = app.devices.first() {
                ui.heading(&device.name);
                ui.label(format!("设备ID: {}", device.device_id));
                ui.label(format!("状态: {}", if device.online { "在线" } else { "离线" }));
                
                ui.separator();
                
                // 根据设备类型显示不同控制
                match device.device_type {
                    DeviceType::Light => {
                        ui.label("灯光控制");
                        let mut on = false;
                        ui.checkbox(&mut on, "开灯");
                        
                        ui.horizontal(|ui| {
                            ui.label("亮度:");
                            let mut brightness = 50;
                            ui.add(egui::Slider::new(&mut brightness, 0..=100));
                        });
                    }
                    DeviceType::Plug => {
                        ui.label("插座控制");
                        let mut on = false;
                        ui.checkbox(&mut on, "开启");
                    }
                    DeviceType::Sensor => {
                        ui.label("传感器数据");
                        ui.label("温度: 25.5°C");
                        ui.label("湿度: 60%");
                    }
                    _ => {
                        ui.label("通用控制");
                        if ui.button("发送指令").clicked() {
                            // TODO: 发送指令
                        }
                    }
                }
            }
        }

        // 底部状态栏
        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            if let Some(msg) = &app.status_message {
                ui.label(msg);
            }
        });
    });
}
