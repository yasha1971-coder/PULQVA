//! Minimal PULQVA core crate bootstrap.
//!
//! Product behavior is intentionally absent at T002. This crate only proves
//! that the Rust workspace exists and is testable without network/runtime
//! dependencies.

/// Marker proving the core crate is linked and available.
pub const CORE_CRATE_READY: bool = true;

#[cfg(test)]
mod tests {
    use super::CORE_CRATE_READY;

    #[test]
    fn core_crate_smoke_test() {
        assert!(CORE_CRATE_READY);
    }
}
