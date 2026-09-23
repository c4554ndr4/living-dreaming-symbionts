//! Deterministic rules shared by the preview CLI and the proof guest.
//! Evidence is supplied data; this module does not authenticate its source.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

pub const SCHEMA_VERSION: u32 = 1;
pub const MAX_INPUT_BYTES: usize = 1_048_576;
pub const MAX_RECORDS: usize = 1_000;
pub const MAX_QUESTIONS: usize = 32;
type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub policy: Policy,
    pub evidence: Evidence,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub schema_version: u32,
    /// A verifier-chosen identifier for this evaluation; not a trusted clock.
    pub run_id: String,
    pub metric: Metric,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Metric {
    SocialViews {
        account: String,
        window_start: u64,
        window_end: u64,
        views_must_exceed: u64,
    },
    MarketResolution {
        market_id: String,
        question: String,
    },
    ParticipantSurvey {
        community_id: String,
        survey_id: String,
        question_ids: Vec<String>,
        minimum_respondents: u32,
        positive_answers_bps: u32,
        positive_respondents_bps: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Evidence {
    SocialViews {
        account: String,
        posts: Vec<Post>,
    },
    MarketResolution {
        market_id: String,
        question: String,
        resolution: Resolution,
    },
    ParticipantSurvey {
        community_id: String,
        survey_id: String,
        question_ids: Vec<String>,
        respondents: Vec<Respondent>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Post {
    pub id: String,
    pub published_at: u64,
    pub views: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Respondent {
    /// Opaque local identifier. Uniqueness is checked, real-world identity is not.
    pub id: String,
    pub consented: bool,
    /// Classified answers, in the exact order declared in question_ids.
    pub positive_answers: Vec<bool>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Resolution {
    Yes,
    No,
    Unresolved,
    Cancelled,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Met,
    NotMet,
    Indeterminate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Summary {
    SocialViews {
        posts: u32,
        total_views: u64,
    },
    MarketResolution {
        resolution: Resolution,
    },
    ParticipantSurvey {
        respondents: u32,
        positive_respondents: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Journal {
    pub schema_version: u32,
    pub policy_sha256: String,
    pub evidence_sha256: String,
    pub outcome: Outcome,
    pub summary: Summary,
}

/// Retained independently by the verifier, never trusted merely because a prover supplied it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Expectation {
    pub policy: Policy,
    pub evidence_sha256: String,
}

fn require(ok: bool, message: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(message.to_owned())
    }
}

fn text(value: &str) -> Result<()> {
    require(
        !value.is_empty()
            && value.len() <= 256
            && value.trim() == value
            && !value.chars().any(char::is_control),
        "identifiers must be 1–256 bytes without surrounding whitespace or control characters",
    )
}

fn unique_strings(values: &[String], max: usize) -> Result<()> {
    require(
        !values.is_empty() && values.len() <= max,
        "invalid list length",
    )?;
    let mut seen = BTreeSet::new();
    for value in values {
        text(value)?;
        require(seen.insert(value), "duplicate identifier")?;
    }
    Ok(())
}

impl Policy {
    pub fn validate(&self) -> Result<()> {
        require(
            self.schema_version == SCHEMA_VERSION,
            "unsupported schema version",
        )?;
        text(&self.run_id)?;
        match &self.metric {
            Metric::SocialViews {
                account,
                window_start,
                window_end,
                ..
            } => {
                text(account)?;
                require(
                    window_start < window_end,
                    "window_start must precede window_end",
                )
            }
            Metric::MarketResolution {
                market_id,
                question,
            } => {
                text(market_id)?;
                text(question)
            }
            Metric::ParticipantSurvey {
                community_id,
                survey_id,
                question_ids,
                minimum_respondents,
                positive_answers_bps,
                positive_respondents_bps,
            } => {
                text(community_id)?;
                text(survey_id)?;
                unique_strings(question_ids, MAX_QUESTIONS)?;
                require(
                    (1..=MAX_RECORDS as u32).contains(minimum_respondents),
                    "minimum_respondents must be 1–1000",
                )?;
                require(
                    (1..=10_000).contains(positive_answers_bps)
                        && (1..=10_000).contains(positive_respondents_bps),
                    "percentage thresholds must be 1–10000 basis points",
                )
            }
        }
    }
}

/// Hash typed JSON, with explicit domain separation. Object key order and whitespace
/// in the input do not matter; array order and string contents deliberately do.
pub fn commitment<T: Serialize>(domain: &str, value: &T) -> Result<String> {
    let bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?;
    let mut hash = Sha256::new();
    hash.update(domain.as_bytes());
    hash.update([0]);
    hash.update(bytes);
    Ok(format!("{:x}", hash.finalize()))
}

pub fn parse_input(bytes: &[u8]) -> Result<Input> {
    require(bytes.len() <= MAX_INPUT_BYTES, "input exceeds 1 MiB")?;
    serde_json::from_slice(bytes).map_err(|e| format!("invalid input JSON: {e}"))
}

pub fn evaluate(input: &Input) -> Result<Journal> {
    input.policy.validate()?;
    let (outcome, summary) = match (&input.policy.metric, &input.evidence) {
        (
            Metric::SocialViews {
                account,
                window_start,
                window_end,
                views_must_exceed,
            },
            Evidence::SocialViews {
                account: source_account,
                posts,
            },
        ) => {
            require(account == source_account, "account mismatch")?;
            require(
                !posts.is_empty() && posts.len() <= MAX_RECORDS,
                "social evidence requires 1–1000 posts",
            )?;
            let mut seen = BTreeSet::new();
            let mut total = 0u64;
            for post in posts {
                text(&post.id)?;
                require(seen.insert(&post.id), "duplicate post")?;
                require(
                    post.published_at >= *window_start && post.published_at < *window_end,
                    "post is outside the half-open policy window",
                )?;
                total = total.checked_add(post.views).ok_or("view total overflow")?;
            }
            (
                if total > *views_must_exceed {
                    Outcome::Met
                } else {
                    Outcome::NotMet
                },
                Summary::SocialViews {
                    posts: posts.len() as u32,
                    total_views: total,
                },
            )
        }
        (
            Metric::MarketResolution {
                market_id,
                question,
            },
            Evidence::MarketResolution {
                market_id: source_id,
                question: source_question,
                resolution,
            },
        ) => {
            require(
                market_id == source_id && question == source_question,
                "market identity or question mismatch",
            )?;
            let outcome = match resolution {
                Resolution::Yes => Outcome::Met,
                Resolution::No => Outcome::NotMet,
                Resolution::Unresolved | Resolution::Cancelled => Outcome::Indeterminate,
            };
            (
                outcome,
                Summary::MarketResolution {
                    resolution: *resolution,
                },
            )
        }
        (
            Metric::ParticipantSurvey {
                community_id,
                survey_id,
                question_ids,
                minimum_respondents,
                positive_answers_bps,
                positive_respondents_bps,
            },
            Evidence::ParticipantSurvey {
                community_id: source_community,
                survey_id: source_survey,
                question_ids: source_questions,
                respondents,
            },
        ) => {
            require(
                community_id == source_community
                    && survey_id == source_survey
                    && question_ids == source_questions,
                "survey identity or ordered questions mismatch",
            )?;
            require(
                !respondents.is_empty() && respondents.len() <= MAX_RECORDS,
                "survey evidence requires 1–1000 respondents",
            )?;
            let mut seen = BTreeSet::new();
            let mut positive = 0u32;
            for respondent in respondents {
                text(&respondent.id)?;
                require(seen.insert(&respondent.id), "duplicate respondent")?;
                require(
                    respondent.consented,
                    "nonconsenting respondents must not be submitted",
                )?;
                require(
                    respondent.positive_answers.len() == question_ids.len(),
                    "incomplete or extra survey answers",
                )?;
                let yes = respondent.positive_answers.iter().filter(|v| **v).count() as u64;
                if yes * 10_000 >= question_ids.len() as u64 * u64::from(*positive_answers_bps) {
                    positive += 1;
                }
            }
            let count = respondents.len() as u32;
            let outcome = if count < *minimum_respondents {
                Outcome::Indeterminate
            } else if u64::from(positive) * 10_000
                >= u64::from(count) * u64::from(*positive_respondents_bps)
            {
                Outcome::Met
            } else {
                Outcome::NotMet
            };
            (
                outcome,
                Summary::ParticipantSurvey {
                    respondents: count,
                    positive_respondents: positive,
                },
            )
        }
        _ => return Err("metric and evidence kinds differ".to_owned()),
    };
    Ok(Journal {
        schema_version: SCHEMA_VERSION,
        policy_sha256: commitment("symbiont/policy/v1", &input.policy)?,
        evidence_sha256: commitment("symbiont/evidence/v1", &input.evidence)?,
        outcome,
        summary,
    })
}

pub fn expectation(input: &Input) -> Result<Expectation> {
    let journal = evaluate(input)?;
    Ok(Expectation {
        policy: input.policy.clone(),
        evidence_sha256: journal.evidence_sha256,
    })
}

/// Compare commitments only. This does not authenticate the journal; callers must
/// first verify its cryptographic receipt against the trusted guest image ID.
pub fn check_expectation(journal: &Journal, expected: &Expectation) -> Result<()> {
    expected.policy.validate()?;
    require(
        expected.evidence_sha256.len() == 64
            && expected
                .evidence_sha256
                .bytes()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
        "expected evidence digest must be lowercase SHA-256 hex",
    )?;
    require(
        journal.schema_version == SCHEMA_VERSION,
        "unsupported journal version",
    )?;
    require(
        journal.policy_sha256 == commitment("symbiont/policy/v1", &expected.policy)?,
        "receipt does not match expected policy",
    )?;
    require(
        journal.evidence_sha256 == expected.evidence_sha256,
        "receipt does not match expected evidence",
    )
}
