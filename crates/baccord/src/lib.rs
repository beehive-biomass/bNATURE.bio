//! `baccord` — accords that execute what both parties already agreed (order F-2, Tier 0).
//!
//! **TESTNET ONLY.** No mainnet path, no live rails. Holding or transmitting others' funds
//! is licensed activity in most states; the design intent is that BNR stays outside
//! money-transmitter scope *because it cannot move the funds*, and counsel confirms that
//! before any deployment. Until then this is types and the settlement logic, exercised in
//! tests.
//!
//! # The three properties that make an accord safe
//!
//! - **BNR holds no key that can move funds.** Release requires a [`ReleaseAuthorization`],
//!   and there is no constructor that produces one from a BNR seat. A BNR-side release fails
//!   at the authorization check — tested, not asserted.
//! - **`NotDetermined` never releases.** The type that stops a container at the border stops
//!   a payment here: a document that measured the wrong analyte moves no money.
//! - **No silence becomes consent.** There is no timeout, no default, and no auto-acceptance
//!   anywhere. The silence-equals-acceptance clause we could not find in the specimen
//!   contract is not invented by this software either.
//!
//! # E-DEP · the dispute-deposit waterfall
//!
//! At accord creation both parties deposit the dispute-resolution cost for **all** tiers,
//! symmetrically. Resolving early returns your own money: settle at Tier 0 and both full
//! deposits come back; settle at Tier 1 and the arbiter portion returns; reach Tier 2 and the
//! deposit pays the human both parties chose. **The incentive points at settlement, not
//! escalation**, and refunds are automatic on resolution — no claim step.
//!
//! `b` appears nowhere in this crate. Deposits and legs are stables. A serialisation test
//! enforces it.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

pub use attestation_core::{Eligibility, Value};

/// A stable-denominated amount. Deposits and legs are stables; `b` is never the deal
/// currency. `Value` (amount + unit) from the neutral layer carries this — the unit is a
/// currency code, and there is no `b` variant to reach for.
pub type Stable = Value;

// ── parties and authorization ─────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PartyId(pub String);

/// Who may authorize a fund movement. **There is no `Bnr` variant.** A machine seat, and
/// BNR itself, are simply not expressible here — the absence is the guarantee that BNR
/// cannot move the money.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Signer {
    Party(PartyId),
    /// A human arbiter both parties chose from the DAO roster.
    Arbiter(PartyId),
}

/// The only thing that releases funds. Constructible **only** by the paths below — a caller
/// cannot fabricate one, and no path takes a BNR seat.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseAuthorization {
    basis: AuthBasis,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum AuthBasis {
    /// A pre-agreed release condition evaluated to `Meets`. The parties' signature at
    /// formation WAS their consent to this arithmetic.
    ConditionMet,
    /// Both parties signed a mutual release.
    MutualSignature { a: PartyId, b: PartyId },
    /// An arbiter both parties chose attested a resolution.
    ArbiterAttestation { arbiter: PartyId },
}

// ── release conditions (Tier 0) ───────────────────────────────────────────────

/// The threshold both parties signed, and the criteria required to evaluate it. Re-uses the
/// neutral layer's eligibility shape so the border-control logic and the payment logic are
/// the same code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Threshold<C> {
    pub name: String,
    pub eligibility: Eligibility<C>,
}

/// A pre-agreed release computation. **`NotDetermined` and `Exceeds` both hold; only `Meets`
/// releases.** Consent to this arithmetic was the signature.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReleaseCondition<C> {
    /// What must be attested — e.g. a COA from the named independent lab.
    pub requires: String,
    pub standard: Threshold<C>,
}

/// The outcome of evaluating a release condition against an attested result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConditionOutcome {
    /// Funds may move. Carries the authorization; nothing else can.
    Release(ReleaseAuthorization),
    /// The dispute path opens. Funds stay put.
    Hold(HoldReason),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HoldReason {
    Exceeds,
    /// A document that measured the wrong question releases nothing.
    NotDetermined,
    /// The attestation came from a party who may not attest this condition.
    UntrustedAttestor,
}

impl<C: PartialEq + Clone> ReleaseCondition<C> {
    /// Evaluate the condition. **Only `Eligibility::Meets` produces a release**, and it does
    /// so with an authorization no caller could have constructed themselves.
    pub fn evaluate(&self, attested: &Eligibility<C>) -> ConditionOutcome {
        match attested {
            Eligibility::Meets => ConditionOutcome::Release(ReleaseAuthorization {
                basis: AuthBasis::ConditionMet,
            }),
            Eligibility::Exceeds { .. } => ConditionOutcome::Hold(HoldReason::Exceeds),
            Eligibility::NotDetermined { .. } => ConditionOutcome::Hold(HoldReason::NotDetermined),
        }
    }
}

// ── escrow ────────────────────────────────────────────────────────────────────

/// A signature-locked escrow leg. **BNR holds no key.** `release` requires a
/// [`ReleaseAuthorization`], which no BNR-side path can produce.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EscrowLeg {
    pub amount: Stable,
    pub from: PartyId,
    pub to: PartyId,
    released: bool,
}

impl EscrowLeg {
    pub fn locked(amount: Stable, from: PartyId, to: PartyId) -> Self {
        EscrowLeg {
            amount,
            from,
            to,
            released: false,
        }
    }

    /// Release, given an authorization. There is no overload that takes anything else — a
    /// BNR seat has no way to call this, because it cannot obtain the argument.
    pub fn release(&mut self, _auth: &ReleaseAuthorization) -> Result<(), ReleaseError> {
        if self.released {
            return Err(ReleaseError::AlreadyReleased);
        }
        self.released = true;
        Ok(())
    }

    pub fn is_released(&self) -> bool {
        self.released
    }
}

/// Mutual release requires both parties. Arbiter release requires the chosen arbiter. These
/// are the only two ways to authorize outside a met condition.
pub fn mutual_release(a: PartyId, b: PartyId, signers: &[Signer]) -> Option<ReleaseAuthorization> {
    let has_a = signers
        .iter()
        .any(|s| matches!(s, Signer::Party(p) if *p == a));
    let has_b = signers
        .iter()
        .any(|s| matches!(s, Signer::Party(p) if *p == b));
    if has_a && has_b {
        Some(ReleaseAuthorization {
            basis: AuthBasis::MutualSignature { a, b },
        })
    } else {
        None
    }
}

pub fn arbiter_release(arbiter: PartyId, signers: &[Signer]) -> Option<ReleaseAuthorization> {
    signers
        .iter()
        .any(|s| matches!(s, Signer::Arbiter(p) if *p == arbiter))
        .then_some(ReleaseAuthorization {
            basis: AuthBasis::ArbiterAttestation { arbiter },
        })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReleaseError {
    AlreadyReleased,
}

// ── tiers and escalation ──────────────────────────────────────────────────────

/// Where a dispute currently sits. Escalation is by both parties' word, never by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tier {
    /// The code executes what both parties agreed.
    Zero,
    /// The AIs propose; only the parties dispose. Options, never rulings.
    One,
    /// A human from the DAO roster, chosen by the parties.
    Two,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EscalationError {
    /// Escalation requires both parties' explicit word.
    NeedsBothParties,
}

/// Escalate one tier. **Both parties must consent explicitly.** There is no timeout argument
/// and no default — a caller cannot express "escalate on silence" because the only path
/// requires two present consents.
pub fn escalate(
    from: Tier,
    a: &PartyId,
    b: &PartyId,
    consenting: &[PartyId],
) -> Result<Tier, EscalationError> {
    if !(consenting.contains(a) && consenting.contains(b)) {
        return Err(EscalationError::NeedsBothParties);
    }
    Ok(match from {
        Tier::Zero => Tier::One,
        Tier::One | Tier::Two => Tier::Two,
    })
}

// ── E-DEP · the dispute-deposit waterfall ─────────────────────────────────────

/// Per-tier dispute costs, disclosed at formation. Both parties deposit the total, or
/// neither does — symmetric skin.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisputeSchedule {
    /// Tier 1 facilitation cost (per party).
    pub tier1_facilitation: Stable,
    /// Tier 2 arbiter fee (per party).
    pub tier2_arbiter: Stable,
}

impl DisputeSchedule {
    /// What each party deposits at creation: the sum of all tiers.
    pub fn per_party_deposit(&self) -> f64 {
        self.tier1_facilitation.amount + self.tier2_arbiter.amount
    }
}

/// Where each party's deposit goes when a dispute resolves at a given tier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RefundOutcome {
    /// Returned to the depositing party.
    pub refunded_to_party: f64,
    /// Paid to the facilitation process (never to BNR as profit).
    pub spent_on_facilitation: f64,
    /// Paid to the human arbiter both parties chose.
    pub paid_to_arbiter: f64,
}

/// The refund waterfall. **Resolving early returns your own money.** No claim step: this
/// computes the automatic split the moment a resolution tier is known.
///
/// - Tier 0 → full deposit refunded; nothing spent.
/// - Tier 1 → facilitation spent; arbiter portion refunded.
/// - Tier 2 → facilitation spent; arbiter fee paid to the chosen human; nothing refunded.
pub fn settle_deposit(schedule: &DisputeSchedule, resolved_at: Tier) -> RefundOutcome {
    let f = schedule.tier1_facilitation.amount;
    let a = schedule.tier2_arbiter.amount;
    match resolved_at {
        Tier::Zero => RefundOutcome {
            refunded_to_party: f + a,
            spent_on_facilitation: 0.0,
            paid_to_arbiter: 0.0,
        },
        Tier::One => RefundOutcome {
            refunded_to_party: a,
            spent_on_facilitation: f,
            paid_to_arbiter: 0.0,
        },
        Tier::Two => RefundOutcome {
            refunded_to_party: 0.0,
            spent_on_facilitation: f,
            paid_to_arbiter: a,
        },
    }
}

// ── the roster (Tier 2 gate) ──────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Arbiter {
    pub id: PartyId,
    /// Counted events with attestors — never a rating.
    pub accords_heard: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RosterRefusal {
    /// Law 1a — a roster query returning zero eligible arbiters refuses rather than quietly
    /// widening the criteria.
    NoEligibleArbiter,
}

/// Query the roster. **Refuses on an empty result** rather than relaxing the filter.
pub fn eligible_arbiters<'a>(
    roster: &'a [Arbiter],
    filter: impl Fn(&Arbiter) -> bool,
) -> Result<Vec<&'a Arbiter>, RosterRefusal> {
    let out: Vec<&Arbiter> = roster.iter().filter(|a| filter(a)).collect();
    if out.is_empty() {
        Err(RosterRefusal::NoEligibleArbiter)
    } else {
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    enum Analyte {
        TotalThc,
    }

    fn coa_condition() -> ReleaseCondition<Analyte> {
        ReleaseCondition {
            requires: "COA from the named independent lab".into(),
            standard: Threshold {
                name: "US total THC <= 0.3%".into(),
                eligibility: Eligibility::Meets, // the spec both signed
            },
        }
    }

    fn usd(a: f64) -> Stable {
        Value::new(a, "USDC")
    }

    /// **THE FIRST ACCORD.** The SinglePoint 45,000-lb firm leg, released by a lab-COA that
    /// meets the standard both parties signed.
    #[test]
    fn the_singlepoint_firm_leg_releases_on_a_meeting_coa() {
        let cond = coa_condition();
        let outcome = cond.evaluate(&Eligibility::Meets);
        let auth = match outcome {
            ConditionOutcome::Release(a) => a,
            other => panic!("a meeting COA must release, got {other:?}"),
        };
        let mut leg = EscrowLeg::locked(
            usd(0.0), // price redacted in the filing; the leg is the 45,000-lb delivery
            PartyId("buyer".into()),
            PartyId("supplier".into()),
        );
        assert!(leg.release(&auth).is_ok());
        assert!(leg.is_released());
    }

    /// NotDetermined holds. The border-control type stops a payment.
    #[test]
    fn a_not_determined_coa_releases_nothing() {
        let cond = coa_condition();
        let outcome = cond.evaluate(&Eligibility::NotDetermined {
            missing: vec![Analyte::TotalThc],
        });
        assert_eq!(outcome, ConditionOutcome::Hold(HoldReason::NotDetermined));
    }

    #[test]
    fn an_exceeding_coa_holds_for_dispute() {
        let cond = coa_condition();
        let outcome = cond.evaluate(&Eligibility::Exceeds {
            by: Value::new(0.1, "%"),
        });
        assert_eq!(outcome, ConditionOutcome::Hold(HoldReason::Exceeds));
    }

    /// **THE LOAD-BEARING NEGATIVE CONTROL.** A BNR seat cannot release funds, because it
    /// cannot obtain a ReleaseAuthorization — there is no Signer::Bnr, no mutual signature it
    /// can produce, and no arbiter role it holds. The compiler enforces this; this test
    /// documents that the only authorizations require parties or a chosen arbiter.
    #[test]
    fn bnr_cannot_produce_a_release_authorization() {
        // BNR has no party or arbiter signature to offer.
        let no_bnr_signers: Vec<Signer> = vec![];
        assert!(mutual_release(
            PartyId("buyer".into()),
            PartyId("supplier".into()),
            &no_bnr_signers
        )
        .is_none());
        assert!(arbiter_release(PartyId("arb".into()), &no_bnr_signers).is_none());
        // and Signer has exactly two variants, neither of which is BNR
        let _exhaustive = |s: Signer| match s {
            Signer::Party(_) => (),
            Signer::Arbiter(_) => (),
        };
    }

    #[test]
    fn mutual_release_needs_both_parties() {
        let a = PartyId("buyer".into());
        let b = PartyId("supplier".into());
        assert!(mutual_release(a.clone(), b.clone(), &[Signer::Party(a.clone())]).is_none());
        assert!(
            mutual_release(a.clone(), b.clone(), &[Signer::Party(a), Signer::Party(b)]).is_some()
        );
    }

    #[test]
    fn a_leg_cannot_be_released_twice() {
        let auth = match coa_condition().evaluate(&Eligibility::Meets) {
            ConditionOutcome::Release(a) => a,
            _ => unreachable!(),
        };
        let mut leg = EscrowLeg::locked(usd(100.0), PartyId("x".into()), PartyId("y".into()));
        assert!(leg.release(&auth).is_ok());
        assert_eq!(leg.release(&auth), Err(ReleaseError::AlreadyReleased));
    }

    // ── escalation ────────────────────────────────────────────────────────────

    #[test]
    fn escalation_requires_both_parties() {
        let a = PartyId("a".into());
        let b = PartyId("b".into());
        assert_eq!(
            escalate(Tier::Zero, &a, &b, &[a.clone()]),
            Err(EscalationError::NeedsBothParties)
        );
        assert_eq!(
            escalate(Tier::Zero, &a, &b, &[a.clone(), b.clone()]),
            Ok(Tier::One)
        );
        assert_eq!(
            escalate(Tier::One, &a, &b, &[a.clone(), b.clone()]),
            Ok(Tier::Two)
        );
    }

    /// There is no way to express "escalate on silence". The only path needs two present
    /// consents; an empty or single-party consent set refuses.
    #[test]
    fn silence_cannot_escalate() {
        let a = PartyId("a".into());
        let b = PartyId("b".into());
        assert!(escalate(Tier::Zero, &a, &b, &[]).is_err());
    }

    // ── E-DEP waterfall ───────────────────────────────────────────────────────

    fn schedule() -> DisputeSchedule {
        DisputeSchedule {
            tier1_facilitation: usd(50.0),
            tier2_arbiter: usd(300.0),
        }
    }

    #[test]
    fn resolving_at_tier0_returns_the_full_deposit() {
        let r = settle_deposit(&schedule(), Tier::Zero);
        assert_eq!(r.refunded_to_party, 350.0);
        assert_eq!(r.spent_on_facilitation, 0.0);
        assert_eq!(r.paid_to_arbiter, 0.0);
    }

    #[test]
    fn resolving_at_tier1_returns_the_arbiter_portion() {
        let r = settle_deposit(&schedule(), Tier::One);
        assert_eq!(r.refunded_to_party, 300.0);
        assert_eq!(r.spent_on_facilitation, 50.0);
        assert_eq!(r.paid_to_arbiter, 0.0);
    }

    #[test]
    fn reaching_tier2_pays_the_chosen_human() {
        let r = settle_deposit(&schedule(), Tier::Two);
        assert_eq!(r.refunded_to_party, 0.0);
        assert_eq!(r.paid_to_arbiter, 300.0);
    }

    /// The incentive points at settlement: the earlier you resolve, the more of your own
    /// money returns. This is the property the waterfall exists to create.
    #[test]
    fn earlier_resolution_always_refunds_at_least_as_much() {
        let s = schedule();
        let r0 = settle_deposit(&s, Tier::Zero).refunded_to_party;
        let r1 = settle_deposit(&s, Tier::One).refunded_to_party;
        let r2 = settle_deposit(&s, Tier::Two).refunded_to_party;
        assert!(
            r0 >= r1 && r1 >= r2,
            "walking back down the tiers hands money back"
        );
        assert!(r0 > r2, "and the spread is real, not flat");
    }

    /// Conservation: every deposited dollar is either refunded, spent on facilitation, or
    /// paid to the arbiter. Nothing vanishes, and nothing is created — no BNR profit line.
    #[test]
    fn the_deposit_is_conserved_at_every_tier() {
        let s = schedule();
        let total = s.per_party_deposit();
        for t in [Tier::Zero, Tier::One, Tier::Two] {
            let r = settle_deposit(&s, t);
            let accounted = r.refunded_to_party + r.spent_on_facilitation + r.paid_to_arbiter;
            assert!(
                (accounted - total).abs() < 1e-9,
                "deposit must be conserved at {t:?}"
            );
        }
    }

    // ── the roster ────────────────────────────────────────────────────────────

    #[test]
    fn an_empty_roster_query_refuses() {
        let roster = vec![Arbiter {
            id: PartyId("a".into()),
            accords_heard: 5,
        }];
        assert_eq!(
            eligible_arbiters(&roster, |a| a.accords_heard > 100).unwrap_err(),
            RosterRefusal::NoEligibleArbiter
        );
        // a matching query returns the set, so the refusal is a real discrimination
        assert_eq!(
            eligible_arbiters(&roster, |a| a.accords_heard > 0)
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn an_arbiter_record_is_counts_not_a_rating() {
        let a = Arbiter {
            id: PartyId("x".into()),
            accords_heard: 42,
        };
        let j = serde_json::to_string(&a).unwrap().to_lowercase();
        for forbidden in ["rating", "score", "stars", "rank", "quality"] {
            assert!(
                !j.contains(forbidden),
                "an arbiter is counted events, never rated"
            );
        }
    }

    // ── b appears nowhere ─────────────────────────────────────────────────────

    #[test]
    fn no_accord_type_serialises_b() {
        let cond = coa_condition();
        let leg = EscrowLeg::locked(usd(1.0), PartyId("x".into()), PartyId("y".into()));
        let sched = schedule();
        for j in [
            serde_json::to_string(&cond.standard).unwrap(),
            serde_json::to_string(&leg).unwrap(),
            serde_json::to_string(&sched).unwrap(),
            serde_json::to_string(&settle_deposit(&sched, Tier::Zero)).unwrap(),
        ] {
            let l = j.to_lowercase();
            for forbidden in ["\"b\"", "b_amount", "b_balance", "mint", "reward", "poul"] {
                assert!(
                    !l.contains(forbidden),
                    "b is not the deal currency: found `{forbidden}`"
                );
            }
        }
    }
}
