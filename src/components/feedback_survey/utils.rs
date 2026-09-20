//! Maps to: CC `components/FeedbackSurvey/utils.ts`.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeedbackSurveyType {
    General,
    Session,
    PostCompact,
    Memory,
    SkillImprovement,
}

impl FeedbackSurveyType {
    pub fn as_official_str(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::Session => "session",
            Self::PostCompact => "post_compact",
            Self::Memory => "memory",
            Self::SkillImprovement => "skill_improvement",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeedbackSurveyResponse {
    Dismissed,
    Bad,
    Fine,
    Good,
}

impl FeedbackSurveyResponse {
    pub fn as_official_str(self) -> &'static str {
        match self {
            Self::Dismissed => "dismissed",
            Self::Bad => "bad",
            Self::Fine => "fine",
            Self::Good => "good",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TranscriptShareResponse {
    Yes,
    No,
    DontAskAgain,
}

impl TranscriptShareResponse {
    pub fn as_official_str(self) -> &'static str {
        match self {
            Self::Yes => "yes",
            Self::No => "no",
            Self::DontAskAgain => "dont_ask_again",
        }
    }
}
