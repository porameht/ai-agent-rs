//! Application layer - Use cases and orchestration.
//!
//! This module contains application services that orchestrate domain logic
//! and infrastructure. Services depend on domain ports (traits) rather than
//! concrete implementations.

pub mod services;

pub use services::{DocumentService, RagService};
