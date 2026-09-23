use serde_json::{json, Value};
use symbiont_core::*;

fn fixture(name: &str) -> Value {
    let bytes: &[u8] = match name {
        "social" => include_bytes!("../../../examples/social-views.json"),
        "market" => include_bytes!("../../../examples/market-yes.json"),
        "survey" => include_bytes!("../../../examples/participant-survey.json"),
        _ => unreachable!(),
    };
    serde_json::from_slice(bytes).unwrap()
}

fn run(value: Value) -> Result<Journal, String> {
    evaluate(&parse_input(&serde_json::to_vec(&value).unwrap())?)
}

#[test]
fn synthetic_examples_recompute_expected_aggregates() {
    let social = run(fixture("social")).unwrap();
    assert_eq!(social.outcome, Outcome::Met);
    assert_eq!(
        social.summary,
        Summary::SocialViews {
            posts: 4,
            total_views: 45_000
        }
    );
    let survey = run(fixture("survey")).unwrap();
    assert_eq!(survey.outcome, Outcome::Met);
    assert_eq!(
        survey.summary,
        Summary::ParticipantSurvey {
            respondents: 5,
            positive_respondents: 4
        }
    );
}

#[test]
fn social_threshold_is_strictly_greater() {
    for (threshold, expected) in [
        (44999, Outcome::Met),
        (45000, Outcome::NotMet),
        (45001, Outcome::NotMet),
    ] {
        let mut v = fixture("social");
        v["policy"]["metric"]["views_must_exceed"] = json!(threshold);
        assert_eq!(run(v).unwrap().outcome, expected);
    }
}

#[test]
fn post_window_is_start_inclusive_end_exclusive() {
    for (time, accepted) in [
        (1751327999u64, false),
        (1751328000, true),
        (1751932799, true),
        (1751932800, false),
    ] {
        let mut v = fixture("social");
        v["evidence"]["posts"][0]["published_at"] = json!(time);
        assert_eq!(run(v).is_ok(), accepted);
    }
}

#[test]
fn duplicate_posts_empty_posts_and_wrong_accounts_are_rejected() {
    let mut v = fixture("social");
    v["evidence"]["posts"][1]["id"] = v["evidence"]["posts"][0]["id"].clone();
    assert!(run(v).unwrap_err().contains("duplicate"));
    let mut v = fixture("social");
    v["evidence"]["posts"] = json!([]);
    assert!(run(v).is_err());
    let mut v = fixture("social");
    v["evidence"]["account"] = json!("someone-else");
    assert!(run(v).is_err());
}

#[test]
fn overflow_negative_and_floating_point_views_are_rejected() {
    for bad in [json!(u64::MAX), json!(-1), json!(1.5)] {
        let mut v = fixture("social");
        v["evidence"]["posts"][0]["views"] = bad;
        assert!(run(v).is_err());
    }
}

#[test]
fn aggregate_claims_cannot_override_records() {
    let mut v = fixture("social");
    v["evidence"]["total_views"] = json!(900000);
    assert!(run(v).is_err());
    let mut v = fixture("survey");
    v["evidence"]["percent_meeting_threshold"] = json!(100);
    assert!(run(v).is_err());
}

#[test]
fn market_no_is_not_success_and_nonfinal_markets_are_indeterminate() {
    for (resolution, expected) in [
        ("yes", Outcome::Met),
        ("no", Outcome::NotMet),
        ("unresolved", Outcome::Indeterminate),
        ("cancelled", Outcome::Indeterminate),
    ] {
        let mut v = fixture("market");
        v["evidence"]["resolution"] = json!(resolution);
        assert_eq!(run(v).unwrap().outcome, expected);
    }
}

#[test]
fn market_identity_question_and_resolution_are_bound() {
    for field in ["market_id", "question", "resolution"] {
        let mut v = fixture("market");
        v["evidence"][field] = json!("forged");
        assert!(run(v).is_err());
    }
}

#[test]
fn survey_four_questions_require_four_positive_answers_at_eighty_percent() {
    let mut v = fixture("survey");
    for respondent in v["evidence"]["respondents"].as_array_mut().unwrap() {
        respondent["positive_answers"] = json!([true, true, true, false]);
    }
    let journal = run(v).unwrap();
    assert_eq!(journal.outcome, Outcome::NotMet);
    assert_eq!(
        journal.summary,
        Summary::ParticipantSurvey {
            respondents: 5,
            positive_respondents: 0
        }
    );
}

#[test]
fn survey_cohort_threshold_uses_exact_integer_arithmetic() {
    for (threshold, expected) in [
        (7999, Outcome::Met),
        (8000, Outcome::Met),
        (8001, Outcome::NotMet),
    ] {
        let mut v = fixture("survey");
        v["policy"]["metric"]["positive_respondents_bps"] = json!(threshold);
        assert_eq!(run(v).unwrap().outcome, expected);
    }
}

#[test]
fn survey_small_sample_is_indeterminate() {
    let mut v = fixture("survey");
    v["policy"]["metric"]["minimum_respondents"] = json!(6);
    assert_eq!(run(v).unwrap().outcome, Outcome::Indeterminate);
}

#[test]
fn survey_rejects_duplicates_nonconsent_and_incomplete_answers() {
    let mut v = fixture("survey");
    v["evidence"]["respondents"][1]["id"] = json!("participant-1");
    assert!(run(v).is_err());
    let mut v = fixture("survey");
    v["evidence"]["respondents"][0]["consented"] = json!(false);
    assert!(run(v).is_err());
    for answers in [
        json!([]),
        json!([true]),
        json!([true, true, true, true, true]),
    ] {
        let mut v = fixture("survey");
        v["evidence"]["respondents"][0]["positive_answers"] = answers;
        assert!(run(v).is_err());
    }
}

#[test]
fn survey_rejects_empty_samples_and_mismatched_question_order() {
    let mut v = fixture("survey");
    v["evidence"]["respondents"] = json!([]);
    assert!(run(v).is_err());
    let mut v = fixture("survey");
    v["evidence"]["question_ids"]
        .as_array_mut()
        .unwrap()
        .swap(0, 1);
    assert!(run(v).is_err());
    for field in ["community_id", "survey_id"] {
        let mut v = fixture("survey");
        v["evidence"][field] = json!("different");
        assert!(run(v).is_err());
    }
}

#[test]
fn policies_reject_invalid_thresholds_windows_and_duplicate_questions() {
    for threshold in [0, 10001] {
        for field in ["positive_answers_bps", "positive_respondents_bps"] {
            let mut v = fixture("survey");
            v["policy"]["metric"][field] = json!(threshold);
            assert!(run(v).is_err());
        }
    }
    let mut v = fixture("social");
    v["policy"]["metric"]["window_end"] = json!(0);
    assert!(run(v).is_err());
    let mut v = fixture("survey");
    v["policy"]["metric"]["question_ids"] = json!(["same", "same"]);
    assert!(run(v).is_err());
}

#[test]
fn inputs_reject_unknown_fields_versions_mismatched_types_and_trailing_json() {
    let mut v = fixture("market");
    v["policy"]["schema_version"] = json!(2);
    assert!(run(v).is_err());
    let mut v = fixture("market");
    v["undocumented"] = json!(true);
    assert!(run(v).is_err());
    let mut v = fixture("market");
    v["evidence"] = fixture("social")["evidence"].clone();
    assert!(run(v).is_err());
    assert!(parse_input(b"{} {}").is_err());
    assert!(parse_input(b"{\"policy\":").is_err());
}

#[test]
fn resource_limits_are_enforced() {
    assert!(parse_input(&vec![b' '; MAX_INPUT_BYTES + 1]).is_err());
    let mut v = fixture("social");
    v["evidence"]["posts"] = json!((0..1001)
        .map(|i| json!({"id":format!("post-{i}"),"published_at":1751328000,"views":1}))
        .collect::<Vec<_>>());
    assert!(run(v).is_err());
    let mut v = fixture("market");
    v["policy"]["run_id"] = json!("x".repeat(257));
    assert!(run(v).is_err());
}

#[test]
fn commitments_ignore_json_layout_but_bind_policy_and_evidence() {
    let value = fixture("market");
    let a = parse_input(&serde_json::to_vec_pretty(&value).unwrap()).unwrap();
    let b = parse_input(&serde_json::to_vec(&value).unwrap()).unwrap();
    let journal = evaluate(&a).unwrap();
    assert_eq!(journal, evaluate(&b).unwrap());
    let expected = expectation(&a).unwrap();
    check_expectation(&journal, &expected).unwrap();
    let mut changed = expected.clone();
    changed.policy.run_id.push_str("-replay");
    assert!(check_expectation(&journal, &changed).is_err());
    let mut changed = expected;
    changed.evidence_sha256 = "0".repeat(64);
    assert!(check_expectation(&journal, &changed).is_err());
    let mut changed_input = a;
    if let Evidence::MarketResolution { resolution, .. } = &mut changed_input.evidence {
        *resolution = Resolution::No;
    }
    assert_ne!(
        journal.evidence_sha256,
        evaluate(&changed_input).unwrap().evidence_sha256
    );
}

#[test]
fn journal_does_not_publish_raw_record_identifiers_or_answers() {
    let encoded = serde_json::to_string(&run(fixture("survey")).unwrap()).unwrap();
    for private in [
        "participant-",
        "positive_answers",
        "summer-reflection",
        "demo-community",
    ] {
        assert!(!encoded.contains(private));
    }
}

#[test]
fn source_authenticity_is_explicitly_outside_the_arithmetic_proof() {
    // Self-consistent invented data passes these rules. Authenticating the collection
    // process is a separate integration, not a property this proof can supply.
    let mut v = fixture("social");
    v["evidence"]["posts"][0]["views"] = json!(1_000_000);
    assert_eq!(run(v).unwrap().outcome, Outcome::Met);
}
