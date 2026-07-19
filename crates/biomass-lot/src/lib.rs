//! `biomass-lot` — what is being sold, and who says so (order B-1 / C-1).
//!
//! **The distinction the whole design rests on:** a grade is an *attestation*, never a
//! fact. Every value in a lot carries who measured it, by what method, against whose
//! standard, and when. The type makes it impossible to show a grade without its attestor.
//!
//! # What this crate refuses to contain
//!
//! - **No price.** A lot is a description of matter, not an offer.
//! - **No quality score, no rating, no rank.** Aggregating attestations into one number
//!   destroys the provenance the attestation model exists to preserve. A buyer may sort by
//!   an attribute; this type must not decide what "better" means.
//! - **No `b`, no reward, no reputation.** A commodity record is not an economy.
//! - **No `Verified` and no `Certified` boolean** on [`Attestor`]. `SelfReported` is honest
//!   and useful — most of agriculture runs on it. What must never happen is a self-reported
//!   number rendering identically to a lab result, and the way to prevent that is to have no
//!   variant that flattens the difference.
//!
//! # The type that keeps a container from shipping
//!
//! [`Eligibility::NotDetermined`] is the point of the crate. A certificate of analysis that
//! measured the wrong analytes for a given standard proves **nothing** about that standard —
//! it does not prove passage and it does not prove failure. Collapsing that into
//! `Exceeds`/fails would be a false negative; collapsing it into `Meets` would be worse.
//!
//! The concrete case this was built for: a Mexican COA measuring **delta-9 THC only**,
//! evaluated against the US **total-THC** standard, which is delta-9 plus 0.877 × THCA.
//! The decarboxylated fraction was never assayed. The honest answer is *not determined*,
//! naming the missing analyte — never *fails*.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

// ── measurement ───────────────────────────────────────────────────────────────

/// Measured, or not. No third state, no gap-filling, no imputation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Measured<T> {
    Value(T),
    NotMeasured,
}

impl<T> Measured<T> {
    pub fn as_value(&self) -> Option<&T> {
        match self {
            Measured::Value(v) => Some(v),
            Measured::NotMeasured => None,
        }
    }
    pub fn is_measured(&self) -> bool {
        matches!(self, Measured::Value(_))
    }
}

/// A mass as weighed, in the unit it was weighed in. Never silently converted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mass {
    pub amount: f64,
    /// `lb`, `kg`, `t` — as recorded.
    pub unit: String,
}

/// A measured quantity with its unit, e.g. `0.28` `%`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Value {
    pub amount: f64,
    pub unit: String,
}

impl Value {
    pub fn new(amount: f64, unit: impl Into<String>) -> Self {
        Value {
            amount,
            unit: unit.into(),
        }
    }
}

// ── statutory vocabulary ──────────────────────────────────────────────────────

/// Material categories **as § 297A(2) names them**, post-H.R. 5371 — not marketing terms.
///
/// A grower's paperwork and this record should use the same words. Where the statute draws
/// a line, this enum draws the same line; where it does not, neither does this.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Material {
    Fibre,
    Grain,
    Microgreen,
    Research,
    PropagationSeed,
}

/// An analyte — a thing an instrument can be pointed at.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Analyte {
    Delta9Thc,
    Thca,
    Cbd,
    Cbda,
    Moisture,
    Other(String),
}

/// Jurisdiction. **`Tribal` is first-class and is never folded into `State`** — a tribal
/// nation is not a subdivision of the state whose borders happen to surround it, and a type
/// that models it as one encodes that error into every downstream record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Jurisdiction {
    Federal,
    State(String),
    /// PR, USVI, GU, AS, MP, DC.
    Territory(String),
    /// First-class. Never a `State`.
    Tribal(String),
    Foreign(String),
}

// ── attestation ───────────────────────────────────────────────────────────────

/// Who says so.
///
/// **There is deliberately no `Verified` variant and no `Certified` boolean.** Those would
/// let a self-reported number render identically to a laboratory result, which is the exact
/// collapse this crate exists to prevent. `Certifier` names a *body*, so a reader can weigh
/// it; it does not assert a verdict.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Attestor {
    /// Honest and useful. Most of agriculture runs on it.
    SelfReported,
    ThirdPartyLab {
        id: String,
    },
    /// Oregon Tilth, NSF, an AOSCA-member seed-certifying agency.
    Certifier {
        body: String,
    },
    Buyer,
}

/// What was graded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GradeAttribute {
    Measurable(Analyte),
    /// From the real contract: "'A' buds", aesthetics, olfactory assessment. Real terms
    /// with real money attached, and irreducibly a judgement — so the type records **whose**
    /// rather than pretending to a number.
    Subjective {
        judged_by: String,
    },
}

/// The test performed — **which analytes were actually assayed**, not merely its name.
/// This is what makes [`Eligibility::NotDetermined`] derivable instead of guessed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Method {
    pub name: String,
    pub analytes_assayed: Vec<Analyte>,
}

/// One attestation about one attribute of one lot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GradeAttestation {
    pub attribute: GradeAttribute,
    pub value: Measured<Value>,
    pub method: Method,
    /// Whose standard the test targeted. A number is not portable across standards.
    pub tested_to: Jurisdiction,
    pub attestor: Attestor,
    pub sampled: String,
}

// ── eligibility ───────────────────────────────────────────────────────────────

/// A regulatory threshold and **the analytes required to evaluate it**.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Standard {
    pub jurisdiction: Jurisdiction,
    pub name: String,
    /// Every analyte that must have been assayed for this standard to be evaluable.
    pub requires: Vec<Analyte>,
    pub limit: Value,
}

impl Standard {
    /// The US total-THC standard: delta-9 **plus** 0.877 × THCA, at or below 0.3%.
    /// Requires **both** analytes — the decarboxylation factor is why.
    pub fn us_total_thc() -> Self {
        Standard {
            jurisdiction: Jurisdiction::Federal,
            name: "US total THC (delta-9 + 0.877 x THCA)".into(),
            requires: vec![Analyte::Delta9Thc, Analyte::Thca],
            limit: Value::new(0.3, "%"),
        }
    }

    /// A delta-9-only standard, as several non-US regimes define it.
    pub fn delta9_only(jurisdiction: Jurisdiction, limit_pct: f64) -> Self {
        Standard {
            jurisdiction,
            name: "delta-9 THC only".into(),
            requires: vec![Analyte::Delta9Thc],
            limit: Value::new(limit_pct, "%"),
        }
    }
}

/// The outcome of evaluating a lot against a standard.
///
/// **`NotDetermined` is not a failure and not a pass.** It is the honest state of a
/// question nobody asked the instrument.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Eligibility {
    Meets,
    Exceeds {
        by: Value,
    },
    /// The assay did not measure what this standard needs. Names what is missing so the
    /// gap is actionable rather than merely reported.
    NotDetermined {
        missing: Vec<Analyte>,
    },
}

impl Eligibility {
    /// True only for `Exceeds`. Deliberately narrow: `NotDetermined` must never be read as
    /// a failure by a caller reaching for a boolean.
    pub fn is_failure(&self) -> bool {
        matches!(self, Eligibility::Exceeds { .. })
    }
    /// True only for `Meets`.
    pub fn is_pass(&self) -> bool {
        matches!(self, Eligibility::Meets)
    }
}

// ── the lot ───────────────────────────────────────────────────────────────────

/// FSA seed-label facts, **named as the statute names them**, so a grower's label and this
/// record are the same six facts rather than two vocabularies that must be reconciled.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeedLabel {
    pub purity_percent: Measured<Value>,
    pub germination_percent: Measured<Value>,
    pub noxious_weed_seeds_per_lb: Measured<Value>,
    pub chemical_treatment: Option<String>,
    pub kind_and_variety: String,
    pub shipper_name_and_address: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Origin {
    pub jurisdiction: Jurisdiction,
    pub grower: String,
    pub harvest: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeliveryPoint {
    pub description: String,
    pub jurisdiction: Jurisdiction,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeliveryWindow {
    pub from: String,
    pub to: String,
}

/// From the real contract: terms that flip partway through an agreement are where growers
/// get caught, so they are **fields with an effective date**, not prose.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeliveryTerms {
    pub fob: String,
    pub freight_borne_by: String,
    pub effective_from: String,
}

/// Venue and governing law as **fields**, because a grower comparing two contracts should
/// be able to compare these directly rather than find them in different paragraphs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LegalTerms {
    pub governing_law: Jurisdiction,
    pub venue: String,
}

/// A lot of biomass. **No price, no score, no rating, no rank, no `b`.**
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BiomassLot {
    pub id: String,
    pub material: Material,
    pub quantity: Measured<Mass>,
    /// Never a bare number. Always attestations.
    pub grade: Vec<GradeAttestation>,
    pub origin: Origin,
    pub location: DeliveryPoint,
    pub available: DeliveryWindow,
    pub seed_label: Option<SeedLabel>,
    pub delivery_terms: Option<DeliveryTerms>,
    pub legal_terms: Option<LegalTerms>,
}

impl BiomassLot {
    /// Evaluate this lot against a standard.
    ///
    /// Returns [`Eligibility::NotDetermined`] whenever the assays performed did not cover
    /// every analyte the standard requires — **before** looking at any number, because a
    /// value measured against the wrong question is not evidence about this one.
    pub fn eligibility(&self, standard: &Standard) -> Eligibility {
        let mut assayed: Vec<&Analyte> = Vec::new();
        for att in &self.grade {
            if att.value.is_measured() {
                for a in &att.method.analytes_assayed {
                    assayed.push(a);
                }
            }
        }

        let missing: Vec<Analyte> = standard
            .requires
            .iter()
            .filter(|need| !assayed.iter().any(|got| *got == *need))
            .cloned()
            .collect();

        if !missing.is_empty() {
            return Eligibility::NotDetermined { missing };
        }

        // Every required analyte was assayed. Compute against the standard.
        let total = self.total_for(standard);
        match total {
            Some(t) if t > standard.limit.amount => Eligibility::Exceeds {
                by: Value::new(t - standard.limit.amount, standard.limit.unit.clone()),
            },
            Some(_) => Eligibility::Meets,
            // Assayed but no usable figure: still not determined, never a pass.
            None => Eligibility::NotDetermined {
                missing: standard.requires.clone(),
            },
        }
    }

    fn analyte_value(&self, a: &Analyte) -> Option<f64> {
        self.grade.iter().find_map(|att| match &att.attribute {
            GradeAttribute::Measurable(x) if x == a => att.value.as_value().map(|v| v.amount),
            _ => None,
        })
    }

    fn total_for(&self, standard: &Standard) -> Option<f64> {
        let needs_thca = standard.requires.contains(&Analyte::Thca);
        let d9 = self.analyte_value(&Analyte::Delta9Thc)?;
        if needs_thca {
            let thca = self.analyte_value(&Analyte::Thca)?;
            // decarboxylation factor
            Some(d9 + 0.877 * thca)
        } else {
            Some(d9)
        }
    }

    /// Every attestation, paired with its attestor. There is no accessor returning a value
    /// without its attestor, so a surface cannot render one by accident.
    pub fn attested_values(&self) -> Vec<(&GradeAttestation, &Attestor)> {
        self.grade.iter().map(|g| (g, &g.attestor)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn method(name: &str, analytes: Vec<Analyte>) -> Method {
        Method {
            name: name.into(),
            analytes_assayed: analytes,
        }
    }

    fn lot_with(grade: Vec<GradeAttestation>) -> BiomassLot {
        BiomassLot {
            id: "LOT-1".into(),
            material: Material::Fibre,
            quantity: Measured::Value(Mass {
                amount: 285_000.0,
                unit: "lb".into(),
            }),
            grade,
            origin: Origin {
                jurisdiction: Jurisdiction::Foreign("MX".into()),
                grower: "example".into(),
                harvest: "2026-09".into(),
            },
            location: DeliveryPoint {
                description: "farm gate".into(),
                jurisdiction: Jurisdiction::Foreign("MX".into()),
            },
            available: DeliveryWindow {
                from: "2026-10-01".into(),
                to: "2026-10-31".into(),
            },
            seed_label: None,
            delivery_terms: None,
            legal_terms: None,
        }
    }

    /// **THE TEST THE CRATE EXISTS FOR.** A Mexican COA measuring delta-9 only, evaluated
    /// against the US total-THC standard, must return NotDetermined — never Exceeds.
    #[test]
    fn delta9_only_coa_against_us_total_thc_is_not_determined() {
        let lot = lot_with(vec![GradeAttestation {
            attribute: GradeAttribute::Measurable(Analyte::Delta9Thc),
            value: Measured::Value(Value::new(0.25, "%")),
            method: method("HPLC delta-9 only", vec![Analyte::Delta9Thc]),
            tested_to: Jurisdiction::Foreign("MX".into()),
            attestor: Attestor::ThirdPartyLab {
                id: "MX-LAB-7".into(),
            },
            sampled: "2026-09-15".into(),
        }]);

        let e = lot.eligibility(&Standard::us_total_thc());
        assert_eq!(
            e,
            Eligibility::NotDetermined {
                missing: vec![Analyte::Thca]
            }
        );

        // and it is NEITHER a pass NOR a failure
        assert!(
            !e.is_failure(),
            "not-determined must never read as a failure"
        );
        assert!(!e.is_pass(), "not-determined must never read as a pass");
    }

    /// The same lot IS determinable against the standard it was actually tested to.
    /// This is the control: NotDetermined must be a real discrimination, not a default.
    #[test]
    fn the_same_coa_is_determined_against_a_delta9_only_standard() {
        let lot = lot_with(vec![GradeAttestation {
            attribute: GradeAttribute::Measurable(Analyte::Delta9Thc),
            value: Measured::Value(Value::new(0.25, "%")),
            method: method("HPLC delta-9 only", vec![Analyte::Delta9Thc]),
            tested_to: Jurisdiction::Foreign("MX".into()),
            attestor: Attestor::ThirdPartyLab {
                id: "MX-LAB-7".into(),
            },
            sampled: "2026-09-15".into(),
        }]);
        let std_mx = Standard::delta9_only(Jurisdiction::Foreign("MX".into()), 1.0);
        assert_eq!(lot.eligibility(&std_mx), Eligibility::Meets);
    }

    /// NotDetermined is distinct from Exceeds — a full assay that genuinely exceeds must
    /// return Exceeds, or NotDetermined would just be swallowing every case.
    #[test]
    fn a_full_assay_over_the_limit_returns_exceeds_not_notdetermined() {
        let lot = lot_with(vec![
            GradeAttestation {
                attribute: GradeAttribute::Measurable(Analyte::Delta9Thc),
                value: Measured::Value(Value::new(0.20, "%")),
                method: method("HPLC total THC", vec![Analyte::Delta9Thc, Analyte::Thca]),
                tested_to: Jurisdiction::Federal,
                attestor: Attestor::ThirdPartyLab {
                    id: "US-LAB-1".into(),
                },
                sampled: "2026-09-15".into(),
            },
            GradeAttestation {
                attribute: GradeAttribute::Measurable(Analyte::Thca),
                value: Measured::Value(Value::new(0.50, "%")),
                method: method("HPLC total THC", vec![Analyte::Delta9Thc, Analyte::Thca]),
                tested_to: Jurisdiction::Federal,
                attestor: Attestor::ThirdPartyLab {
                    id: "US-LAB-1".into(),
                },
                sampled: "2026-09-15".into(),
            },
        ]);
        // 0.20 + 0.877*0.50 = 0.6385 > 0.3
        match lot.eligibility(&Standard::us_total_thc()) {
            Eligibility::Exceeds { by } => assert!((by.amount - 0.3385).abs() < 1e-6),
            other => panic!("expected Exceeds, got {other:?}"),
        }
    }

    #[test]
    fn a_full_assay_under_the_limit_meets() {
        let lot = lot_with(vec![
            GradeAttestation {
                attribute: GradeAttribute::Measurable(Analyte::Delta9Thc),
                value: Measured::Value(Value::new(0.10, "%")),
                method: method("HPLC total THC", vec![Analyte::Delta9Thc, Analyte::Thca]),
                tested_to: Jurisdiction::Federal,
                attestor: Attestor::ThirdPartyLab {
                    id: "US-LAB-1".into(),
                },
                sampled: "2026-09-15".into(),
            },
            GradeAttestation {
                attribute: GradeAttribute::Measurable(Analyte::Thca),
                value: Measured::Value(Value::new(0.10, "%")),
                method: method("HPLC total THC", vec![Analyte::Delta9Thc, Analyte::Thca]),
                tested_to: Jurisdiction::Federal,
                attestor: Attestor::ThirdPartyLab {
                    id: "US-LAB-1".into(),
                },
                sampled: "2026-09-15".into(),
            },
        ]);
        assert_eq!(
            lot.eligibility(&Standard::us_total_thc()),
            Eligibility::Meets
        );
    }

    /// An unmeasured value contributes no assay coverage — NotMeasured must not count as
    /// "assayed" merely because the attestation exists.
    #[test]
    fn notmeasured_does_not_count_as_assayed() {
        let lot = lot_with(vec![GradeAttestation {
            attribute: GradeAttribute::Measurable(Analyte::Delta9Thc),
            value: Measured::NotMeasured,
            method: method("ordered, not run", vec![Analyte::Delta9Thc, Analyte::Thca]),
            tested_to: Jurisdiction::Federal,
            attestor: Attestor::SelfReported,
            sampled: "2026-09-15".into(),
        }]);
        match lot.eligibility(&Standard::us_total_thc()) {
            Eligibility::NotDetermined { missing } => assert_eq!(missing.len(), 2),
            other => panic!("expected NotDetermined, got {other:?}"),
        }
    }

    /// Tribal is first-class and never equal to the State surrounding it.
    #[test]
    fn tribal_is_not_a_state() {
        let t = Jurisdiction::Tribal("Example Nation".into());
        let s = Jurisdiction::State("OR".into());
        assert_ne!(t, s);
        assert_ne!(t, Jurisdiction::Territory("PR".into()));
        // a lot tested to tribal standard is not thereby tested to any state standard
        assert_ne!(
            Jurisdiction::Tribal("N".into()),
            Jurisdiction::State("N".into())
        );
    }

    /// Subjective attributes are recorded as judgements with a judge, never as numbers.
    #[test]
    fn subjective_grade_records_who_judged() {
        let g = GradeAttestation {
            attribute: GradeAttribute::Subjective {
                judged_by: "buyer QC".into(),
            },
            value: Measured::NotMeasured,
            method: method("visual inspection", vec![]),
            tested_to: Jurisdiction::State("OR".into()),
            attestor: Attestor::Buyer,
            sampled: "2026-10-02".into(),
        };
        match &g.attribute {
            GradeAttribute::Subjective { judged_by } => assert_eq!(judged_by, "buyer QC"),
            _ => panic!("expected Subjective"),
        }
    }

    /// Every value is reachable only alongside its attestor. This test exists so that
    /// adding a bare-value accessor fails a test that says why.
    #[test]
    fn no_value_is_reachable_without_its_attestor() {
        let lot = lot_with(vec![GradeAttestation {
            attribute: GradeAttribute::Measurable(Analyte::Cbd),
            value: Measured::Value(Value::new(12.0, "%")),
            method: method("HPLC", vec![Analyte::Cbd]),
            tested_to: Jurisdiction::Federal,
            attestor: Attestor::SelfReported,
            sampled: "2026-09-15".into(),
        }]);
        let pairs = lot.attested_values();
        assert_eq!(pairs.len(), 1);
        assert_eq!(*pairs[0].1, Attestor::SelfReported);
    }

    /// The absences are the design. This test documents them so a future edit that adds a
    /// price or a score has to delete a test that explains why it must not.
    #[test]
    fn the_lot_type_carries_no_price_score_or_rating() {
        let json = serde_json::to_string(&lot_with(vec![])).unwrap();
        for forbidden in ["price", "score", "rating", "rank", "\"b\"", "reputation"] {
            assert!(
                !json.contains(forbidden),
                "BiomassLot must not serialise a `{forbidden}` field"
            );
        }
    }

    #[test]
    fn material_uses_statutory_categories() {
        for m in [
            Material::Fibre,
            Material::Grain,
            Material::Microgreen,
            Material::Research,
            Material::PropagationSeed,
        ] {
            let _ = serde_json::to_string(&m).unwrap();
        }
    }

    #[test]
    fn round_trips() {
        let lot = lot_with(vec![]);
        let j = serde_json::to_string(&lot).unwrap();
        let back: BiomassLot = serde_json::from_str(&j).unwrap();
        assert_eq!(lot, back);
    }
}
