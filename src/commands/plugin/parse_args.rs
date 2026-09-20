//! Maps to: CC `commands/plugin/parseArgs.ts`.

use serde::Serialize;

/// Maps to: CC `commands/plugin/parseArgs.ts:2-15#ParsedCommand`.
/// Optional strings retain `Some("")`: JS distinguishes it from undefined.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ParsedCommand {
    Menu,
    Help,
    Install {
        #[serde(skip_serializing_if = "Option::is_none")]
        marketplace: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        plugin: Option<String>,
    },
    Manage,
    Uninstall {
        #[serde(skip_serializing_if = "Option::is_none")]
        plugin: Option<String>,
    },
    Enable {
        #[serde(skip_serializing_if = "Option::is_none")]
        plugin: Option<String>,
    },
    Disable {
        #[serde(skip_serializing_if = "Option::is_none")]
        plugin: Option<String>,
    },
    Validate {
        #[serde(skip_serializing_if = "Option::is_none")]
        path: Option<String>,
    },
    Marketplace {
        #[serde(skip_serializing_if = "Option::is_none")]
        action: Option<MarketplaceAction>,
        #[serde(skip_serializing_if = "Option::is_none")]
        target: Option<String>,
    },
}

/// Rust discriminant carrier for the inline action union in CC
/// `commands/plugin/parseArgs.ts:13` (no separate source definition).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MarketplaceAction {
    Add,
    Remove,
    Update,
    List,
}

/// Maps to: CC `commands/plugin/parseArgs.ts:17-103#parsePluginArgs`.
pub fn parse_plugin_args(args: Option<&str>) -> ParsedCommand {
    let Some(args) = args.filter(|args| !args.is_empty()) else {
        return ParsedCommand::Menu;
    };

    // L1 ECMAScript string/regex carrier: trim and /\s+/ use the same
    // WhiteSpace + LineTerminator set; Rust adds U+0085 and omits FEFF.
    let whitespace = |ch: char| (ch.is_whitespace() && ch != '\u{85}') || ch == '\u{feff}';
    let parts: Vec<_> = args
        .trim_matches(whitespace)
        .split(whitespace)
        .filter(|part| !part.is_empty())
        .collect();
    let command = parts.first().copied().unwrap_or("").to_lowercase();

    match command.as_str() {
        "help" | "--help" | "-h" => ParsedCommand::Help,
        "install" | "i" => {
            let Some(target) = parts.get(1).copied() else {
                return ParsedCommand::Install {
                    marketplace: None,
                    plugin: None,
                };
            };
            if target.contains('@') {
                let mut segments = target.split('@');
                let plugin = segments.next().map(str::to_string);
                let marketplace = segments.next().map(str::to_string);
                return ParsedCommand::Install {
                    plugin,
                    marketplace,
                };
            }
            let is_marketplace = target.starts_with("http://")
                || target.starts_with("https://")
                || target.starts_with("file://")
                || target.contains('/')
                || target.contains('\\');
            if is_marketplace {
                ParsedCommand::Install {
                    marketplace: Some(target.to_string()),
                    plugin: None,
                }
            } else {
                ParsedCommand::Install {
                    marketplace: None,
                    plugin: Some(target.to_string()),
                }
            }
        }
        "manage" => ParsedCommand::Manage,
        "uninstall" => ParsedCommand::Uninstall {
            plugin: parts.get(1).map(|part| (*part).to_string()),
        },
        "enable" => ParsedCommand::Enable {
            plugin: parts.get(1).map(|part| (*part).to_string()),
        },
        "disable" => ParsedCommand::Disable {
            plugin: parts.get(1).map(|part| (*part).to_string()),
        },
        "validate" => {
            let path = parts[1..].join(" ");
            let path = path.trim_matches(whitespace);
            ParsedCommand::Validate {
                path: (!path.is_empty()).then(|| path.to_string()),
            }
        }
        "marketplace" | "market" => {
            let action = parts.get(1).copied().unwrap_or("").to_lowercase();
            let target = parts.get(2..).unwrap_or_default().join(" ");
            let action = match action.as_str() {
                "add" => Some(MarketplaceAction::Add),
                "remove" | "rm" => Some(MarketplaceAction::Remove),
                "update" => Some(MarketplaceAction::Update),
                "list" => Some(MarketplaceAction::List),
                _ => None,
            };
            ParsedCommand::Marketplace {
                target: action
                    .filter(|action| *action != MarketplaceAction::List)
                    .map(|_| target),
                action,
            }
        }
        _ => ParsedCommand::Menu,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_plugin_args_matches_official_bun_oracle() {
        // CC commands/plugin/parseArgs.ts:17-103, actual Bun import.
        // Captured by research/proof/plugin-foundation-0913/parse-args-oracle.ts.
        // The JSON is inline so the tests do not depend on task artifacts.
        let cases: serde_json::Value = serde_json::from_str(r#"[
            {"input":null,"expected":{"type":"menu"}},
            {"input":"","expected":{"type":"menu"}},
            {"input":"   ","expected":{"type":"menu"}},
            {"input":"help","expected":{"type":"help"}},
            {"input":"--help","expected":{"type":"help"}},
            {"input":"-h","expected":{"type":"help"}},
            {"input":" HeLP ignored ","expected":{"type":"help"}},
            {"input":"install","expected":{"type":"install"}},
            {"input":"i","expected":{"type":"install"}},
            {"input":"install thing","expected":{"type":"install","plugin":"thing"}},
            {"input":"install thing ignored","expected":{"type":"install","plugin":"thing"}},
            {"input":"install thing@market","expected":{"type":"install","plugin":"thing","marketplace":"market"}},
            {"input":"install @market","expected":{"type":"install","plugin":"","marketplace":"market"}},
            {"input":"install thing@","expected":{"type":"install","plugin":"thing","marketplace":""}},
            {"input":"install @","expected":{"type":"install","plugin":"","marketplace":""}},
            {"input":"install thing@market@ignored","expected":{"type":"install","plugin":"thing","marketplace":"market"}},
            {"input":"install https://example.test/market","expected":{"type":"install","marketplace":"https://example.test/market"}},
            {"input":"install file:///tmp/market","expected":{"type":"install","marketplace":"file:///tmp/market"}},
            {"input":"install ./market","expected":{"type":"install","marketplace":"./market"}},
            {"input":"install owner/repo","expected":{"type":"install","marketplace":"owner/repo"}},
            {"input":"install C:\\market","expected":{"type":"install","marketplace":"C:\\market"}},
            {"input":"install https://user@example.test/market","expected":{"type":"install","plugin":"https://user","marketplace":"example.test/market"}},
            {"input":"manage","expected":{"type":"manage"}},
            {"input":"manage extra","expected":{"type":"manage"}},
            {"input":"uninstall","expected":{"type":"uninstall"}},
            {"input":"uninstall thing@market extra","expected":{"type":"uninstall","plugin":"thing@market"}},
            {"input":"enable","expected":{"type":"enable"}},
            {"input":"enable thing","expected":{"type":"enable","plugin":"thing"}},
            {"input":"disable","expected":{"type":"disable"}},
            {"input":"disable thing@market","expected":{"type":"disable","plugin":"thing@market"}},
            {"input":"validate","expected":{"type":"validate"}},
            {"input":"validate /path with   spaces","expected":{"type":"validate","path":"/path with spaces"}},
            {"input":"validate \"quoted path\"","expected":{"type":"validate","path":"\"quoted path\""}},
            {"input":"marketplace","expected":{"type":"marketplace"}},
            {"input":"market","expected":{"type":"marketplace"}},
            {"input":"marketplace add","expected":{"type":"marketplace","action":"add","target":""}},
            {"input":"marketplace ADD ./a  b","expected":{"type":"marketplace","action":"add","target":"./a b"}},
            {"input":"marketplace remove","expected":{"type":"marketplace","action":"remove","target":""}},
            {"input":"market rm x y","expected":{"type":"marketplace","action":"remove","target":"x y"}},
            {"input":"marketplace update","expected":{"type":"marketplace","action":"update","target":""}},
            {"input":"marketplace list ignored","expected":{"type":"marketplace","action":"list"}},
            {"input":"marketplace unknown extra","expected":{"type":"marketplace"}},
            {"input":"unknown","expected":{"type":"menu"}},
            {"input":"INSTALL \u4e2d\u6587\ud83c\udf19","expected":{"type":"install","plugin":"\u4e2d\u6587\ud83c\udf19"}},
            {"input":"install thing\u0085tail","expected":{"type":"install","plugin":"thing\u0085tail"}},
            {"input":"\u0085help","expected":{"type":"menu"}},
            {"input":"\ufeffhelp\ufeff","expected":{"type":"help"}},
            {"input":"\tINSTALL\tfoo@bar\t","expected":{"type":"install","plugin":"foo","marketplace":"bar"}},
            {"input":"\nINSTALL\nfoo@bar\n","expected":{"type":"install","plugin":"foo","marketplace":"bar"}},
            {"input":"\u000bINSTALL\u000bfoo@bar\u000b","expected":{"type":"install","plugin":"foo","marketplace":"bar"}},
            {"input":"\fINSTALL\ffoo@bar\f","expected":{"type":"install","plugin":"foo","marketplace":"bar"}},
            {"input":"\rINSTALL\rfoo@bar\r","expected":{"type":"install","plugin":"foo","marketplace":"bar"}},
            {"input":" INSTALL foo@bar ","expected":{"type":"install","plugin":"foo","marketplace":"bar"}},
            {"input":"\u00a0INSTALL\u00a0foo@bar\u00a0","expected":{"type":"install","plugin":"foo","marketplace":"bar"}},
            {"input":"\u1680INSTALL\u1680foo@bar\u1680","expected":{"type":"install","plugin":"foo","marketplace":"bar"}},
            {"input":"\u2000INSTALL\u2000foo@bar\u2000","expected":{"type":"install","plugin":"foo","marketplace":"bar"}},
            {"input":"\u2001INSTALL\u2001foo@bar\u2001","expected":{"type":"install","plugin":"foo","marketplace":"bar"}},
            {"input":"\u2002INSTALL\u2002foo@bar\u2002","expected":{"type":"install","plugin":"foo","marketplace":"bar"}},
            {"input":"\u2003INSTALL\u2003foo@bar\u2003","expected":{"type":"install","plugin":"foo","marketplace":"bar"}},
            {"input":"\u2004INSTALL\u2004foo@bar\u2004","expected":{"type":"install","plugin":"foo","marketplace":"bar"}},
            {"input":"\u2005INSTALL\u2005foo@bar\u2005","expected":{"type":"install","plugin":"foo","marketplace":"bar"}},
            {"input":"\u2006INSTALL\u2006foo@bar\u2006","expected":{"type":"install","plugin":"foo","marketplace":"bar"}},
            {"input":"\u2007INSTALL\u2007foo@bar\u2007","expected":{"type":"install","plugin":"foo","marketplace":"bar"}},
            {"input":"\u2008INSTALL\u2008foo@bar\u2008","expected":{"type":"install","plugin":"foo","marketplace":"bar"}},
            {"input":"\u2009INSTALL\u2009foo@bar\u2009","expected":{"type":"install","plugin":"foo","marketplace":"bar"}},
            {"input":"\u200aINSTALL\u200afoo@bar\u200a","expected":{"type":"install","plugin":"foo","marketplace":"bar"}},
            {"input":"\u2028INSTALL\u2028foo@bar\u2028","expected":{"type":"install","plugin":"foo","marketplace":"bar"}},
            {"input":"\u2029INSTALL\u2029foo@bar\u2029","expected":{"type":"install","plugin":"foo","marketplace":"bar"}},
            {"input":"\u202fINSTALL\u202ffoo@bar\u202f","expected":{"type":"install","plugin":"foo","marketplace":"bar"}},
            {"input":"\u205fINSTALL\u205ffoo@bar\u205f","expected":{"type":"install","plugin":"foo","marketplace":"bar"}},
            {"input":"\u3000INSTALL\u3000foo@bar\u3000","expected":{"type":"install","plugin":"foo","marketplace":"bar"}},
            {"input":"\ufeffINSTALL\ufefffoo@bar\ufeff","expected":{"type":"install","plugin":"foo","marketplace":"bar"}},
            {"input":"install\u0085foo@bar","expected":{"type":"menu"}},
            {"input":"install\u180efoo@bar","expected":{"type":"menu"}},
            {"input":"install\u200bfoo@bar","expected":{"type":"menu"}},
            {"input":"install\u001cfoo@bar","expected":{"type":"menu"}}
        ]"#).unwrap();
        for case in cases.as_array().unwrap() {
            let actual = parse_plugin_args(case["input"].as_str());
            assert_eq!(
                serde_json::to_value(actual).unwrap(),
                case["expected"],
                "input: {:?}",
                case["input"]
            );
        }
    }
}
