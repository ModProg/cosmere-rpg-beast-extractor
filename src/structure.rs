use derive_more::Display;
use derive_more::with_trait::FromStr;
use serde::Serialize;

#[derive(FromStr, Display, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy, Debug, Serialize)]
pub enum Role {
    Minion,
    Rival,
    Boss,
}

#[derive(FromStr, Display, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Debug, Serialize)]
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

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize)]
pub struct Attributes {
    pub strength: String,
    pub speed: String,
    pub intellect: String,
    pub willpower: String,
    pub awareness: String,
    pub presence: String,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize)]
pub struct Defenses {
    pub physical: String,
    pub cognitive: String,
    pub spiritual: String,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize)]
pub struct DescValue<T> {
    pub value: T,
    pub desc: String,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize)]
pub struct Movement {
    pub value: u64,
    pub extra: Vec<DescValue<u64>>,
    pub desc: Option<String>,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize)]
pub struct Skill {
    pub name: String,
    pub value: String,
    pub desc: Option<String>,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize)]
pub struct Feature {
    pub name: String,
    pub desc: String,
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
    pub desc: String,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize)]
pub struct Beast {
    pub name: String,
    pub tier: u64,
    pub role: Role,
    pub size: Option<Size>,
    pub kind: String,
    pub attributes: Attributes,
    pub defenses: Defenses,
    pub health: Ranged,
    pub focus: u64,
    pub investiture: u64,
    pub deflect: Option<DescValue<u64>>,
    pub movement: Movement,
    pub senses: DescValue<u64>,
    pub immunities: Vec<String>,
    pub physical_skills: Vec<Skill>,
    pub cognitive_skills: Vec<Skill>,
    pub spiritual_skills: Vec<Skill>,
    pub surge_skills: Vec<Skill>,
    pub languages: Option<Vec<String>>,
    pub features: Vec<Feature>,
    pub actions: Vec<Action>,
    pub opportunity: Option<String>,
    pub complication: Option<String>,
}

impl Beast {
    pub fn update_for_obsidian(mut self) -> Self {
        for action in &mut self.actions {
            action.name.insert_str(0, &format!("{} ", action.kind));
        }
        self
    }
}
