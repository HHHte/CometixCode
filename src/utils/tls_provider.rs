//! Process-wide TLS crypto-provider initialization.
//!
//! Desktop builds use `rustls-no-provider`, so the application owns the
//! provider choice. Android uses `native-tls` and has no rustls provider to
//! install. Keeping this at one owner gives direct library and test entry
//! points the same initialization contract as the normal launcher.

/// Install the selected desktop rustls crypto provider.
///
/// The process-wide slot is naturally idempotent: subsequent calls report
/// that a provider is already installed, which is the expected result for
/// direct library/test entry points and is intentionally ignored here.
#[inline]
pub fn install_crypto_provider() {
    #[cfg(not(target_os = "android"))]
    {
        let _ = rustls::crypto::ring::default_provider().install_default();
    }
}
