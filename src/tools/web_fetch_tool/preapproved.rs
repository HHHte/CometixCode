//! Maps to CC `tools/WebFetchTool/preapproved.ts`.
//!
//! These hosts are preapproved only for the read-only WebFetch tool. They must
//! not be reused for sandbox/network allow-lists.

pub const PREAPPROVED_HOSTS: &[&str] = &[
    "platform.claude.com",
    "code.claude.com",
    "modelcontextprotocol.io",
    "github.com/anthropics",
    "agentskills.io",
    "docs.python.org",
    "en.cppreference.com",
    "docs.oracle.com",
    "learn.microsoft.com",
    "developer.mozilla.org",
    "go.dev",
    "pkg.go.dev",
    "www.php.net",
    "docs.swift.org",
    "kotlinlang.org",
    "ruby-doc.org",
    "doc.rust-lang.org",
    "www.typescriptlang.org",
    "react.dev",
    "angular.io",
    "vuejs.org",
    "nextjs.org",
    "expressjs.com",
    "nodejs.org",
    "bun.sh",
    "jquery.com",
    "getbootstrap.com",
    "tailwindcss.com",
    "d3js.org",
    "threejs.org",
    "redux.js.org",
    "webpack.js.org",
    "jestjs.io",
    "reactrouter.com",
    "docs.djangoproject.com",
    "flask.palletsprojects.com",
    "fastapi.tiangolo.com",
    "pandas.pydata.org",
    "numpy.org",
    "www.tensorflow.org",
    "pytorch.org",
    "scikit-learn.org",
    "matplotlib.org",
    "requests.readthedocs.io",
    "jupyter.org",
    "laravel.com",
    "symfony.com",
    "wordpress.org",
    "docs.spring.io",
    "hibernate.org",
    "tomcat.apache.org",
    "gradle.org",
    "maven.apache.org",
    "asp.net",
    "dotnet.microsoft.com",
    "nuget.org",
    "blazor.net",
    "reactnative.dev",
    "docs.flutter.dev",
    "developer.apple.com",
    "developer.android.com",
    "keras.io",
    "spark.apache.org",
    "huggingface.co",
    "www.kaggle.com",
    "www.mongodb.com",
    "redis.io",
    "www.postgresql.org",
    "dev.mysql.com",
    "www.sqlite.org",
    "graphql.org",
    "prisma.io",
    "docs.aws.amazon.com",
    "cloud.google.com",
    "kubernetes.io",
    "www.docker.com",
    "www.terraform.io",
    "www.ansible.com",
    "vercel.com/docs",
    "docs.netlify.com",
    "devcenter.heroku.com",
    "cypress.io",
    "selenium.dev",
    "docs.unity.com",
    "docs.unrealengine.com",
    "git-scm.com",
    "nginx.org",
    "httpd.apache.org",
];

/// Maps to CC `isPreapprovedHost(hostname, pathname)`.
pub fn is_preapproved_host(hostname: &str, pathname: &str) -> bool {
    let hostname = hostname.to_ascii_lowercase();
    PREAPPROVED_HOSTS.iter().any(|entry| {
        if let Some((host, path_prefix)) = entry.split_once('/') {
            if hostname != host {
                return false;
            }
            let prefix = format!("/{path_prefix}");
            pathname == prefix || pathname.starts_with(&format!("{prefix}/"))
        } else {
            hostname == *entry
        }
    })
}

/// Extracts `(hostname, pathname)` for the WebFetch permission fast path.
///
/// The official code uses `new URL(url)`. This parser intentionally supports
/// only the absolute HTTP(S)-URL subset accepted by the tool schema/validator.
pub fn parse_web_fetch_url_host_path(url: &str) -> Option<(String, String)> {
    let trimmed = url.trim();
    let after_scheme = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))?;
    let authority_end = after_scheme
        .find(['/', '?', '#'])
        .unwrap_or(after_scheme.len());
    let mut authority = &after_scheme[..authority_end];
    if authority.is_empty() {
        return None;
    }
    if let Some((_, host_part)) = authority.rsplit_once('@') {
        authority = host_part;
    }
    let host = if let Some(rest) = authority.strip_prefix('[') {
        let (ipv6, _) = rest.split_once(']')?;
        ipv6
    } else {
        authority
            .split_once(':')
            .map_or(authority, |(host, _)| host)
    };
    let host = host.trim().trim_end_matches('.').to_ascii_lowercase();
    if host.is_empty() {
        return None;
    }

    let path =
        if authority_end < after_scheme.len() && after_scheme[authority_end..].starts_with('/') {
            let tail = &after_scheme[authority_end..];
            let path_end = tail.find(['?', '#']).unwrap_or(tail.len());
            tail[..path_end].to_string()
        } else {
            "/".to_string()
        };
    Some((host, path))
}

/// Maps to CC `WebFetchTool.checkPermissions(...)` preapproved-host shortcut.
pub fn is_preapproved_web_fetch_url(url: &str) -> bool {
    parse_web_fetch_url_host_path(url)
        .as_ref()
        .is_some_and(|(host, path)| is_preapproved_host(host, path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preapproved_host_matches_hostname_and_path_scoped_entries_like_official() {
        assert!(is_preapproved_web_fetch_url(
            "https://doc.rust-lang.org/book/"
        ));
        assert!(is_preapproved_web_fetch_url(
            "https://github.com/anthropics/claude-code"
        ));
        assert!(!is_preapproved_web_fetch_url(
            "https://github.com/anthropics-evil/claude-code"
        ));
        assert!(is_preapproved_web_fetch_url(
            "https://vercel.com/docs/functions"
        ));
        assert!(!is_preapproved_web_fetch_url("https://vercel.com/blog"));
    }

    #[test]
    fn parse_web_fetch_url_host_path_handles_ports_userinfo_and_queries() {
        let (host, path) = parse_web_fetch_url_host_path(
            "https://user:pass@Example.COM:443/docs/page?q=1#section",
        )
        .expect("url should parse");
        assert_eq!(host, "example.com");
        assert_eq!(path, "/docs/page");
        assert!(parse_web_fetch_url_host_path("mailto:dev@example.com").is_none());
    }
}
