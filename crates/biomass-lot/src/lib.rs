//! `biomass-lot` — what is being sold, and who says so (orders B-1 / C-1).
//!
//! The attestation machinery lives in [`attestation_core`], which is neutral and shared.
//! **This crate holds only what is actually hemp:** the statutory material categories, the
//! analytes, the FSA seed-label facts, and the total-THC arithmetic.
//!
//! The split is the point. `Jurisdiction`, `Attestor`, `Measured<T>`, `Attestation` and
//! `Eligibility` are not hemp types — a festival permit has an issuer, a date, an expiry and
//! a jurisdiction in exactly the same shape. A shared thing that lives inside one consumer
//! makes that consumer its owner, so it does not live here.
//!
//! # What this crate refuses to contain
//!
//! **No price, no quality score, no rating, no rank, no `b`, no reputation.** A lot is a
//! description of matter, not an offer and not an economy. Aggregating attestations into a
//! single number destroys the provenance the attestation model exists to preserve — a buyer
//! may sort by an attribute; this type must not decide what "better" means.
//!
//! # The type that keeps a container from shipping
//!
//! A Mexican certificate of analysis measuring **delta-9 THC only**, evaluated against the
//! US **total-THC** standard — delta-9 plus 0.877 × THCA — has not measured the
//! decarboxylated fraction at all. The honest answer is
//! [`Eligibility::NotDetermined`][attestation_core::Eligibility::NotDetermined], naming the
//! missing analyte. Never *fails*. A shipment refused on a document that proves nothing is
//! a different injustice from one refused on a document that proves something.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

pub use attestation_core::{
    coverage_gap, Attestation, Attestor, Attribute, Eligibility, Jurisdiction, Measured, Method,
    Standard, Value,
};

/// Domain aliases. The shared layer is deliberately domain-neutral; hemp reads these names.
pub type GradeAttestation = Attestation<Analyte>;
pub type GradeAttribute = Attribute<Analyte>;
pub type LotEligibility = Eligibility<Analyte>;

/// A mass as weighed, in the unit it was weighed in. Never silently converted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mass {
    pub amount: f64,
    /// `lb`, `kg`, `t` — as recorded.
    pub unit: String,
}

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

/// An analyte — the hemp domain's criterion type.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Analyte {
    Delta9Thc,
    Thca,
    Cbd,
    Cbda,
    Moisture,
    Other(String),
}

/// The standards this domain evaluates against.
pub struct Standards;

impl Standards {
    /// US total THC: delta-9 **plus** 0.877 × THCA, at or below 0.3%.
    /// Requires **both** analytes — the decarboxylation factor is exactly why.
    pub fn us_total_thc() -> Standard<Analyte> {
        Standard {
            jurisdiction: Jurisdiction::Federal,
            name: "US total THC (delta-9 + 0.877 x THCA)".into(),
            requires: vec![Analyte::Delta9Thc, Analyte::Thca],
            limit: Value::new(0.3, "%"),
        }
    }

    /// A delta-9-only standard, as several non-US regimes define it.
    pub fn delta9_only(jurisdiction: Jurisdiction, limit_pct: f64) -> Standard<Analyte> {
        Standard {
            jurisdiction,
            name: "delta-9 THC only".into(),
            requires: vec![Analyte::Delta9Thc],
            limit: Value::new(limit_pct, "%"),
        }
    }
}

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

/// Venue and governing law as **fields**, because a grower comparing two contracts should be
/// able to compare these directly rather than find them in different paragraphs.
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
    /// Coverage is checked **first**, by [`coverage_gap`], because a value measured against
    /// the wrong question is not evidence about this one. Only if every required analyte was
    /// actually assayed does any number get read.
    pub fn eligibility(&self, standard: &Standard<Analyte>) -> LotEligibility {
        let missing = coverage_gap(&self.grade, standard);
        if !missing.is_empty() {
            return Eligibility::NotDetermined { missing };
        }
        match self.total_for(standard) {
            Some(t) if t > standard.limit.amount => Eligibility::Exceeds {
                by: Value::new(t - standard.limit.amount, standard.limit.unit.clone()),
            },
            Some(_) => Eligibility::Meets,
            // Covered but no usable figure: still not determined, never a pass.
            None => Eligibility::NotDetermined {
                missing: standard.requires.clone(),
            },
        }
    }

    fn analyte_value(&self, a: &Analyte) -> Option<f64> {
        self.grade.iter().find_map(|att| match &att.attribute {
            Attribute::Measurable(x) if x == a => att.value.as_value().map(|v| v.amount),
            _ => None,
        })
    }

    fn total_for(&self, standard: &Standard<Analyte>) -> Option<f64> {
        let d9 = self.analyte_value(&Analyte::Delta9Thc)?;
        if standard.requires.contains(&Analyte::Thca) {
            let thca = self.analyte_value(&Analyte::Thca)?;
            Some(d9 + 0.877 * thca) // decarboxylation factor
        } else {
            Some(d9)
        }
    }

    /// Every attestation paired with its attestor. There is no accessor returning a value
    /// without its attestor, so a surface cannot render one by accident.
    pub fn attested_values(&self) -> Vec<(&GradeAttestation, &Attestor)> {
        self.grade.iter().map(|g| (g, &g.attestor)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn method(name: &str, covered: Vec<Analyte>) -> Method<Analyte> {
        Method {
            name: name.into(),
            covered,
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

    fn assay(a: Analyte, v: f64, covered: Vec<Analyte>) -> GradeAttestation {
        Attestation {
            attribute: Attribute::Measurable(a),
            value: Measured::Value(Value::new(v, "%")),
            method: method("HPLC", covered),
            tested_to: Jurisdiction::Federal,
            attestor: Attestor::IndependentAssessor { id: "LAB".into() },
            sampled: "2026-09-15".into(),
        }
    }

    /// **THE TEST THE CRATE EXISTS FOR.**
    #[test]
    fn delta9_only_coa_against_us_total_thc_is_not_determined() {
        let lot = lot_with(vec![assay(
            Analyte::Delta9Thc,
            0.25,
            vec![Analyte::Delta9Thc],
        )]);
        let e = lot.eligibility(&Standards::us_total_thc());
        assert_eq!(
            e,
            Eligibility::NotDetermined {
                missing: vec![Analyte::Thca]
            }
        );
        assert!(
            !e.is_failure(),
            "not-determined must never read as a failure"
        );
        assert!(!e.is_pass(), "not-determined must never read as a pass");
    }

    /// The control: the same COA IS determinable against the standard it was tested to.
    #[test]
    fn the_same_coa_is_determined_against_a_delta9_only_standard() {
        let lot = lot_with(vec![assay(
            Analyte::Delta9Thc,
            0.25,
            vec![Analyte::Delta9Thc],
        )]);
        let s = Standards::delta9_only(Jurisdiction::Foreign("MX".into()), 1.0);
        assert_eq!(lot.eligibility(&s), Eligibility::Meets);
    }

    #[test]
    fn a_full_assay_over_the_limit_returns_exceeds_not_notdetermined() {
        let both = vec![Analyte::Delta9Thc, Analyte::Thca];
        let lot = lot_with(vec![
            assay(Analyte::Delta9Thc, 0.20, both.clone()),
            assay(Analyte::Thca, 0.50, both),
        ]);
        match lot.eligibility(&Standards::us_total_thc()) {
            Eligibility::Exceeds { by } => assert!((by.amount - 0.3385).abs() < 1e-6),
            other => panic!("expected Exceeds, got {other:?}"),
        }
    }

    #[test]
    fn a_full_assay_under_the_limit_meets() {
        let both = vec![Analyte::Delta9Thc, Analyte::Thca];
        let lot = lot_with(vec![
            assay(Analyte::Delta9Thc, 0.10, both.clone()),
            assay(Analyte::Thca, 0.10, both),
        ]);
        assert_eq!(
            lot.eligibility(&Standards::us_total_thc()),
            Eligibility::Meets
        );
    }

    #[test]
    fn notmeasured_does_not_count_as_assayed() {
        let lot = lot_with(vec![Attestation {
            attribute: Attribute::Measurable(Analyte::Delta9Thc),
            value: Measured::NotMeasured,
            method: method("ordered, not run", vec![Analyte::Delta9Thc, Analyte::Thca]),
            tested_to: Jurisdiction::Federal,
            attestor: Attestor::SelfReported,
            sampled: "2026-09-15".into(),
        }]);
        match lot.eligibility(&Standards::us_total_thc()) {
            Eligibility::NotDetermined { missing } => assert_eq!(missing.len(), 2),
            other => panic!("expected NotDetermined, got {other:?}"),
        }
    }

    #[test]
    fn tribal_is_not_a_state() {
        assert_ne!(
            Jurisdiction::Tribal("N".into()),
            Jurisdiction::State("N".into())
        );
    }

    #[test]
    fn subjective_grade_records_who_judged() {
        let g: GradeAttestation = Attestation {
            attribute: Attribute::Subjective {
                judged_by: "buyer QC".into(),
            },
            value: Measured::NotMeasured,
            method: method("visual inspection", vec![]),
            tested_to: Jurisdiction::State("OR".into()),
            attestor: Attestor::Counterparty,
            sampled: "2026-10-02".into(),
        };
        match &g.attribute {
            Attribute::Subjective { judged_by } => assert_eq!(judged_by, "buyer QC"),
            _ => panic!("expected Subjective"),
        }
    }

    #[test]
    fn no_value_is_reachable_without_its_attestor() {
        let lot = lot_with(vec![assay(Analyte::Cbd, 12.0, vec![Analyte::Cbd])]);
        let pairs = lot.attested_values();
        assert_eq!(pairs.len(), 1);
        assert!(matches!(pairs[0].1, Attestor::IndependentAssessor { .. }));
    }

    /// The absences are the design.
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
