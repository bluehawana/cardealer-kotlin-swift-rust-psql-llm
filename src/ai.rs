use serde::{Deserialize, Serialize};

use crate::config::AiConfig;
use crate::models::*;

/// System prompt for the vehicle evaluation AI.
const SYSTEM_PROMPT: &str = r#"You are a professional Swedish used-car evaluator, intimately familiar with Swedish Transport Agency data, vehicle inspection systems, common vehicle defects, dealer practices, market pricing, and risk assessment.

You will receive vehicle data in JSON format and must generate a professional evaluation report.

Requirements:
1. Respond in the language specified by the "locale" field (e.g., "en" for English, "zh-CN" for Chinese, "sv" for Swedish)
2. Structure your response clearly with sections
3. Include: condition inference, risk warnings, price range estimate, suitability analysis, and purchase recommendation
4. Do NOT simply repeat the raw JSON fields — provide professional interpretation and insight
5. Tone: professional, objective, and easy to understand

You MUST respond with valid JSON in the following structure:
{
  "risk": { "score": <0-100>, "level": "<low|medium|high>", "factors": [], "description": "" },
  "price_estimate": { "min_sek": 0, "max_sek": 0, "median_sek": 0, "currency": "SEK", "source": "" },
  "condition_inference": { "rating": "<excellent|good|fair|poor>", "confidence": 0.0, "indicators": [], "description": "" },
  "suitability": { "score": <0-100>, "recommendation": "<buy|consider|avoid>", "pros": [], "cons": [] },
  "ai_summary": ""
}"#;

/// LLM client for OpenAI-compatible APIs.
#[derive(Clone)]
pub struct LlmClient {
    http: reqwest::Client,
    cfg: AiConfig,
}

// ─── OpenAI request/response types ──────────────────────────────

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f64,
}

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

// ─── AI enrichment result ────────────────────────────────────────

#[derive(Deserialize)]
pub struct AiEnrichment {
    pub risk: RiskInfo,
    pub price_estimate: PriceEstimate,
    pub condition_inference: ConditionInference,
    pub suitability: Suitability,
    pub ai_summary: String,
}

impl LlmClient {
    pub fn new(cfg: AiConfig) -> Self {
        Self {
            http: reqwest::Client::new(),
            cfg,
        }
    }

    /// Generate an AI evaluation for a vehicle report.
    /// Returns None if the API key is not configured.
    pub async fn evaluate(&self, report: &VehicleReport, locale: &str) -> anyhow::Result<Option<AiEnrichment>> {
        if self.cfg.api_key.is_empty() {
            tracing::warn!("AI API key not configured, skipping AI enrichment");
            return Ok(None);
        }

        let vehicle_json = serde_json::to_string_pretty(report)?;
        let user_prompt = format!(
            "Please analyze the following vehicle data and generate an evaluation report.\n\nLocale: {}\n\nVehicle Data:\n{}",
            locale, vehicle_json
        );

        let req = ChatRequest {
            model: self.cfg.model.clone(),
            messages: vec![
                ChatMessage { role: "system".into(), content: SYSTEM_PROMPT.into() },
                ChatMessage { role: "user".into(), content: user_prompt },
            ],
            max_tokens: self.cfg.max_tokens,
            temperature: 0.3,
        };

        let url = format!("{}/chat/completions", self.cfg.base_url);
        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.cfg.api_key))
            .json(&req)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("AI API error {}: {}", status, body);
        }

        let chat_resp: ChatResponse = resp.json().await?;
        let content = chat_resp
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        match serde_json::from_str::<AiEnrichment>(&content) {
            Ok(enrichment) => Ok(Some(enrichment)),
            Err(e) => {
                tracing::warn!("failed to parse AI response as structured JSON: {}", e);
                // Return a partial enrichment with just the summary
                Ok(Some(AiEnrichment {
                    risk: RiskInfo::default(),
                    price_estimate: PriceEstimate::default(),
                    condition_inference: ConditionInference::default(),
                    suitability: Suitability::default(),
                    ai_summary: content,
                }))
            }
        }
    }
}
