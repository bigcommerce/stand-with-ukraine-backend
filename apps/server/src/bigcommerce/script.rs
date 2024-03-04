use serde::Deserialize;

#[derive(Deserialize)]
pub struct ListResponse {
    pub data: Vec<GetResponse>,
}

#[derive(Deserialize)]
pub struct GetResponse {
    pub uuid: String,
    pub api_client_id: String,
    pub enabled: bool,
    pub channel_id: i16,
    pub name: String,
}

#[derive(Debug)]
pub struct Script {
    name: String,
    description: String,
    html: String,
}

impl Script {
    pub const fn new(name: String, description: String, html: String) -> Self {
        Self {
            name,
            description,
            html,
        }
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn generate_script_body(&self) -> serde_json::Value {
        serde_json::json!({
            "name": self.name,
            "description": self.description,
            "html": self.html,
            "kind": "script_tag",
            "load_method": "default",
            "location": "footer",
            "visibility": "storefront",
            "consent_category": "essential",
            "auto_uninstall": true,
            "enabled": true,
        })
    }
}
