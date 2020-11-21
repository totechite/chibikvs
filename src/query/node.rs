
use crate::sql::token::{ArithmeticOperator};

pub trait Command_Declare {}

#[derive(Debug, Default)]
pub struct SELECT_FROM_Declare {
    pub text: String,
    pub select_term: SELECT_Declare,
    pub from_term: FROM_Declare,
    pub join_term: Option<JOIN_Declare>,
    pub where_term: Option<WHERE_Declare>,
    pub order_by: Option<ORDER_BY_Declare>,
    pub group_by: Option<GROUP_BY_Declare>,
}

impl SELECT_FROM_Declare {
    pub fn new() -> Self {
        Default::default()
    }
}

#[derive(Debug, Default)]
pub struct SELECT_Declare{
    pub text: String,
    pub contents: Vec<Column_Term>,
}

#[derive(Debug)]
pub struct Column_Term {
    pub text: String,
    pub context: Column_Context,
    pub alias: Option<String>,
}

#[derive(Debug)]
pub enum Column_Context {
    Literal { column_name: String, attribute: Option<String> },
    SubQuery(Box<SELECT_FROM_Declare>),
    Computation(ComputationTerm),
}

#[derive(Debug, Default)]
pub struct FROM_Declare {
    pub text: String,
    pub contents: Vec<Table_Term>,
}

#[derive(Debug)]
pub struct Table_Term {
    pub text: String,
    pub context: Table_Context,
    pub alias: Option<String>,
}

#[derive(Debug)]
pub enum Literal {
    String(String),
    Number(usize),
}

#[derive(Debug, Clone)]
pub enum OperatorTerm<O> where O: Clone {
    String(String),
    Number(usize),
    Boolean(bool),
    Operator(O),
}

#[derive(Debug)]
pub enum Table_Context {
    Literal(String),
    SubQuery(Box<SELECT_FROM_Declare>),
}

#[derive(Debug, )]
pub struct WHERE_Declare {
    pub text: String,
    pub content: Box<ConditionTerm>,
}

#[derive(Debug, Default)]
pub struct JOIN_Declare {
    pub text: String,
    pub table: String,

}

#[derive(Debug, Clone)]
pub struct ConditionTerm {
    pub text: String,
    pub operator: LogicalOperator,
    pub left_v: OperatorTerm<Box<ConditionTerm>>,
    pub right_v: OperatorTerm<Box<ConditionTerm>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum LogicalOperator {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    EqualOrGreaterThan,
    EqualOrLessThan,
    AND,
    OR,
    NOT,
}

#[derive(Debug)]
pub struct ComputationTerm {
    text: String,
    operator: ArithmeticOperator,
    left_v: OperatorTerm<Box<ConditionTerm>>,
    right_v: OperatorTerm<Box<ConditionTerm>>,
}

#[derive(Debug, Default)]
pub struct ORDER_BY_Declare {}

#[derive(Debug, Default)]
pub struct GROUP_BY_Declare {}


use serde::{Serialize, Deserialize};

trait Command {}

struct Plan {
    command: dyn Command
}

#[derive(Debug)]
pub struct SELECT {
    pub fields: Vec<Field>,
    pub FROM: Option<Vec<String>>,
    pub WHERE: Option<SearchCondition>,
}

#[derive(Debug)]
pub struct CREATE {
    pub TABLE: Option<(String, Vec<FieldDefinition>)>
}

pub struct INSERT {
    pub INTO: (String, Option<Vec<String>>),
    // (tableName, Vec<field>)
    pub VALUES: Vec<Vec<String>>,
}

#[derive(Debug)]
pub enum Field {
    All,
    Plain { name: String, table_name: Option<String>, AS: Option<String> },
    Calc { expr: Box<Expression>, name: String, table_name: Option<String>, AS: Option<String> },
}

pub struct Table {
    pub name: String,
    pub scheme: Option<String>,
    pub AS: Option<String>,
}

#[derive(Debug)]
pub enum Expression {
    //    User defined value.

    Var(String),
    Number(u32),

    //    Operator
    //    +
    Add(Box<Expression>, Box<Expression>),
    //    -
    Sub(Box<Expression>, Box<Expression>),
    //    *
    Mul(Box<Expression>, Box<Expression>),
    //    /
    Div(Box<Expression>, Box<Expression>),
}

#[derive(Debug)]
pub enum SearchCondition {
    Equal(Value, Value),
    NotEqual(Value, Value),
    GreaterThan(Value, Value),
    LessThan(Value, Value),
    EqualOrGreaterThan(Value, Value),
    EqualOrLessThan(Value, Value),
    AND(Box<SearchCondition>, Box<SearchCondition>),
    OR(Box<SearchCondition>, Box<SearchCondition>),
    NOT(Box<SearchCondition>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    String(String),
    Number(isize),
}

#[derive(Clone, Debug, PartialEq)]
pub struct FieldDefinition {
    pub name: String,
    pub T: Type,
    // constraint: Option<Vec<Constraint>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Constraint {
    PRIMARY_KEY(Vec<String>),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Type {
    integer,
    text,
}

