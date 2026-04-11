use serde::Deserialize;
use std::collections::HashMap;

pub type ApiData = HashMap<String, Provider>;

#[derive(Debug, Deserialize)]
pub struct Provider {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub env: Vec<String>,
    #[serde(default)]
    pub npm: String,
    #[serde(default)]
    pub api: String,
    #[serde(default)]
    pub doc: String,
    #[serde(default)]
    pub models: HashMap<String, Model>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Model {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub family: Option<String>,
    #[serde(default)]
    pub attachment: bool,
    #[serde(default)]
    pub reasoning: bool,
    #[serde(default)]
    pub tool_call: bool,
    #[serde(default)]
    pub temperature: bool,
    #[serde(default)]
    pub knowledge: Option<String>,
    #[serde(default)]
    pub release_date: Option<String>,
    #[serde(default)]
    pub last_updated: Option<String>,
    #[serde(default)]
    pub modalities: Modalities,
    #[serde(default)]
    pub open_weights: Option<bool>,
    #[serde(default)]
    pub cost: Option<Cost>,
    #[serde(default)]
    pub limit: Option<Limit>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Modalities {
    #[serde(default)]
    pub input: Vec<String>,
    #[serde(default)]
    pub output: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Cost {
    #[serde(default)]
    pub input: Option<f64>,
    #[serde(default)]
    pub output: Option<f64>,
    #[serde(default)]
    pub reasoning: Option<f64>,
    #[serde(default)]
    pub cache_read: Option<f64>,
    #[serde(default)]
    pub cache_write: Option<f64>,
    #[serde(default)]
    pub input_audio: Option<f64>,
    #[serde(default)]
    pub output_audio: Option<f64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Limit {
    #[serde(default)]
    pub context: Option<u64>,
    #[serde(default)]
    pub output: Option<u64>,
}

pub fn fetch_data() -> Result<ApiData, Box<dyn std::error::Error>> {
    let url = "https://models.dev/api.json";
    let response = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?
        .get(url)
        .send()?;
    let text = response.text()?;
    let data: ApiData = serde_json::from_str(&text)?;
    Ok(data)
}
