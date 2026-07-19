//! `gap-report` — the distance between what a person needs and what a document guarantees.
//!
//! bLOVErAi's measuring function (order F-1). It computes one thing: **your stated
//! requirement, what the text actually guarantees, and the difference** — with every number
//! anchored to the words that produce it.
//!
//! # The ceiling, which is law rather than manners
//!
//! | this DOES | this NEVER |
//! |---|---|
//! | explain what a clause says, anchored to its text | say sign or don't sign |
//! | compute gaps against the user's own goals | characterise a deal as good or bad |
//! | enumerate document outcomes under scenarios | estimate counterparty behaviour |
//!
//! That line is the **unauthorized-practice-of-law boundary** in most US states, not only
//! this project's ethic. The ceiling sentence is fixed vocabulary: *"consider an attorney's
//! review before signing"* — third instance of the same shape, after *"consider discussing
//! this with a clinician."*
//!
//! # The absences, each with a test
//!
//! - **[`GapReport`] has no recommendation, advice, or signal field.** It reports a
//!   distance. What to do about a distance is the reader's, and in most jurisdictions it is
//!   an attorney's.
//! - **`Likelihood` is not a type in this crate.** [`Scenario`] enumerates what the
//!   *document* permits — arithmetic over deterministic text. **The counterparty's actual
//!   behaviour is not determined**, and a component that estimated it would be a liability
//!   engine that was also wrong.
//! - **[`Goals`] never leaves the user's side.** A counterparty who knows your minimum
//!   acceptable terms owns the negotiation, which makes goals the most commercially
//!   sensitive data in the system. There is no BNR-side type here that can carry one, and a
//!   test asserts it.
//! - **[`Goals`] has no field for the counterparty's goals.** Symmetry between adversarial
//!   parties comes from each side running its own instance, never from one instance seeing
//!   both.
//!
//! # Law 1d, wired in
//!
//! A [`GapReport`] carries the digest of the document it measured, and
//! [`measure`] refuses when that digest does not match the document handed to it.
//! Measuring the right goals against the wrong document is a confident answer about
//! something else.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

pub use attestation_core::{Measured, Value};

// ── the user's side ───────────────────────────────────────────────────────────

/// What a deal must do for this person. **Never transmitted.**
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Goals {
    pub requirements: Vec<Requirement>,
}

/// A single stated requirement, captured in the user's own language.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Requirement {
    pub attribute: DealAttribute,
    pub threshold: Value,
    pub direction: Direction,
    /// The language the user stated it in. Their words are the record.
    pub stated_in: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DealAttribute {
    QuantityGuaranteed,
    PaymentTiming,
    DeliveryWindow,
    VenueDistance,
    Other(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    AtLeast,
    AtMost,
}

// ── the document's side ───────────────────────────────────────────────────────

/// A verbatim quotation and where it sits. **Every number in a report traces to one.**
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnchoredQuote {
    /// Verbatim. Never paraphrased, never translated — quotation marks assert provenance.
    pub text: String,
    pub locator: String,
}

/// The document under measurement, identified by digest so a report cannot drift off it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentRef {
    pub digest: String,
    pub operative_language: String,
}

/// What a document guarantees for one attribute — and, separately, what it merely permits.
///
/// **Firm commitments and counterparty options are different fields and are never summed.**
/// That distinction is the entire finding in the specimen contract: 45,000 guaranteed,
/// 240,000 at the buyer's option, 285,000 only if you add two unlike things together.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentProvision {
    pub attribute: DealAttribute,
    pub guaranteed: Measured<Value>,
    pub at_counterparty_option: Measured<Value>,
    pub turns_on: Vec<AnchoredQuote>,
}

// ── the report ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Delta {
    pub amount: f64,
    pub unit: String,
}

/// The measurement. **No recommendation field. No advice field. No signal enum.**
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GapReport {
    pub requirement: Requirement,
    pub document_provides: Measured<Value>,
    pub gap: Option<Delta>,
    pub turns_on: Vec<AnchoredQuote>,
    /// Law 1d: the document this was measured against.
    pub measured_against: DocumentRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeasureRefusal {
    /// Law 1d — the digest handed in does not match the document loaded.
    DocumentMismatch { expected: String, got: String },
    /// Law 1a — nothing to measure.
    NoRequirements,
    /// A number with no quotation behind it is not a finding.
    UnanchoredValue { attribute: DealAttribute },
}

/// Measure goals against a document's provisions.
///
/// Refuses rather than guessing: a digest mismatch, an empty requirement set, or a provision
/// carrying a value with no anchoring quotation all produce a refusal.
pub fn measure(
    goals: &Goals,
    provisions: &[DocumentProvision],
    doc: &DocumentRef,
    loaded_digest: &str,
) -> Result<Vec<GapReport>, MeasureRefusal> {
    // Law 1d — assert you are looking at the thing before asserting anything about it.
    if doc.digest != loaded_digest {
        return Err(MeasureRefusal::DocumentMismatch {
            expected: doc.digest.clone(),
            got: loaded_digest.to_string(),
        });
    }
    // Law 1a — a measurement over zero requirements is a missing test, not a clean bill.
    if goals.requirements.is_empty() {
        return Err(MeasureRefusal::NoRequirements);
    }

    let mut out = Vec::new();
    for req in &goals.requirements {
        let prov = provisions.iter().find(|p| p.attribute == req.attribute);
        let (provides, quotes) = match prov {
            Some(p) => {
                if p.guaranteed.is_measured() && p.turns_on.is_empty() {
                    return Err(MeasureRefusal::UnanchoredValue {
                        attribute: p.attribute.clone(),
                    });
                }
                (p.guaranteed.clone(), p.turns_on.clone())
            }
            None => (Measured::NotMeasured, Vec::new()),
        };

        // The gap is arithmetic against the user's own threshold, never against a norm.
        let gap = match (&provides, req.direction) {
            (Measured::Value(v), Direction::AtLeast) if v.amount < req.threshold.amount => {
                Some(Delta {
                    amount: req.threshold.amount - v.amount,
                    unit: req.threshold.unit.clone(),
                })
            }
            (Measured::Value(v), Direction::AtMost) if v.amount > req.threshold.amount => {
                Some(Delta {
                    amount: v.amount - req.threshold.amount,
                    unit: req.threshold.unit.clone(),
                })
            }
            _ => None,
        };

        out.push(GapReport {
            requirement: req.clone(),
            document_provides: provides,
            gap,
            turns_on: quotes,
            measured_against: doc.clone(),
        });
    }
    Ok(out)
}

// ── scenarios: the document's behaviour, never the person's ───────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartyChoice {
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentOutcome {
    pub attribute: DealAttribute,
    pub value: Value,
    pub turns_on: Vec<AnchoredQuote>,
}

/// One branch the text permits. **No likelihood field, and no `Likelihood` type exists.**
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scenario {
    pub assumptions: Vec<PartyChoice>,
    pub outcome: DocumentOutcome,
}

/// The sentence a surface must show alongside any scenario set. Fixed vocabulary.
pub const COUNTERPARTY_BEHAVIOUR_SENTENCE: &str = "The document is silent on how likely this is.";

/// The ceiling sentence. Fixed vocabulary, one per domain.
pub const ATTORNEY_CEILING: &str = "Consider an attorney's review before signing.";

// ── the advice lint ───────────────────────────────────────────────────────────

/// Phrases that turn an explanation into advice. Matched case-insensitively.
const ADVICE_MARKERS: &[&str] = &[
    "you should",
    "we recommend",
    "i recommend",
    "you ought",
    "don't sign",
    "do not sign",
    "you must sign",
    "a good deal",
    "a bad deal",
    "favourable",
    "favorable",
    "unfavourable",
    "unfavorable",
    "walk away",
    "push back",
    "negotiate for",
    "ask them to",
    "likely",
    "unlikely",
    "probably",
    "chances are",
    "in my opinion",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LintFinding {
    Advice { marker: String, context: String },
    Probability { marker: String, context: String },
}

/// Lint output text for advice and probability language.
///
/// **Deliberately covers both**, because they fail the same way: advice exceeds the
/// paralegal ceiling, and probability language estimates a counterparty when the document is
/// silent. Quotation text is exempt — the contract is allowed to say whatever it says, and
/// linting a verbatim quotation would corrupt the provenance it exists to preserve.
pub fn lint_output(text: &str, verbatim_quotes: &[&str]) -> Vec<LintFinding> {
    let mut haystack = text.to_lowercase();
    // remove verbatim quotations before linting — the document's words are not our register
    for q in verbatim_quotes {
        haystack = haystack.replace(&q.to_lowercase(), " ");
    }
    let probability = ["likely", "unlikely", "probably", "chances are"];
    ADVICE_MARKERS
        .iter()
        .filter(|m| haystack.contains(**m))
        .map(|m| {
            let ctx = haystack
                .find(*m)
                .map(|i| {
                    let s = i.saturating_sub(24);
                    let e = (i + m.len() + 24).min(haystack.len());
                    haystack[s..e].to_string()
                })
                .unwrap_or_default();
            if probability.contains(m) {
                LintFinding::Probability {
                    marker: (*m).to_string(),
                    context: ctx,
                }
            } else {
                LintFinding::Advice {
                    marker: (*m).to_string(),
                    context: ctx,
                }
            }
        })
        .collect()
}

// ── what BNR is allowed to see ────────────────────────────────────────────────

/// The **only** shape that crosses to BNR: a count, and nothing about what was measured.
///
/// There is no field here that can carry a requirement, a threshold, a gap, or a document.
/// The absence is the guarantee — a counterparty who knows your minimum terms owns the
/// negotiation, so the minimum terms never travel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BnrSideTelemetry {
    pub reports_generated: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quote(t: &str) -> AnchoredQuote {
        AnchoredQuote {
            text: t.into(),
            locator: "Purchase Commitment".into(),
        }
    }

    const DIGEST: &str = "5d697c4831a4d55892580094f2c2380a17d88ccb796bb44b7f8b7637aa3a30c8";

    fn doc() -> DocumentRef {
        DocumentRef {
            digest: DIGEST.into(),
            operative_language: "en".into(),
        }
    }

    /// The grower's own words, structured. 200,000 lb guaranteed or the season does not work.
    fn grower_goals() -> Goals {
        Goals {
            requirements: vec![Requirement {
                attribute: DealAttribute::QuantityGuaranteed,
                threshold: Value::new(200_000.0, "lb"),
                direction: Direction::AtLeast,
                stated_in: "en".into(),
            }],
        }
    }

    /// What the specimen contract actually provides: 45,000 firm, 240,000 at the buyer's
    /// option. The two are never summed.
    fn singlepoint_provisions() -> Vec<DocumentProvision> {
        vec![DocumentProvision {
            attribute: DealAttribute::QuantityGuaranteed,
            guaranteed: Measured::Value(Value::new(45_000.0, "lb")),
            at_counterparty_option: Measured::Value(Value::new(240_000.0, "lb")),
            turns_on: vec![
                quote("Supplier will sell and Buyer will purchase 285,000 (two hundred eighty five thousand) lbs of Consumable Hemp Flower"),
                quote("Buyer may place monthly Purchase Orders in excess of the Initial Order."),
            ],
        }]
    }

    /// **THE END-TO-END CASE.** 200,000 required, 45,000 guaranteed, gap 155,000 — and the
    /// gap turns on the word "may".
    #[test]
    fn the_155000_gap_against_the_singlepoint_fixture() {
        let reports = measure(&grower_goals(), &singlepoint_provisions(), &doc(), DIGEST)
            .expect("measurement should succeed");
        assert_eq!(reports.len(), 1);
        let r = &reports[0];

        assert_eq!(
            r.document_provides,
            Measured::Value(Value::new(45_000.0, "lb"))
        );
        let gap = r.gap.as_ref().expect("a gap exists");
        assert_eq!(gap.amount, 155_000.0);
        assert_eq!(gap.unit, "lb");

        // every number traces to frozen text, and the operative word is present
        assert!(!r.turns_on.is_empty());
        assert!(r
            .turns_on
            .iter()
            .any(|q| q.text.contains("may place monthly")));
        assert_eq!(r.measured_against.digest, DIGEST);
    }

    /// The firm and optional volumes are never added together. 45,000 + 240,000 = 285,000
    /// arithmetically, and reporting that as "provided" is the whole defect.
    #[test]
    fn guaranteed_and_optional_are_never_summed() {
        let p = &singlepoint_provisions()[0];
        let g = p.guaranteed.as_value().unwrap().amount;
        let o = p.at_counterparty_option.as_value().unwrap().amount;
        assert_eq!(
            g + o,
            285_000.0,
            "the arithmetic reconciles — that is why it misleads"
        );
        let reports = measure(&grower_goals(), &singlepoint_provisions(), &doc(), DIGEST).unwrap();
        assert_eq!(
            reports[0].document_provides.as_value().unwrap().amount,
            45_000.0,
            "the report must carry the guaranteed figure alone"
        );
    }

    /// Law 1d — measuring the right goals against the wrong document.
    #[test]
    fn a_digest_mismatch_refuses() {
        let err = measure(
            &grower_goals(),
            &singlepoint_provisions(),
            &doc(),
            "deadbeef",
        )
        .unwrap_err();
        assert!(matches!(err, MeasureRefusal::DocumentMismatch { .. }));
    }

    /// Law 1a — zero requirements is a missing measurement, not a clean one.
    #[test]
    fn no_requirements_refuses() {
        let empty = Goals {
            requirements: vec![],
        };
        assert_eq!(
            measure(&empty, &singlepoint_provisions(), &doc(), DIGEST).unwrap_err(),
            MeasureRefusal::NoRequirements
        );
    }

    /// A number with no quotation behind it is not a finding.
    #[test]
    fn an_unanchored_value_refuses() {
        let unanchored = vec![DocumentProvision {
            attribute: DealAttribute::QuantityGuaranteed,
            guaranteed: Measured::Value(Value::new(45_000.0, "lb")),
            at_counterparty_option: Measured::NotMeasured,
            turns_on: vec![],
        }];
        assert!(matches!(
            measure(&grower_goals(), &unanchored, &doc(), DIGEST).unwrap_err(),
            MeasureRefusal::UnanchoredValue { .. }
        ));
    }

    /// A requirement the document says nothing about is NotMeasured, never zero.
    #[test]
    fn a_silent_document_is_notmeasured_not_zero() {
        let g = Goals {
            requirements: vec![Requirement {
                attribute: DealAttribute::PaymentTiming,
                threshold: Value::new(30.0, "days"),
                direction: Direction::AtMost,
                stated_in: "en".into(),
            }],
        };
        let r = measure(&g, &singlepoint_provisions(), &doc(), DIGEST).unwrap();
        assert_eq!(r[0].document_provides, Measured::NotMeasured);
        assert!(r[0].gap.is_none(), "no gap is computable against silence");
    }

    // ── the lint, proven to bite ──────────────────────────────────────────────

    #[test]
    fn the_lint_catches_advice() {
        let f = lint_output("This is a bad deal and you should walk away.", &[]);
        assert!(f.len() >= 2);
        assert!(f.iter().any(|x| matches!(x, LintFinding::Advice { .. })));
    }

    #[test]
    fn the_lint_catches_probability_language() {
        let f = lint_output("The buyer will probably exercise the option.", &[]);
        assert!(f
            .iter()
            .any(|x| matches!(x, LintFinding::Probability { .. })));
    }

    /// The register that IS allowed must pass — otherwise the lint bans the product.
    #[test]
    fn the_permitted_register_passes() {
        let ok = "Your stated requirement is 200,000 lbs guaranteed. This document \
                  guarantees 45,000. The gap is 155,000 lbs, and it turns on the word \
                  'may' in the pricing paragraph. Consider an attorney's review before signing.";
        assert!(
            lint_output(ok, &[]).is_empty(),
            "the permitted register must not trip the lint"
        );
    }

    /// A verbatim quotation is exempt — the contract may say what it says, and linting a
    /// quotation would corrupt the provenance the quotation exists to preserve.
    #[test]
    fn verbatim_quotations_are_exempt_from_the_lint() {
        let q = "Seller shall have 30 days in which to cure any breach";
        let text = format!("The remedy clause reads: \"{q}\"");
        assert!(lint_output(&text, &[q]).is_empty());
        // and with the exemption removed, a quotation containing a marker WOULD trip it —
        // proving the exemption is doing work rather than being decorative
        let marker_quote = "Buyer shall determine whether the goods are favourable";
        let t2 = format!("It reads: \"{marker_quote}\"");
        assert!(
            !lint_output(&t2, &[]).is_empty(),
            "control: unexempted text trips"
        );
        assert!(
            lint_output(&t2, &[marker_quote]).is_empty(),
            "exempted, it does not"
        );
    }

    // ── the absences ──────────────────────────────────────────────────────────

    /// Goals never leave the user's side. The only BNR-side shape carries a count.
    #[test]
    fn goals_never_appear_in_the_bnr_side_schema() {
        let t = BnrSideTelemetry {
            reports_generated: 3,
        };
        let j = serde_json::to_string(&t).unwrap().to_lowercase();
        for forbidden in [
            "requirement",
            "threshold",
            "goal",
            "gap",
            "attribute",
            "document",
            "quote",
            "digest",
            "direction",
        ] {
            assert!(
                !j.contains(forbidden),
                "BNR-side telemetry must not carry `{forbidden}` — a counterparty who knows \
                 your minimum terms owns the negotiation"
            );
        }
    }

    /// The report carries a distance and no verdict about it.
    #[test]
    fn the_report_carries_no_recommendation_or_signal() {
        let r = &measure(&grower_goals(), &singlepoint_provisions(), &doc(), DIGEST).unwrap()[0];
        let j = serde_json::to_string(r).unwrap().to_lowercase();
        for forbidden in [
            "recommend",
            "advice",
            "signal",
            "verdict",
            "rating",
            "score",
            "good",
            "bad",
            "likelihood",
            "probability",
            "confidence",
        ] {
            assert!(
                !j.contains(forbidden),
                "GapReport must not serialise `{forbidden}`"
            );
        }
    }

    /// Goals hold one party's requirements. There is no cross-principal path.
    #[test]
    fn goals_cannot_hold_the_counterpartys_goals() {
        let j = serde_json::to_string(&grower_goals())
            .unwrap()
            .to_lowercase();
        for forbidden in ["counterparty", "their_", "other_party", "buyer_goals"] {
            assert!(!j.contains(forbidden));
        }
    }

    /// A scenario describes the document, and says so.
    #[test]
    fn a_scenario_carries_no_likelihood() {
        let s = Scenario {
            assumptions: vec![PartyChoice {
                description: "buyer exercises zero options".into(),
            }],
            outcome: DocumentOutcome {
                attribute: DealAttribute::QuantityGuaranteed,
                value: Value::new(45_000.0, "lb"),
                turns_on: vec![quote(
                    "Buyer may place monthly Purchase Orders in excess of the Initial Order.",
                )],
            },
        };
        let j = serde_json::to_string(&s).unwrap().to_lowercase();
        for forbidden in ["likelihood", "probability", "chance", "odds", "expected"] {
            assert!(!j.contains(forbidden));
        }
        assert!(COUNTERPARTY_BEHAVIOUR_SENTENCE.contains("silent on how likely"));
    }

    #[test]
    fn round_trips() {
        let r = &measure(&grower_goals(), &singlepoint_provisions(), &doc(), DIGEST).unwrap()[0];
        let j = serde_json::to_string(r).unwrap();
        assert_eq!(*r, serde_json::from_str::<GapReport>(&j).unwrap());
    }
}
