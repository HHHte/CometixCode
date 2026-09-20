//! OAuth service owners mirrored from CC `services/oauth/`.
//!
//! The live token-refresh slice is source-backed. Login, profile enrichment,
//! account population, and authorization-code flows remain explicit partial
//! seams. Source-shaped preparation stays available while the single
//! user-authorized L2 constant in `constants/oauth.rs` keeps final OAuth
//! credential side effects default-closed at their source-owned outlets.

pub mod client;
