//! Peer address parsing.
//!
//! Maps to: CC `utils/peerAddress.ts`. Kept separate from peer-registry state
//! for the same reason as the official module: SendMessageTool needs
//! `parse_address` at tool-enumeration time without pulling in bridge or UDS
//! transports.

/// Maps to: CC `utils/peerAddress.ts:9-11` `parseAddress(...)` scheme union.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PeerAddressScheme {
    Uds,
    Bridge,
    Other,
}

/// Maps to: CC `utils/peerAddress.ts:8-21` `parseAddress(...)` return shape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeerAddress {
    pub scheme: PeerAddressScheme,
    pub target: String,
}

/// Maps to: CC `utils/peerAddress.ts:8-21` `parseAddress(to)`.
pub fn parse_address(to: &str) -> PeerAddress {
    if let Some(target) = to.strip_prefix("uds:") {
        return PeerAddress {
            scheme: PeerAddressScheme::Uds,
            target: target.to_string(),
        };
    }
    if let Some(target) = to.strip_prefix("bridge:") {
        return PeerAddress {
            scheme: PeerAddressScheme::Bridge,
            target: target.to_string(),
        };
    }
    // Legacy: old-code UDS senders emit bare socket paths in `from=`; route
    // them through the UDS branch so replies aren't dropped into teammate
    // routing.
    if to.starts_with('/') {
        return PeerAddress {
            scheme: PeerAddressScheme::Uds,
            target: to.to_string(),
        };
    }
    PeerAddress {
        scheme: PeerAddressScheme::Other,
        target: to.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_address_splits_official_schemes_and_targets() {
        assert_eq!(
            parse_address("uds:/tmp/sock"),
            PeerAddress {
                scheme: PeerAddressScheme::Uds,
                target: "/tmp/sock".to_string(),
            }
        );
        assert_eq!(
            parse_address("bridge:abc123"),
            PeerAddress {
                scheme: PeerAddressScheme::Bridge,
                target: "abc123".to_string(),
            }
        );
        assert_eq!(
            parse_address("/tmp/legacy.sock"),
            PeerAddress {
                scheme: PeerAddressScheme::Uds,
                target: "/tmp/legacy.sock".to_string(),
            }
        );
        assert_eq!(
            parse_address("researcher"),
            PeerAddress {
                scheme: PeerAddressScheme::Other,
                target: "researcher".to_string(),
            }
        );
    }

    /// CC keeps the empty target rather than falling through to `other`; the
    /// tool's `validateInput` is what rejects it.
    #[test]
    fn parse_address_keeps_empty_targets_for_the_tool_validator() {
        assert_eq!(
            parse_address("bridge:"),
            PeerAddress {
                scheme: PeerAddressScheme::Bridge,
                target: String::new(),
            }
        );
        assert_eq!(
            parse_address("uds:"),
            PeerAddress {
                scheme: PeerAddressScheme::Uds,
                target: String::new(),
            }
        );
    }
}
