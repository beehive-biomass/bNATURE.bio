//! `treasury` — the fiscal layer of an accord (order F-3, legs T-1 through T-5).
//!
//! **TYPES AND LINT ONLY. Live rails wait on counsel; the types do not.** No leg here holds
//! `b`. The tithe and the duty are stables; obligations are computed at formation and
//! remitted on release; the citizen holds their own record and BNR reports nothing.
//!
//! # What each leg is
//!
//! - **T-1 · the tithe.** ~10% of collected accord fees, gifted to a treasury, **DAO-ratified,
//!   never fiat.** A disbursement cannot be constructed without a ratification record —
//!   there is no path to a tithe the senary circles did not vote. Framed as infrastructure
//!   reciprocity and **never as tax substitution**, enforced by a copy lint that bars both
//!   directions: no phrasing that it offsets taxes, and no phrasing that calls it a tax.
//! - **T-2 · the duty leg.** Tariffs computed and disclosed *before* signature, escrowed as
//!   their own leg, remitted on release. Law 1d: a duty computed against a jurisdiction
//!   other than the accord's refuses.
//! - **T-3 · the permission screen.** Sanctions, licences, corridor rules — nation-to-nation
//!   and state-to-state alike. A screened accord holds at `NotDetermined` pending licence
//!   rather than executing into a violation. **Law 1a with teeth: a screen that loaded zero
//!   rules REFUSES.** An empty prohibition list is a broken feed, not a clean world.
//! - **T-4 · the citizen's record.** Each party receives a complete itemised record of the
//!   transaction's obligations, **theirs.** BNR reports nothing: there is no
//!   report-to-government path to construct. Compliance becomes trivial for the person and
//!   verification trivial for the state, without surveillance, because the record sits in
//!   the citizen's hands.
//! - **T-5 · rails.** Settlement in existing regulated stables. **No BNR-issued fiat token**;
//!   `fUSD` is definition-only and may not appear in any executable path.
//!
//! # Not built here, and why
//!
//! **T-0 — the Treasury smartCONTRACT that may hold `b` as collateral — is deliberately
//! absent.** The fire order scopes F-3 to legs T-1–T-5, and T-0 is the one place `b` custody
//! lives. It is ratified as design (RELAY_05, 2026-07-19) but is not in this build step, so
//! it is not built here.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

pub use attestation_core::{Eligibility, Jurisdiction, Value};

/// A stable-denominated amount. No leg here is denominated in `b`.
pub type Stable = Value;

// ── T-1 · the tithe ───────────────────────────────────────────────────────────

/// A record that the senary circles ratified an allocation. **The only way to a tithe.**
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaoRatification {
    pub proposal_id: String,
    /// The rate the circles voted, in basis points. DAO-tunable; never founder fiat.
    pub rate_bps: u32,
    pub tally: VoteTally,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoteTally {
    pub for_votes: u64,
    pub against_votes: u64,
}

impl DaoRatification {
    /// Ratified means more for than against, over a non-empty vote. A tithe cannot rest on
    /// a vote nobody cast (Law 1a).
    pub fn is_ratified(&self) -> bool {
        let total = self.tally.for_votes + self.tally.against_votes;
        total > 0 && self.tally.for_votes > self.tally.against_votes
    }
}

/// The recipient treasury and the legal basis for a *gift* (not a payment of obligation).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GiftDestination {
    pub jurisdiction: Jurisdiction,
    /// e.g. "31 U.S.C. §3113" — the gifts account. Named, so the basis is checkable.
    pub statutory_basis: String,
}

/// A tithe disbursement. **Constructible only with a ratification record**, and only when
/// that record is actually ratified — the type makes an un-voted tithe unrepresentable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TitheDisbursement {
    amount: Stable,
    destination: GiftDestination,
    ratified_by: DaoRatification,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TitheRefusal {
    /// No disbursement without a ratification record that actually passed.
    NotRatified,
}

impl TitheDisbursement {
    pub fn create(
        amount: Stable,
        destination: GiftDestination,
        ratified_by: DaoRatification,
    ) -> Result<Self, TitheRefusal> {
        if !ratified_by.is_ratified() {
            return Err(TitheRefusal::NotRatified);
        }
        Ok(TitheDisbursement {
            amount,
            destination,
            ratified_by,
        })
    }

    pub fn amount(&self) -> &Stable {
        &self.amount
    }
    pub fn ratification(&self) -> &DaoRatification {
        &self.ratified_by
    }
}

/// Copy lint for tithe-facing text. **Bars both directions of the distinction:** text may not
/// imply the tithe offsets anyone's taxes, and may not call the tithe a tax. A tithe is a
/// voluntary gift; a tax is an obligation; the whole legal and moral content is that they are
/// different, so the words must keep them different.
pub fn tithe_copy_findings(text: &str) -> Vec<&'static str> {
    let t = text.to_lowercase();
    let mut out = Vec::new();
    // direction 1: implying it offsets / replaces tax
    for phrase in [
        "instead of tax",
        "in lieu of tax",
        "offset your tax",
        "offsets tax",
        "reduces your tax",
        "counts toward tax",
        "tax credit",
        "deduct",
        "write-off",
        "write off",
    ] {
        if t.contains(phrase) {
            out.push("implies-tax-offset");
            break;
        }
    }
    // direction 2: calling the tithe itself a tax
    for phrase in [
        "the tithe is a tax",
        "tithe tax",
        "this tax",
        "our tax",
        "a tax we",
    ] {
        if t.contains(phrase) {
            out.push("calls-tithe-a-tax");
            break;
        }
    }
    out
}

// ── T-2 · the duty leg ────────────────────────────────────────────────────────

/// A duty computed at formation, escrowed as its own leg, remitted on release.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DutyLeg {
    pub amount: Stable,
    /// The jurisdiction whose tariff this is.
    pub jurisdiction: Jurisdiction,
    /// Disclosed before signature — part of the landed cost, honest up front.
    pub disclosed_at_formation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DutyRefusal {
    /// Law 1d — a duty computed against a jurisdiction other than the accord's is a
    /// confident number about the wrong border.
    JurisdictionMismatch,
    /// A duty a grower does not see before signing is not honest landed cost.
    NotDisclosedAtFormation,
}

/// Attach a duty leg to an accord, refusing on a jurisdiction mismatch or undisclosed cost.
pub fn duty_for_accord(
    leg: DutyLeg,
    accord_jurisdiction: &Jurisdiction,
) -> Result<DutyLeg, DutyRefusal> {
    if leg.jurisdiction != *accord_jurisdiction {
        return Err(DutyRefusal::JurisdictionMismatch);
    }
    if !leg.disclosed_at_formation {
        return Err(DutyRefusal::NotDisclosedAtFormation);
    }
    Ok(leg)
}

// ── T-3 · the permission screen ───────────────────────────────────────────────

/// A prohibition rule — a sanction, a licence requirement, a corridor rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProhibitionRule {
    pub id: String,
    /// Whose rule — nation-to-nation and state-to-state alike.
    pub jurisdiction: Jurisdiction,
}

/// The outcome of screening an accord. A screened accord **holds** pending licence rather
/// than executing into a violation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScreenOutcome {
    Clear,
    /// Held pending licence. Uses the eligibility shape: not a failure, a not-yet-determined.
    HoldPendingLicence {
        matched: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScreenRefusal {
    /// **Law 1a with teeth.** A screen loaded with zero rules is a broken feed, not a clean
    /// world. It refuses rather than clearing.
    EmptyRuleSet,
}

/// Screen an accord against a rule set. **Refuses if the rule set is empty** — an empty
/// prohibition list means the feed failed to load, and clearing on it would pass every
/// sanctioned deal.
pub fn screen(
    rules: &[ProhibitionRule],
    hits: impl Fn(&ProhibitionRule) -> bool,
) -> Result<ScreenOutcome, ScreenRefusal> {
    if rules.is_empty() {
        return Err(ScreenRefusal::EmptyRuleSet);
    }
    let matched: Vec<String> = rules
        .iter()
        .filter(|r| hits(r))
        .map(|r| r.id.clone())
        .collect();
    if matched.is_empty() {
        Ok(ScreenOutcome::Clear)
    } else {
        Ok(ScreenOutcome::HoldPendingLicence { matched })
    }
}

/// The eligibility a held accord carries: `NotDetermined`, never a pass, never a silent
/// execution. Provided so the accord engine reads screening in the same vocabulary as
/// border control.
pub fn screen_eligibility<C: Clone>(outcome: &ScreenOutcome, licence: C) -> Eligibility<C> {
    match outcome {
        ScreenOutcome::Clear => Eligibility::Meets,
        ScreenOutcome::HoldPendingLicence { .. } => Eligibility::NotDetermined {
            missing: vec![licence],
        },
    }
}

// ── T-4 · the citizen's record ────────────────────────────────────────────────

/// A complete itemised record of one transaction's obligations, **held by the citizen.**
///
/// The absence is the guarantee: there is no `report_to`, no `filed_with`, no government
/// recipient field, and no method that transmits this anywhere. A serialisation test bars the
/// vocabulary. Voluntary remittance is an election the citizen makes, never a default.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CitizenRecord {
    pub party: String,
    pub jurisdiction: Jurisdiction,
    pub obligations: Vec<Obligation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Obligation {
    pub description: String,
    pub amount: Stable,
    /// True only if the citizen elected to remit at settlement. Default is false — an
    /// election, never automatic.
    pub remitted_by_election: bool,
}

// ── T-5 · rails ───────────────────────────────────────────────────────────────

/// The settlement currency of a leg. **Stables only.** `fUSD` is definition-only and has no
/// variant here — it cannot enter an executable path because the type cannot name it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettlementCurrency {
    /// An existing regulated stablecoin (USDC, USDT, …).
    RegulatedStable(String),
    /// A fiat currency settled through a licensed agent.
    Fiat(String),
    // No `FUsd` variant. fUSD is counsel-gated definition-only; naming it here would put it
    // in an executable path, which the negative control forbids.
}

/// Escrow-tier pricing. Private escrow prices **above** public, and the differential funds
/// the free layer — privacy costs the commons aggregate data, so it pays more.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TierPricing {
    /// Public escrow, with consented aggregate disclosure (the `bData` accord).
    pub public_price: Stable,
    /// Private escrow (the `zbData` accord).
    pub private_price: Stable,
}

impl TierPricing {
    /// Private must price at or above public — the invariant that funds the free layer.
    pub fn is_valid(&self) -> bool {
        self.private_price.amount >= self.public_price.amount
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn usd(a: f64) -> Stable {
        Value::new(a, "USDC")
    }

    fn passed() -> DaoRatification {
        DaoRatification {
            proposal_id: "TITHE-2026-Q3".into(),
            rate_bps: 1000,
            tally: VoteTally {
                for_votes: 40,
                against_votes: 3,
            },
        }
    }

    // ── T-1 · the tithe ───────────────────────────────────────────────────────

    #[test]
    fn a_tithe_needs_a_ratification_record() {
        let dest = GiftDestination {
            jurisdiction: Jurisdiction::Federal,
            statutory_basis: "31 U.S.C. §3113".into(),
        };
        assert!(TitheDisbursement::create(usd(1000.0), dest.clone(), passed()).is_ok());

        // an un-passed vote cannot fund a tithe
        let failed = DaoRatification {
            proposal_id: "x".into(),
            rate_bps: 1000,
            tally: VoteTally {
                for_votes: 2,
                against_votes: 40,
            },
        };
        assert_eq!(
            TitheDisbursement::create(usd(1000.0), dest.clone(), failed).unwrap_err(),
            TitheRefusal::NotRatified
        );

        // and a vote nobody cast is not ratification (Law 1a)
        let empty = DaoRatification {
            proposal_id: "y".into(),
            rate_bps: 1000,
            tally: VoteTally {
                for_votes: 0,
                against_votes: 0,
            },
        };
        assert_eq!(
            TitheDisbursement::create(usd(1000.0), dest, empty).unwrap_err(),
            TitheRefusal::NotRatified
        );
    }

    #[test]
    fn the_tithe_copy_lint_bars_both_directions() {
        // direction 1: implying it offsets tax
        assert!(!tithe_copy_findings("Tithe now and offset your tax bill.").is_empty());
        assert!(tithe_copy_findings("Contribute to the tithe — a voluntary gift.").is_empty());
        // direction 2: calling the tithe a tax
        assert!(
            tithe_copy_findings("The tithe is a tax on every deal.").contains(&"calls-tithe-a-tax")
        );
        // the permitted framing — infrastructure reciprocity — passes
        let ok = "The tithe is a voluntary gift to the treasury whose courts and customs \
                  systems this stack relies on. Your own tax obligations are unchanged.";
        assert!(
            tithe_copy_findings(ok).is_empty(),
            "the reciprocity framing must pass"
        );
    }

    // ── T-2 · the duty leg ────────────────────────────────────────────────────

    #[test]
    fn a_duty_must_match_the_accords_jurisdiction() {
        let leg = DutyLeg {
            amount: usd(500.0),
            jurisdiction: Jurisdiction::Foreign("MX".into()),
            disclosed_at_formation: true,
        };
        assert!(duty_for_accord(leg.clone(), &Jurisdiction::Foreign("MX".into())).is_ok());
        assert_eq!(
            duty_for_accord(leg, &Jurisdiction::Federal).unwrap_err(),
            DutyRefusal::JurisdictionMismatch
        );
    }

    #[test]
    fn an_undisclosed_duty_refuses() {
        let leg = DutyLeg {
            amount: usd(500.0),
            jurisdiction: Jurisdiction::Federal,
            disclosed_at_formation: false,
        };
        assert_eq!(
            duty_for_accord(leg, &Jurisdiction::Federal).unwrap_err(),
            DutyRefusal::NotDisclosedAtFormation
        );
    }

    // ── T-3 · the permission screen ───────────────────────────────────────────

    #[test]
    fn an_empty_rule_set_refuses_rather_than_clearing() {
        let empty: Vec<ProhibitionRule> = vec![];
        assert_eq!(
            screen(&empty, |_| false).unwrap_err(),
            ScreenRefusal::EmptyRuleSet
        );
    }

    #[test]
    fn a_loaded_screen_clears_or_holds() {
        let rules = vec![
            ProhibitionRule {
                id: "OFAC-123".into(),
                jurisdiction: Jurisdiction::Federal,
            },
            ProhibitionRule {
                id: "OR-corridor".into(),
                jurisdiction: Jurisdiction::State("OR".into()),
            },
        ];
        // nothing matches → clear
        assert_eq!(screen(&rules, |_| false).unwrap(), ScreenOutcome::Clear);
        // a match → hold pending licence, naming what matched
        match screen(&rules, |r| r.id == "OFAC-123").unwrap() {
            ScreenOutcome::HoldPendingLicence { matched } => assert_eq!(matched, vec!["OFAC-123"]),
            _ => panic!("a matched rule must hold"),
        }
    }

    #[test]
    fn a_held_accord_is_not_determined_never_a_pass() {
        let held = ScreenOutcome::HoldPendingLicence {
            matched: vec!["OFAC-123".into()],
        };
        let e: Eligibility<String> = screen_eligibility(&held, "export licence".into());
        assert!(matches!(e, Eligibility::NotDetermined { .. }));
        let clear: Eligibility<String> = screen_eligibility(&ScreenOutcome::Clear, "x".into());
        assert_eq!(clear, Eligibility::Meets);
    }

    // ── T-4 · the citizen's record ────────────────────────────────────────────

    #[test]
    fn the_citizen_record_has_no_reporting_path() {
        let rec = CitizenRecord {
            party: "grower".into(),
            jurisdiction: Jurisdiction::Foreign("MX".into()),
            obligations: vec![Obligation {
                description: "import duty".into(),
                amount: usd(500.0),
                remitted_by_election: false,
            }],
        };
        let j = serde_json::to_string(&rec).unwrap().to_lowercase();
        for forbidden in [
            "report_to",
            "reported",
            "filed_with",
            "government",
            "irs",
            "agency",
            "transmit",
            "submitted",
            "surveillance",
            "disclosed_to",
        ] {
            assert!(
                !j.contains(forbidden),
                "T-4 is citizen-held: no `{forbidden}` field"
            );
        }
    }

    #[test]
    fn remittance_is_an_election_defaulting_to_false() {
        let o = Obligation {
            description: "duty".into(),
            amount: usd(1.0),
            remitted_by_election: false,
        };
        assert!(
            !o.remitted_by_election,
            "remittance is an election, never a default"
        );
    }

    // ── T-5 · rails ───────────────────────────────────────────────────────────

    #[test]
    fn settlement_currency_cannot_name_fusd() {
        // the enum has no fUSD variant; a value serialises only its real variants
        for c in [
            SettlementCurrency::RegulatedStable("USDC".into()),
            SettlementCurrency::Fiat("USD".into()),
        ] {
            let j = serde_json::to_string(&c).unwrap().to_lowercase();
            assert!(
                !j.contains("fusd"),
                "fUSD is definition-only and cannot enter an executable path"
            );
        }
    }

    #[test]
    fn private_escrow_prices_at_or_above_public() {
        assert!(TierPricing {
            public_price: usd(10.0),
            private_price: usd(25.0)
        }
        .is_valid());
        assert!(!TierPricing {
            public_price: usd(25.0),
            private_price: usd(10.0)
        }
        .is_valid());
    }

    // ── b appears nowhere in the treasury ─────────────────────────────────────

    #[test]
    fn no_treasury_type_holds_b() {
        let tithe = TitheDisbursement::create(
            usd(100.0),
            GiftDestination {
                jurisdiction: Jurisdiction::Federal,
                statutory_basis: "31 U.S.C. §3113".into(),
            },
            passed(),
        )
        .unwrap();
        let duty = DutyLeg {
            amount: usd(1.0),
            jurisdiction: Jurisdiction::Federal,
            disclosed_at_formation: true,
        };
        for j in [
            serde_json::to_string(&tithe).unwrap(),
            serde_json::to_string(&duty).unwrap(),
            serde_json::to_string(&TierPricing {
                public_price: usd(1.0),
                private_price: usd(2.0),
            })
            .unwrap(),
        ] {
            let l = j.to_lowercase();
            for forbidden in [
                "\"b\"",
                "b_amount",
                "b_balance",
                "collateral",
                "mint",
                "poul",
            ] {
                assert!(
                    !l.contains(forbidden),
                    "F-3 holds no b: found `{forbidden}` — T-0 is not built here"
                );
            }
        }
    }
}
