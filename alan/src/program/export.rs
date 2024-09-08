#[derive(Clone, Debug)]
pub enum Export {
    // TODO: Add other export types over time
    Function,
    Const,
    Type,
    OpMap,
    TypeOpMap,
}
