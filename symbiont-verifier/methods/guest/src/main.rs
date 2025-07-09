use risc0_zkvm::guest::env;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum MetricType {
    XAccountViews {
        account: String,
        week_start: String,
        week_end: String,
        threshold: u64,
    },
    PredictionMarket {
        market_address: String,
        market_question: String,
    },
    DiscordSurvey {
        server_id: String,
        channel: String,
        survey_questions: Vec<String>,
        threshold_percent: f64,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerificationInput {
    pub metric: MetricType,
    pub verification_data: String, // JSON data for the specific verification
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerificationResult {
    pub metric_met: bool,
    pub actual_value: String,
    pub threshold: String,
    pub evidence: String,
    pub timestamp: u64,
}

fn main() {
    // Read the verification input
    let input: VerificationInput = env::read();
    
    let result = match input.metric {
        MetricType::XAccountViews { account, week_start, week_end, threshold } => {
            verify_x_account_views(&account, &week_start, &week_end, threshold, &input.verification_data)
        },
        MetricType::PredictionMarket { market_address, market_question } => {
            verify_prediction_market(&market_address, &market_question, &input.verification_data)
        },
        MetricType::DiscordSurvey { server_id, channel, survey_questions, threshold_percent } => {
            verify_discord_survey(&server_id, &channel, &survey_questions, threshold_percent, &input.verification_data)
        },
    };
    
    // Commit the verification result to the journal
    env::commit(&result);
}

fn verify_x_account_views(account: &str, week_start: &str, week_end: &str, threshold: u64, verification_data: &str) -> VerificationResult {
    // Parse the LLM response data
    let llm_response: XAccountViewsData = serde_json::from_str(verification_data)
        .expect("Failed to parse X account views data");
    
    let metric_met = llm_response.total_views >= threshold;
    
    VerificationResult {
        metric_met,
        actual_value: llm_response.total_views.to_string(),
        threshold: threshold.to_string(),
        evidence: format!("Posts analyzed: {}, Week: {} to {}", llm_response.posts_analyzed, week_start, week_end),
        timestamp: get_current_timestamp(),
    }
}

fn verify_prediction_market(market_address: &str, market_question: &str, verification_data: &str) -> VerificationResult {
    // Parse the market resolution data
    let market_data: PredictionMarketData = serde_json::from_str(verification_data)
        .expect("Failed to parse prediction market data");
    
    VerificationResult {
        metric_met: market_data.resolved,
        actual_value: market_data.resolution.clone(),
        threshold: "true".to_string(),
        evidence: format!("Market: {}, Question: {}", market_address, market_question),
        timestamp: get_current_timestamp(),
    }
}

fn verify_discord_survey(server_id: &str, channel: &str, questions: &[String], threshold_percent: f64, verification_data: &str) -> VerificationResult {
    // Parse the survey analysis data
    let survey_data: DiscordSurveyData = serde_json::from_str(verification_data)
        .expect("Failed to parse discord survey data");
    
    let metric_met = survey_data.percent_meeting_threshold >= threshold_percent;
    
    VerificationResult {
        metric_met,
        actual_value: format!("{:.1}%", survey_data.percent_meeting_threshold),
        threshold: format!("{:.1}%", threshold_percent),
        evidence: format!("Respondents: {}, Server: {}", survey_data.total_respondents, server_id),
        timestamp: get_current_timestamp(),
    }
}

fn get_current_timestamp() -> u64 {
    // In a real implementation, this would get actual timestamp
    // For now, using a placeholder
    1735689600 // Jan 1, 2025
}

// Data structures for different verification types
#[derive(Debug, Serialize, Deserialize)]
struct XAccountViewsData {
    total_views: u64,
    posts_analyzed: u32,
    week_start: String,
    week_end: String,
    post_details: Vec<PostDetail>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PostDetail {
    url: String,
    views: u64,
    date: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PredictionMarketData {
    resolved: bool,
    resolution: String,
    market_address: String,
    resolution_date: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct DiscordSurveyData {
    total_respondents: u32,
    percent_meeting_threshold: f64,
    individual_scores: Vec<IndividualScore>,
}

#[derive(Debug, Serialize, Deserialize)]
struct IndividualScore {
    user_id: String,
    positive_response_rate: f64,
}
