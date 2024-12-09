#[derive(Debug, Default, Clone)]
pub enum BranchTypes {
    #[default]
    Dev,
    Stage,
    DevNext,
    Custom(String),
}

impl std::str::FromStr for BranchTypes {
    type Err = ();

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        match str {
            "stage" | "Stage" | "s" | "S" => Ok(BranchTypes::Stage),
            "next" | "Next" | "n" | "N" => Ok(BranchTypes::DevNext),
            "dev" | "Dev" | "d" | "D" => Ok(BranchTypes::Dev),
            "" => Err(()),
            str => Ok(BranchTypes::Custom(str.to_string())),
        }
    }
}

impl std::fmt::Display for BranchTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BranchTypes::Dev => write!(f, "Dev"),
            BranchTypes::Stage => write!(f, "Stage"),
            BranchTypes::DevNext => write!(f, "DevNext"),
            BranchTypes::Custom(str) => write!(f, "{}", str),
        }
    }
}
