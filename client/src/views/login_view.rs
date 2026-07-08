use eframe::egui;
use crate::IoTApp;
use crate::api::ApiClient;

pub fn show(app: &mut IoTApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            ui.heading("OpenIoT 智能家居平台");
            ui.add_space(40.0);

            ui.horizontal(|ui| {
                ui.label("用户名:");
                ui.text_edit_singleline(&mut app.login_username);
            });

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                ui.label("密码:");
                ui.text_edit_singleline(&mut app.login_password);
            });

            ui.add_space(20.0);

            ui.horizontal(|ui| {
                if ui.button("登录").clicked() {
                    let client = ApiClient::new();
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    match rt.block_on(client.login(&app.login_username, &app.login_password)) {
                        Ok(token_resp) => {
                            app.token = Some(token_resp.token);
                            app.username = Some(token_resp.username);
                            app.current_view = crate::AppView::Dashboard;
                            app.status_message = Some("登录成功".to_string());
                        }
                        Err(e) => {
                            app.status_message = Some(format!("登录失败: {}", e));
                        }
                    }
                }

                if ui.button("注册").clicked() {
                    let client = ApiClient::new();
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    match rt.block_on(client.register(&app.login_username, &app.login_password)) {
                        Ok(token_resp) => {
                            app.token = Some(token_resp.token);
                            app.username = Some(token_resp.username);
                            app.current_view = crate::AppView::Dashboard;
                            app.status_message = Some("注册成功".to_string());
                        }
                        Err(e) => {
                            app.status_message = Some(format!("注册失败: {}", e));
                        }
                    }
                }
            });

            if let Some(msg) = &app.status_message {
                ui.add_space(20.0);
                ui.label(msg);
            }
        });
    });
}
