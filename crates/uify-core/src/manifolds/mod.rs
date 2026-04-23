//! Lie groups used by trackers.
//!
//! We roll these ourselves rather than depend on a third-party crate: the
//! semantics (`exp`, `log`, `⊕`, `⊖`, left vs. right trivialization) must
//! exactly match the filter code that consumes them, and owning the module
//! keeps accuracy-critical identities under one test suite.
//!
//! Every group here must satisfy the property tests in
//! `tests/manifold_identities.rs`:
//!
//! - `exp(log(x)) ≈ x` for all `x` in the group
//! - `log(exp(ξ)) ≈ ξ` for all tangent vectors `ξ`
//! - `x ⊖ x ≈ 0`
//! - `(x ⊕ (y ⊖ x)) ≈ y`
//!
//! Any new group added here must add these tests before it lands.

pub mod se2;
pub mod se3;
pub mod sl3;
pub mod so2;
pub mod so3;

/// Marker trait for a matrix Lie group with a well-defined tangent space.
pub trait LieGroup: Sized {
    /// Tangent-space element type (typically a fixed-size vector).
    type Tangent;

    /// Group identity element.
    fn identity() -> Self;

    /// Group composition: `self ∘ other`.
    fn compose(&self, other: &Self) -> Self;

    /// Group inverse.
    fn inverse(&self) -> Self;

    /// Exponential map: tangent → group.
    fn exp(xi: &Self::Tangent) -> Self;

    /// Logarithm: group → tangent.
    fn log(&self) -> Self::Tangent;

    /// Right-plus: `self ⊕ ξ = self ∘ exp(ξ)`.
    fn plus(&self, xi: &Self::Tangent) -> Self {
        self.compose(&Self::exp(xi))
    }

    /// Right-minus: `other ⊖ self = log(self⁻¹ ∘ other)`.
    fn minus(&self, other: &Self) -> Self::Tangent {
        self.inverse().compose(other).log()
    }
}
