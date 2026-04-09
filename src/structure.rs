use std::path::PathBuf;

use derive_more::Display;
use derive_more::with_trait::FromStr;
use serde::Serialize;

#[derive(FromStr, Display, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy, Debug, Serialize)]
pub enum Role {
    Minion,
    Rival,
    Boss,
}

#[derive(FromStr, Display, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy, Debug, Serialize)]
pub enum Size {
    Small,
    Medium,
    Large,
    Huge,
    Gargantuan,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Serialize)]
pub struct Ranged {
    pub value: u64,
    pub min: u64,
    pub max: u64,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Serialize)]
pub struct Attributes {
    pub strength: u64,
    pub speed: u64,
    pub intellect: u64,
    pub willpower: u64,
    pub awareness: u64,
    pub presence: u64,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Serialize)]
pub struct Defenses {
    pub physical_defense: u64,
    pub cognitive_defense: u64,
    pub spiritual_defense: u64,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize)]
pub struct Movement {
    pub value: u64,
    pub extra: Vec<(u64, String)>,
    pub comment: Option<String>,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize)]
pub struct Skill {
    pub name: String,
    pub value: u64,
    pub ranks: Option<u64>,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize)]
pub struct Feature {
    pub name: String,
    pub description: String,
}

#[derive(Display, PartialEq, Eq, Hash, Clone, Copy, Debug, Serialize)]
pub enum ActionKind {
    #[display("▶")]
    One,
    #[display("▶▶")]
    Two,
    #[display("▶▶▶")]
    Three,
    #[display("▷")]
    Free,
    #[display("↩")]
    Reaction,
}

impl FromStr for ActionKind {
    type Err = derive_more::FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(
            match s
                .to_ascii_lowercase()
                .replace(char::is_whitespace, "")
                .as_str()
            {
                "one" | "▶" => Self::One,
                "two" | "▶▶" => Self::Two,
                "three" | "▶▶▶" => Self::Three,
                "free" | "▷" => Self::Free,
                "reaction" | "↩" => Self::Reaction,
                _ => return Err(derive_more::FromStrError::new("ActionKind")),
            },
        )
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize)]
pub struct Action {
    pub kind: ActionKind,
    pub name: String,
    pub description: String,
}

#[derive(FromStr, Display, Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize)]
pub enum BeastKind {
    Humanoid,
    Animal,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize)]
pub struct OpportunityAndComplication {
    pub opportunity: String,
    pub complication: String,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize)]
pub struct Beast {
    pub title: String,
    pub tier: u64,
    pub role: Role,
    pub size: Size,
    pub kind: BeastKind,
    pub attributes: Attributes,
    pub defenses: Defenses,
    pub health: Ranged,
    pub focus: u64,
    pub investiture: u64,
    pub deflect: Option<(u64, String)>,
    pub movement: Movement,
    pub senses: (u64, String),
    pub immunities: Vec<String>,
    pub physical_skills: Vec<Skill>,
    pub cognitive_skills: Vec<Skill>,
    pub spiritual_skills: Vec<Skill>,
    pub surge_skills: Vec<Skill>,
    pub languages: Option<Vec<String>>,
    pub features: Vec<Feature>,
    pub actions: Vec<Action>,
    pub opportunities_and_complications: Option<OpportunityAndComplication>,
    pub image: Option<PathBuf>,
    pub description: Option<String>,
    pub tactics: Option<String>,
}
