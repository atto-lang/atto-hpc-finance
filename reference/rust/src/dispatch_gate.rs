//! Fail-closed host reference for the finance compute-to-venue boundary.
//!
//! This models the logical state machine only. Physical Region lengths must
//! come from stable runtime descriptors, and only the runtime scheduler may
//! certify a completion fence. Neither authority exists in this Rust model;
//! do not treat its tokens as guest-sendable or as a backend implementation.

use crate::batch_shape::{validate_batch_shape, BatchShape, BatchShapeError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathMode {
    Strict,
    Reproducible,
    Relaxed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegionLengths {
    pub bars: usize,
    pub returns: usize,
    pub rolling: usize,
    pub chunks: usize,
    pub total: usize,
    pub signal: usize,
    pub order: usize,
    pub decision: usize,
}

impl RegionLengths {
    fn agrees_with(self, shape: BatchShape) -> bool {
        self.bars == shape.bars_capacity
            && self.returns == shape.returns_capacity
            && self.rolling == shape.rolling_capacity
            && self.chunks == shape.chunks_capacity
            && self.total == shape.total_capacity
            && self.signal == shape.signal_capacity
            && self.order == shape.order_capacity
            && self.decision == shape.decision_capacity
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ticket {
    pub batch_id: u64,
    pub generation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RiskDecision {
    pub approved: bool,
    pub reason_code: i64,
}

#[derive(Debug, Clone, Copy)]
pub struct OrderIntent {
    pub side: i64,
    pub quantity: f64,
    pub limit_price: f64,
    pub max_quantity: f64,
}

impl OrderIntent {
    fn valid(self) -> bool {
        (self.side == 1 || self.side == -1)
            && self.quantity.is_finite()
            && self.quantity > 0.0
            && self.limit_price.is_finite()
            && self.limit_price > 0.0
            && self.max_quantity.is_finite()
            && self.max_quantity >= 0.0
            && self.quantity <= self.max_quantity
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchError {
    InvalidBatch(BatchShapeError),
    UntrustedCapacityMismatch,
    UnsafeMathMode,
    Busy,
    GenerationOverflow,
    NoActiveTask,
    WrongTicket,
    NotStaged,
    NotFenced,
    Denied,
    InvalidOrder,
}

#[derive(Debug, Clone, Copy)]
enum Phase {
    Idle,
    Pending(Ticket),
    Staged(Ticket, RiskDecision),
    Fenced(Ticket, RiskDecision),
}

/// Reference-only state machine. The production runtime must provide a
/// non-forgeable descriptor reader, fence receipt, owner lease and scheduler
/// handoff; these model calls alone prove none of those capabilities.
pub struct DispatchGate {
    generation: u64,
    phase: Phase,
}

impl Default for DispatchGate {
    fn default() -> Self {
        Self::new()
    }
}

impl DispatchGate {
    pub fn new() -> Self {
        Self {
            generation: 0,
            phase: Phase::Idle,
        }
    }

    /// `actual` represents an independently read, stable runtime descriptor.
    /// The model rejects untrusted capacity claims differing from that read.
    pub fn begin(
        &mut self,
        batch_id: u64,
        claimed: BatchShape,
        actual: RegionLengths,
        math: MathMode,
    ) -> Result<Ticket, DispatchError> {
        if !matches!(self.phase, Phase::Idle) {
            return Err(DispatchError::Busy);
        }
        if math != MathMode::Strict {
            return Err(DispatchError::UnsafeMathMode);
        }
        if !actual.agrees_with(claimed) {
            return Err(DispatchError::UntrustedCapacityMismatch);
        }
        validate_batch_shape(claimed).map_err(DispatchError::InvalidBatch)?;
        let generation = self
            .generation
            .checked_add(1)
            .ok_or(DispatchError::GenerationOverflow)?;
        self.generation = generation;
        let ticket = Ticket {
            batch_id,
            generation,
        };
        // No old risk decision can be read from the next generation.
        self.phase = Phase::Pending(ticket);
        Ok(ticket)
    }

    /// Model a completed risk-kernel write. This is *not* a fence.
    pub fn stage_decision(
        &mut self,
        ticket: Ticket,
        decision: RiskDecision,
    ) -> Result<(), DispatchError> {
        match self.phase {
            Phase::Pending(current) if current == ticket => {
                self.phase = Phase::Staged(ticket, decision);
                Ok(())
            }
            Phase::Pending(_) | Phase::Staged(_, _) | Phase::Fenced(_, _) => {
                Err(DispatchError::WrongTicket)
            }
            Phase::Idle => Err(DispatchError::NoActiveTask),
        }
    }

    /// In production, only a release/acquire synchronized completion
    /// callback on the owning scheduler may call the corresponding gate.
    pub fn record_fence(&mut self, ticket: Ticket) -> Result<(), DispatchError> {
        match self.phase {
            Phase::Staged(current, decision) if current == ticket => {
                self.phase = Phase::Fenced(ticket, decision);
                Ok(())
            }
            Phase::Staged(_, _) | Phase::Fenced(_, _) | Phase::Pending(_) => {
                Err(DispatchError::NotStaged)
            }
            Phase::Idle => Err(DispatchError::NoActiveTask),
        }
    }

    pub fn cancel(&mut self, ticket: Ticket) -> Result<(), DispatchError> {
        match self.phase {
            Phase::Pending(current)
            | Phase::Staged(current, _)
            | Phase::Fenced(current, _)
                if current == ticket =>
            {
                self.phase = Phase::Idle;
                Ok(())
            }
            Phase::Idle => Err(DispatchError::NoActiveTask),
            _ => Err(DispatchError::WrongTicket),
        }
    }

    /// A one-use mock permit is issued only after a matching fence, a
    /// positive risk decision and a *second* terminal-order check.
    pub fn authorize(
        &mut self,
        ticket: Ticket,
        order: OrderIntent,
    ) -> Result<ExecutionPermit, DispatchError> {
        match self.phase {
            Phase::Fenced(current, decision) if current == ticket => {
                // Consume the decision on either success or failure.
                self.phase = Phase::Idle;
                if !decision.approved || decision.reason_code != 0 {
                    return Err(DispatchError::Denied);
                }
                if !order.valid() {
                    return Err(DispatchError::InvalidOrder);
                }
                Ok(ExecutionPermit { ticket })
            }
            Phase::Pending(current) | Phase::Staged(current, _) if current == ticket => {
                Err(DispatchError::NotFenced)
            }
            Phase::Idle => Err(DispatchError::NoActiveTask),
            _ => Err(DispatchError::WrongTicket),
        }
    }
}

/// Opaque, non-Clone token consumed by the reference mock venue.
#[derive(Debug)]
pub struct ExecutionPermit {
    ticket: Ticket,
}

#[derive(Default)]
pub struct MockVenue {
    pub calls: usize,
}

impl MockVenue {
    pub fn submit(&mut self, permit: ExecutionPermit) -> Ticket {
        self.calls += 1;
        permit.ticket
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shape() -> BatchShape {
        BatchShape {
            bar_count: 4,
            return_count: 3,
            rolling_count: 2,
            window: 2,
            chunk_size: 2,
            chunk_count: 2,
            bars_capacity: 4,
            returns_capacity: 3,
            rolling_capacity: 2,
            chunks_capacity: 2,
            total_capacity: 1,
            signal_capacity: 1,
            order_capacity: 1,
            decision_capacity: 1,
        }
    }

    fn lengths() -> RegionLengths {
        RegionLengths {
            bars: 4,
            returns: 3,
            rolling: 2,
            chunks: 2,
            total: 1,
            signal: 1,
            order: 1,
            decision: 1,
        }
    }

    fn order() -> OrderIntent {
        OrderIntent {
            side: 1,
            quantity: 2.0,
            limit_price: 25.0,
            max_quantity: 5.0,
        }
    }

    fn approved() -> RiskDecision {
        RiskDecision {
            approved: true,
            reason_code: 0,
        }
    }

    #[test]
    fn strict_math_only_and_no_submission_on_bad_extent() {
        let mut gate = DispatchGate::new();
        assert_eq!(
            gate.begin(1, shape(), lengths(), MathMode::Relaxed),
            Err(DispatchError::UnsafeMathMode)
        );
        assert_eq!(
            gate.begin(1, shape(), lengths(), MathMode::Reproducible),
            Err(DispatchError::UnsafeMathMode)
        );
        let mut bad = lengths();
        bad.decision = 0;
        assert_eq!(
            gate.begin(1, shape(), bad, MathMode::Strict),
            Err(DispatchError::UntrustedCapacityMismatch)
        );
        assert_eq!(gate.begin(1, shape(), lengths(), MathMode::Strict).unwrap().generation, 1);
    }

    #[test]
    fn shape_error_is_distinct_from_descriptor_mismatch() {
        let mut gate = DispatchGate::new();
        let mut bad = shape();
        bad.window = 0;
        assert_eq!(
            gate.begin(2, bad, lengths(), MathMode::Strict),
            Err(DispatchError::InvalidBatch(BatchShapeError::InvalidWindow))
        );
    }

    #[test]
    fn pending_or_staged_result_never_reaches_venue() {
        let mut gate = DispatchGate::new();
        let mut venue = MockVenue::default();
        let ticket = gate.begin(1, shape(), lengths(), MathMode::Strict).unwrap();
        assert!(matches!(gate.authorize(ticket, order()), Err(DispatchError::NotFenced)));
        assert_eq!(gate.record_fence(ticket), Err(DispatchError::NotStaged));
        gate.stage_decision(ticket, approved()).unwrap();
        assert!(matches!(gate.authorize(ticket, order()), Err(DispatchError::NotFenced)));
        assert_eq!(venue.calls, 0);
        gate.record_fence(ticket).unwrap();
        let permit = gate.authorize(ticket, order()).unwrap();
        assert_eq!(venue.submit(permit), ticket);
        assert_eq!(venue.calls, 1);
        assert!(matches!(gate.authorize(ticket, order()), Err(DispatchError::NoActiveTask)));
    }

    #[test]
    fn rejected_decision_must_not_reach_venue() {
        let mut gate = DispatchGate::new();
        let venue = MockVenue::default();
        let ticket = gate.begin(2, shape(), lengths(), MathMode::Strict).unwrap();
        gate.stage_decision(ticket, RiskDecision { approved: false, reason_code: 1 }).unwrap();
        gate.record_fence(ticket).unwrap();
        assert!(matches!(gate.authorize(ticket, order()), Err(DispatchError::Denied)));
        assert_eq!(venue.calls, 0);
    }

    #[test]
    fn inconsistent_approval_and_bad_order_are_denied() {
        let mut gate = DispatchGate::new();
        let venue = MockVenue::default();
        let ticket = gate.begin(3, shape(), lengths(), MathMode::Strict).unwrap();
        gate.stage_decision(ticket, RiskDecision { approved: true, reason_code: 4 }).unwrap();
        gate.record_fence(ticket).unwrap();
        assert!(matches!(gate.authorize(ticket, order()), Err(DispatchError::Denied)));
        let ticket = gate.begin(4, shape(), lengths(), MathMode::Strict).unwrap();
        gate.stage_decision(ticket, approved()).unwrap();
        gate.record_fence(ticket).unwrap();
        assert!(matches!(gate.authorize(ticket, OrderIntent { quantity: f64::NAN, ..order() }), Err(DispatchError::InvalidOrder)));
        assert_eq!(venue.calls, 0);
    }

    #[test]
    fn invalid_terminal_order_variants_all_deny() {
        let cases = [
            OrderIntent { side: 0, ..order() },
            OrderIntent { side: 2, ..order() },
            OrderIntent { side: 257, ..order() },
            OrderIntent { quantity: 0.0, ..order() },
            OrderIntent { quantity: -1.0, ..order() },
            OrderIntent { quantity: f64::INFINITY, ..order() },
            OrderIntent { quantity: 8.0, ..order() },
            OrderIntent { limit_price: 0.0, ..order() },
            OrderIntent { limit_price: f64::NEG_INFINITY, ..order() },
            OrderIntent { max_quantity: f64::NAN, ..order() },
            OrderIntent { max_quantity: -1.0, ..order() },
        ];
        let mut gate = DispatchGate::new();
        let venue = MockVenue::default();
        for (i, bad_order) in cases.into_iter().enumerate() {
            let ticket = gate.begin(i as u64, shape(), lengths(), MathMode::Strict).unwrap();
            gate.stage_decision(ticket, approved()).unwrap();
            gate.record_fence(ticket).unwrap();
            assert!(matches!(gate.authorize(ticket, bad_order), Err(DispatchError::InvalidOrder)));
        }
        assert_eq!(venue.calls, 0);
    }

    #[test]
    fn stale_completion_cannot_authorize_reused_batch_id() {
        let mut gate = DispatchGate::new();
        let first = gate.begin(10, shape(), lengths(), MathMode::Strict).unwrap();
        gate.stage_decision(first, approved()).unwrap();
        gate.record_fence(first).unwrap();
        gate.cancel(first).unwrap();
        let second = gate.begin(10, shape(), lengths(), MathMode::Strict).unwrap();
        assert_ne!(first, second);
        assert_eq!(gate.stage_decision(first, approved()), Err(DispatchError::WrongTicket));
        assert!(matches!(gate.authorize(first, order()), Err(DispatchError::WrongTicket)));
        assert_eq!(gate.record_fence(first), Err(DispatchError::NotStaged));
        assert!(matches!(gate.authorize(second, order()), Err(DispatchError::NotFenced)));
        gate.stage_decision(second, approved()).unwrap();
        gate.record_fence(second).unwrap();
        assert!(gate.authorize(second, order()).is_ok());
    }

    #[test]
    fn cancellation_invalidates_staged_decision_and_late_completion() {
        let mut gate = DispatchGate::new();
        let ticket = gate.begin(7, shape(), lengths(), MathMode::Strict).unwrap();
        gate.stage_decision(ticket, approved()).unwrap();
        gate.cancel(ticket).unwrap();
        assert_eq!(gate.record_fence(ticket), Err(DispatchError::NoActiveTask));
        assert!(matches!(gate.authorize(ticket, order()), Err(DispatchError::NoActiveTask)));
    }

    #[test]
    fn second_submission_waits_until_terminal_consumption() {
        let mut gate = DispatchGate::new();
        let first = gate.begin(1, shape(), lengths(), MathMode::Strict).unwrap();
        assert_eq!(gate.begin(2, shape(), lengths(), MathMode::Strict), Err(DispatchError::Busy));
        gate.cancel(first).unwrap();
        assert_eq!(gate.begin(2, shape(), lengths(), MathMode::Strict).unwrap().generation, 2);
    }
}
