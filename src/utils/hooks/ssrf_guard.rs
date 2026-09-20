//! SSRF guard for HTTP hooks.
//! Maps to: CC `utils/hooks/ssrfGuard.ts`.
//!
//! HTTP hook execution (`execHttpHook.ts`) uses this guard to prevent
//! project-configured hook URLs from reaching private/link-local infrastructure
//! or cloud metadata endpoints. Loopback remains allowed for local development,
//! matching Claude Code.

use std::net::{IpAddr, ToSocketAddrs};

/// Maps to: CC `isBlockedAddress(address)`.
pub fn is_blocked_address(address: &str) -> bool {
    match address.parse::<IpAddr>() {
        Ok(IpAddr::V4(address)) => is_blocked_v4(address.octets()),
        Ok(IpAddr::V6(address)) => {
            if let Some(mapped) = address.to_ipv4_mapped() {
                return is_blocked_v4(mapped.octets());
            }
            is_blocked_v6(address.segments())
        }
        Err(_) => false,
    }
}

/// Maps to: CC `isBlockedV4(address)`.
fn is_blocked_v4(octets: [u8; 4]) -> bool {
    let [a, b, _, _] = octets;

    // Loopback explicitly allowed.
    if a == 127 {
        return false;
    }

    // 0.0.0.0/8, 10.0.0.0/8, 169.254.0.0/16, 172.16.0.0/12,
    // 100.64.0.0/10, 192.168.0.0/16.
    a == 0
        || a == 10
        || (a == 169 && b == 254)
        || (a == 172 && (16..=31).contains(&b))
        || (a == 100 && (64..=127).contains(&b))
        || (a == 192 && b == 168)
}

/// Maps to: CC `isBlockedV6(address)`.
fn is_blocked_v6(segments: [u16; 8]) -> bool {
    // ::1 loopback explicitly allowed.
    if segments == [0, 0, 0, 0, 0, 0, 0, 1] {
        return false;
    }

    // :: unspecified.
    if segments == [0; 8] {
        return true;
    }

    let first = segments[0];

    // fc00::/7 unique local.
    if (first & 0xfe00) == 0xfc00 {
        return true;
    }

    // fe80::/10 link-local.
    if (first & 0xffc0) == 0xfe80 {
        return true;
    }

    false
}

/// Error object projection for blocked addresses.
/// Maps to: CC `ssrfError(hostname, address)` message and code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SsrfGuardError {
    pub code: &'static str,
    pub hostname: String,
    pub address: String,
    pub message: String,
}

impl std::fmt::Display for SsrfGuardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for SsrfGuardError {}

/// Maps to: CC `ssrfError(hostname, address)`.
pub fn ssrf_error(hostname: &str, address: &str) -> SsrfGuardError {
    SsrfGuardError {
        code: "ERR_HTTP_HOOK_BLOCKED_ADDRESS",
        hostname: hostname.to_string(),
        address: address.to_string(),
        message: format!(
            "HTTP hook blocked: {hostname} resolves to {address} (private/link-local address). Loopback (127.0.0.1, ::1) is allowed for local dev."
        ),
    }
}

/// Rust equivalent of CC `ssrfGuardedLookup(...)`.
///
/// Node passes this as Axios's `lookup` callback so the socket uses the same
/// IPs that were validated. Rust callers can use the returned validated address
/// list when constructing HTTP clients. IP literals are validated directly;
/// hostnames are resolved with the platform resolver and every result is
/// checked before any address is returned.
pub fn ssrf_guarded_lookup(hostname: &str) -> Result<Vec<IpAddr>, SsrfGuardError> {
    if let Ok(ip) = hostname.parse::<IpAddr>() {
        let address = ip.to_string();
        if is_blocked_address(&address) {
            return Err(ssrf_error(hostname, &address));
        }
        return Ok(vec![ip]);
    }

    let addresses = (hostname, 0)
        .to_socket_addrs()
        .map_err(|error| SsrfGuardError {
            code: "ENOTFOUND",
            hostname: hostname.to_string(),
            address: String::new(),
            message: format!("ENOTFOUND {hostname}: {error}"),
        })?
        .map(|socket| socket.ip())
        .collect::<Vec<_>>();

    if addresses.is_empty() {
        return Err(SsrfGuardError {
            code: "ENOTFOUND",
            hostname: hostname.to_string(),
            address: String::new(),
            message: format!("ENOTFOUND {hostname}"),
        });
    }

    for address in &addresses {
        let address_text = address.to_string();
        if is_blocked_address(&address_text) {
            return Err(ssrf_error(hostname, &address_text));
        }
    }

    Ok(addresses)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_official_ipv4_private_linklocal_and_metadata_ranges() {
        for address in [
            "0.0.0.0",
            "10.1.2.3",
            "169.254.169.254",
            "172.16.0.1",
            "172.31.255.255",
            "100.64.0.1",
            "100.100.100.200",
            "192.168.1.1",
        ] {
            assert!(is_blocked_address(address), "{address} should be blocked");
        }

        for address in ["127.0.0.1", "8.8.8.8", "172.32.0.1", "100.128.0.1"] {
            assert!(!is_blocked_address(address), "{address} should be allowed");
        }
    }

    #[test]
    fn blocks_official_ipv6_private_linklocal_unspecified_and_mapped_ranges() {
        for address in [
            "::",
            "fc00::1",
            "fdff::1",
            "fe80::1",
            "febf::1",
            "::ffff:169.254.169.254",
            "::ffff:a9fe:a9fe",
            "0:0:0:0:0:ffff:c0a8:0101",
        ] {
            assert!(is_blocked_address(address), "{address} should be blocked");
        }

        for address in ["::1", "2001:4860:4860::8888", "fec0::1", "::ffff:127.0.0.1"] {
            assert!(!is_blocked_address(address), "{address} should be allowed");
        }
    }

    #[test]
    fn invalid_ip_literals_are_not_blocked_by_literal_helper() {
        assert!(!is_blocked_address("example.com"));
        assert!(!is_blocked_address("not an ip"));
    }

    #[test]
    fn guarded_lookup_validates_ip_literals_without_dns() {
        let loopback = ssrf_guarded_lookup("127.0.0.1").expect("loopback allowed");
        assert_eq!(loopback.len(), 1);

        let error = ssrf_guarded_lookup("169.254.169.254").expect_err("metadata blocked");
        assert_eq!(error.code, "ERR_HTTP_HOOK_BLOCKED_ADDRESS");
        assert!(
            error
                .message
                .contains("Loopback (127.0.0.1, ::1) is allowed")
        );
    }
}
