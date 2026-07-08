use shared::types::*;

const SERVER_URL: &str = "http://localhost:3000";

pub struct ApiClient {
    client: reqwest::Client,
    base_url: String,
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: SERVER_URL.to_string(),
        }
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<TokenResponse, String> {
        let resp = self
            .client
            .post(&format!("{}/api/login", self.base_url))
            .json(&LoginRequest {
                username: username.to_string(),
                password: password.to_string(),
            })
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let api_resp: ApiResponse<TokenResponse> = resp.json().await.map_err(|e| e.to_string())?;
        api_resp.data.ok_or(api_resp.message)
    }

    pub async fn register(&self, username: &str, password: &str) -> Result<TokenResponse, String> {
        let resp = self
            .client
            .post(&format!("{}/api/register", self.base_url))
            .json(&RegisterRequest {
                username: username.to_string(),
                password: password.to_string(),
            })
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let api_resp: ApiResponse<TokenResponse> = resp.json().await.map_err(|e| e.to_string())?;
        api_resp.data.ok_or(api_resp.message)
    }

    pub async fn get_devices(&self, token: &str) -> Result<Vec<Device>, String> {
        let resp = self
            .client
            .get(&format!("{}/api/devices", self.base_url))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let api_resp: ApiResponse<Vec<Device>> = resp.json().await.map_err(|e| e.to_string())?;
        api_resp.data.ok_or(api_resp.message)
    }

    #[allow(dead_code)]
    pub async fn send_command(&self, token: &str, device_id: &str, cmd: &DeviceCommand) -> Result<(), String> {
        let resp = self
            .client
            .post(&format!("{}/api/devices/{}/command", self.base_url, device_id))
            .header("Authorization", format!("Bearer {}", token))
            .json(cmd)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let api_resp: ApiResponse<String> = resp.json().await.map_err(|e| e.to_string())?;
        if api_resp.code == 0 {
            Ok(())
        } else {
            Err(api_resp.message)
        }
    }
}
