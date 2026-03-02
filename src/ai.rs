use serde::{Deserialize, Serialize};

use crate::config::AiConfig;
use crate::models::*;

/// System prompt — produces deep investigative reports like a professional car evaluator.
const SYSTEM_PROMPT: &str = r#"You are a world-class Swedish used-car intelligence analyst. You have deep expertise in:
- Swedish Transport Agency (Transportstyrelsen) data interpretation
- Besiktning (MOT inspection) failure patterns and what they reveal about vehicle condition
- Mileage fraud detection — spotting inconsistencies between inspection records, service history, and listed mileage
- Re-listing pattern analysis — when a car keeps appearing on the market, it means something
- Swedish market pricing — Blocket, Bytbil, Kvdbil auction data
- Recall severity assessment — which recalls are critical vs cosmetic
- Ownership pattern analysis — 1-owner vs 7-owner cars tell very different stories
- Negotiation strategy — how to help buyers get a fair price based on data

Your reports must follow this structure:

## 1. TIMELINE ANALYSIS
Build a chronological timeline from all available data (ownership changes, inspections, recalls, listings).
Highlight suspicious gaps, rapid transfers, or patterns.

## 2. RED FLAG DETECTION
Identify and explain each concern:
- Mileage discrepancies between records
- Failed inspections (besiktning) — what broke and what it implies
- Multiple re-listings / price drops (market rejection signal)
- High owner count relative to vehicle age
- Import history concerns
Rate overall concern level: 🟢 Clean / 🟡 Caution / 🔴 Warning

## 3. MARKET INTELLIGENCE
- Current market value range (min / median / max in SEK)
- How does the asking price compare?
- Days on market / price drop history if available
- Similar vehicles for comparison

## 4. CONDITION INFERENCE
Based on inspection patterns, mileage progression, and ownership history, infer:
- Engine/drivetrain condition
- Suspension/brakes condition
- Body/rust assessment
- Overall rating: Excellent / Good / Fair / Poor
- Confidence level (0-100%)

## 5. SUITABILITY & RECOMMENDATION
- Score 0-100
- Clear recommendation: BUY / CONSIDER / AVOID
- Pros and cons list
- "Use it and resell" strategy assessment

## 6. NEGOTIATION STRATEGY (Premium feature)
If the asking price is known:
- Suggested opening offer
- Counter-offer tactics
- Key leverage points from the data
- Predicted settlement range

CRITICAL RULES:
1. Respond in the language specified by "locale" (zh-CN = Chinese, sv = Swedish, en = English, ar = Arabic, ru = Russian, fa = Persian)
2. Be direct and opinionated — buyers pay for your expert opinion, not diplomatic hedging
3. Use data to support every claim
4. Never repeat raw JSON — interpret and explain what the data MEANS
5. Include emoji indicators for quick scanning (🟢 🟡 🔴 ✅ ⚠️ ❌)

You MUST return valid JSON in this structure:
{
  "risk": { "score": <0-100>, "level": "<low|medium|high>", "factors": [], "description": "" },
  "price_estimate": { "min_sek": 0, "max_sek": 0, "median_sek": 0, "currency": "SEK", "source": "" },
  "condition_inference": { "rating": "<excellent|good|fair|poor>", "confidence": 0.0, "indicators": [], "description": "" },
  "suitability": { "score": <0-100>, "recommendation": "<buy|consider|avoid>", "pros": [], "cons": [] },
  "ai_summary": "<FULL investigative report in markdown format, 300-500 words, in the requested locale language>"
}"#;

/// System prompt for multi-vehicle comparison.
const COMPARISON_PROMPT: &str = r#"You are a world-class Swedish used-car intelligence analyst. You will receive data for MULTIPLE vehicles.

Your job is to produce a comparative analysis that helps the buyer choose the best option.

Structure:
1. COMPARISON TABLE — side-by-side overview of all vehicles (owners, mileage, inspections, price, fuel economy)
2. INDIVIDUAL ANALYSIS — brief assessment of each vehicle's strengths/weaknesses
3. RANKING — order from best to worst with clear reasoning
4. FINAL RECOMMENDATION — which one to buy and why
5. NEGOTIATION — suggested price and strategy for the recommended vehicle

CRITICAL RULES:
1. Respond in the language specified by "locale"
2. Be direct — tell the buyer exactly which car to buy
3. Include emoji for quick scanning (⭐ ratings, 🟢🔴 flags)
4. Use tables for easy comparison
5. Consider total cost of ownership (purchase + fuel + tax + expected repairs)

You MUST return valid JSON:
{
  "comparison": [
    { "plate": "", "rank": 1, "score": 0, "verdict": "" }
  ],
  "best_pick": { "plate": "", "reason": "" },
  "ai_summary": "<FULL comparative analysis in markdown, in the requested locale language>"
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub comparison: Vec<ComparisonItem>,
    pub best_pick: BestPick,
    pub ai_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonItem {
    pub plate: String,
    pub rank: i32,
    pub score: i32,
    pub verdict: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BestPick {
    pub plate: String,
    pub reason: String,
}

impl LlmClient {
    pub fn new(cfg: AiConfig) -> Self {
        Self {
            http: reqwest::Client::new(),
            cfg,
        }
    }

    /// Generate an AI evaluation for a single vehicle.
    pub async fn evaluate(&self, report: &VehicleReport, listings: &[ListingRow], locale: &str) -> anyhow::Result<Option<AiEnrichment>> {
        if self.cfg.api_key.is_empty() {
            tracing::warn!("AI API key not configured, skipping AI enrichment");
            return Ok(None);
        }

        let vehicle_json = serde_json::to_string_pretty(report)?;
        let listings_json = serde_json::to_string_pretty(listings)?;

        let user_prompt = format!(
            "Analyze this vehicle and generate a deep investigative report.\n\nLocale: {}\n\nVehicle Data:\n{}\n\nListing History:\n{}",
            locale, vehicle_json, listings_json
        );

        let content = self.chat(SYSTEM_PROMPT, &user_prompt).await?;

        match serde_json::from_str::<AiEnrichment>(&content) {
            Ok(enrichment) => Ok(Some(enrichment)),
            Err(e) => {
                tracing::warn!("failed to parse AI response as structured JSON: {}", e);
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

    /// Compare multiple vehicles and recommend the best one.
    pub async fn compare(&self, reports: &[VehicleReport], listings: &[Vec<ListingRow>], locale: &str) -> anyhow::Result<Option<ComparisonResult>> {
        if self.cfg.api_key.is_empty() {
            tracing::warn!("AI API key not configured, skipping comparison");
            return Ok(None);
        }

        let mut data_parts = Vec::new();
        for (i, report) in reports.iter().enumerate() {
            let vehicle_json = serde_json::to_string_pretty(report)?;
            let listing_json = if i < listings.len() {
                serde_json::to_string_pretty(&listings[i])?
            } else {
                "[]".to_string()
            };
            data_parts.push(format!("--- Vehicle {} ({}) ---\n{}\n\nListing History:\n{}", i + 1, report.plate, vehicle_json, listing_json));
        }

        let user_prompt = format!(
            "Compare these {} vehicles and recommend the best purchase.\n\nLocale: {}\n\n{}",
            reports.len(), locale, data_parts.join("\n\n")
        );

        let content = self.chat(COMPARISON_PROMPT, &user_prompt).await?;

        match serde_json::from_str::<ComparisonResult>(&content) {
            Ok(result) => Ok(Some(result)),
            Err(e) => {
                tracing::warn!("failed to parse comparison response: {}", e);
                Ok(Some(ComparisonResult {
                    comparison: vec![],
                    best_pick: BestPick { plate: String::new(), reason: String::new() },
                    ai_summary: content,
                }))
            }
        }
    }

    /// Low-level chat completion call.
    async fn chat(&self, system: &str, user: &str) -> anyhow::Result<String> {
        let req = ChatRequest {
            model: self.cfg.model.clone(),
            messages: vec![
                ChatMessage { role: "system".into(), content: system.into() },
                ChatMessage { role: "user".into(), content: user.into() },
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

        Ok(content)
    }
}
