#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Characters(String),
    Expansion(Expansion),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    IfUnset,
    IfUnsetOrNull,
    IfSet,
    IfSetAndNotNull,
    AssignIfUnset,
    AssignIfUnsetOrNull,
    ErrorIfUnset,
    ErrorIfUnsetOrNull,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    pub name: String,
    pub value: Vec<Expression>,
}

impl Assignment {
    pub fn new(name: String, value: Vec<Expression>) -> Self {
        Self { name, value }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expansion {
    pub name: String,
    pub operator: Operator,
    pub rhs: Vec<Expression>,
}

impl Expansion {
    pub fn new(name: String, operator: Operator, rhs: Vec<Expression>) -> Self {
        Self {
            name,
            operator,
            rhs,
        }
    }
}
