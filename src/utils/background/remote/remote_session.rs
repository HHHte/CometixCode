//! Background remote session preconditions.
//!
//! Maps to: CC `utils/background/remote/remoteSession.ts`.
//!
//! SEAM (checks missing): `checkBackgroundRemoteSessionEligibility` (CC
//! `:41-…`) performs login/remote-environment/git/GitHub-app/policy probes
//! against the teleport + GitHub stacks, none of which are ported; only the
//! precondition vocabulary lands so `formatPreconditionError`
//! (`tasks/remote_agent_task.rs`) has its input type.

/// Maps to: CC `utils/background/remote/remoteSession.ts:31-37`
/// `BackgroundRemoteSessionPrecondition` — the discriminated union of failed
/// eligibility checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackgroundRemoteSessionPrecondition {
    NotLoggedIn,
    NoRemoteEnvironment,
    NotInGitRepo,
    NoGitRemote,
    GithubAppNotInstalled,
    PolicyBlocked,
}
