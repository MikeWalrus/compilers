use serde::Serialize;

use crate::token::{RelopKind, TokenKind};

#[derive(Debug, Serialize)]
pub(crate) struct TranslationUnit {
    pub(crate) external_declarations: Vec<ExternalDeclaration>,
}

#[derive(Debug, Serialize)]
pub(crate) enum ExternalDeclaration {
    FunctionDeclaration(FunctionDefinition),
    Declaration(Declaration),
}

#[derive(Debug, Serialize)]
pub(crate) struct FunctionDefinition {
    pub(crate) declaration_specifier: DeclarationSpecifier,
    pub(crate) declarator: Declarator,
    pub(crate) compound_statement: CompoundStatement,
}

#[derive(Debug, Serialize)]
pub(crate) struct Declaration {
    pub(crate) declaration_specifier: DeclarationSpecifier,
    pub(crate) init_declarator_list: InitDeclaratorList,
}

#[derive(Debug, Serialize)]
pub(crate) struct InitDeclaratorList(pub(crate) Vec<InitDeclarator>);

#[derive(Debug, Serialize)]
pub(crate) enum DeclarationSpecifier {
    TypeSpecifier(TypeSpecifier),
}

#[derive(Debug, Serialize)]
pub(crate) enum TypeSpecifier {
    Void,
    Int,
    Double,
}

#[derive(Debug, Serialize)]
pub(crate) struct InitDeclarator {
    pub(crate) declarator: Declarator,
    pub(crate) initializer: Option<Initializer>,
}

#[derive(Debug, Serialize)]
pub(crate) struct Declarator {
    pub(crate) pointer: Pointer,
    pub(crate) direct: DirectDeclarator,
}

#[derive(Debug, Serialize)]
pub(crate) struct Pointer(pub(crate) usize);

#[derive(Debug, Serialize)]
pub(crate) struct DirectDeclarator {
    pub(crate) simple_declarator: SimpleDirectDeclarator,
    pub(crate) modifiers: Vec<DirectDeclaratorModifier>,
}

#[derive(Debug, Serialize)]
pub(crate) enum DirectDeclaratorModifier {
    Array(usize),
    Function(Vec<ParameterDeclaration>),
}

#[derive(Debug, Serialize)]
pub(crate) enum SimpleDirectDeclarator {
    Identifier(usize),
    Declarator(Box<Declarator>),
}

#[derive(Debug, Serialize)]
pub(crate) struct ParameterDeclaration {
    pub(crate) specifier: DeclarationSpecifier,
    pub(crate) declarator: Declarator,
}

#[derive(Debug, Serialize)]
pub(crate) enum Initializer {
    Expression(Expression),
    List(Vec<Initializer>),
}

#[derive(Debug, Serialize)]
pub(crate) enum Statement {
    Compound(CompoundStatement),
    Expression(Option<Expression>),
    Selection(SelectionStatement),
    Iteration(IterationStatement),
    Jump(JumpStatement),
}

#[derive(Debug, Serialize)]
pub(crate) struct CompoundStatement(pub(crate) Vec<BlockItem>);

#[derive(Debug, Serialize)]
pub(crate) enum BlockItem {
    Declaration(Declaration),
    Statement(Statement),
}

#[derive(Debug, Serialize)]
pub(crate) struct SelectionStatement {
    pub(crate) condition: Expression,
    pub(crate) consequent: Box<Statement>,
    pub(crate) alternative: Option<Box<Statement>>,
}

#[derive(Debug, Serialize)]
pub(crate) enum IterationStatement {
    While(WhileStatement),
    Do(DoStatement),
    For(ForStatement),
}

#[derive(Debug, Serialize)]
pub(crate) struct WhileStatement {
    pub(crate) condition: Expression,
    pub(crate) body: Box<Statement>,
}

#[derive(Debug, Serialize)]
pub(crate) struct DoStatement {
    pub(crate) body: Box<Statement>,
    pub(crate) condition: Expression,
}

#[derive(Debug, Serialize)]
pub(crate) struct ForStatement {
    pub(crate) initialization: ForInitialization,
    pub(crate) condition: Option<Expression>,
    pub(crate) update: Option<Expression>,
    pub(crate) body: Box<Statement>,
}

#[derive(Debug, Serialize)]
pub(crate) enum ForInitialization {
    Expression(Option<Expression>),
    Declaration(Declaration),
}

#[derive(Debug, Serialize)]
pub(crate) enum JumpStatement {
    Continue,
    Break,
    Return(Option<Expression>),
}

#[derive(Debug, Serialize)]
pub(crate) enum Expression {
    Assignment(AssignmentExpression),
    Binary(BinaryExpression),
    Unary(UnaryExpression),
    Postfix(PostfixExpression),
    Atom(AtomExpression),
}

#[derive(Debug, Serialize)]
pub(crate) struct AssignmentExpression {
    pub(crate) left: Box<Expression>,
    pub(crate) right: Box<Expression>,
}

#[derive(Debug, Serialize)]
pub(crate) struct BinaryExpression {
    pub(crate) operator: BinaryOperator,
    pub(crate) left: Box<Expression>,
    pub(crate) right: Box<Expression>,
}

#[derive(Debug, Serialize)]
pub(crate) enum BinaryOperator {
    Add,
    Minus,
    Multiply,
    DivideBy,
    LogicalAnd,
    LogicalOr,
    And,
    Or,
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Neq,
}

trace::init_depth_var!();

impl From<&TokenKind> for BinaryOperator {
    #[trace::trace]
    fn from(t: &TokenKind) -> Self {
        match t {
            TokenKind::And => BinaryOperator::LogicalAnd,
            TokenKind::Or => BinaryOperator::LogicalOr,
            TokenKind::BitAnd => BinaryOperator::And,
            TokenKind::BitOr => BinaryOperator::Or,
            TokenKind::Plus => BinaryOperator::Add,
            TokenKind::Minus => BinaryOperator::Minus,
            TokenKind::Star => BinaryOperator::Multiply,
            TokenKind::Divide => BinaryOperator::DivideBy,
            TokenKind::Relop(RelopKind::Lt) => BinaryOperator::Lt,
            TokenKind::Relop(RelopKind::Gt) => BinaryOperator::Gt,
            TokenKind::Relop(RelopKind::Le) => BinaryOperator::Le,
            TokenKind::Relop(RelopKind::Ge) => BinaryOperator::Ge,
            TokenKind::Relop(RelopKind::Eq) => BinaryOperator::Eq,
            TokenKind::Relop(RelopKind::Neq) => BinaryOperator::Neq,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct UnaryExpression {
    pub(crate) operator: UnaryOperator,
    pub(crate) operand: Box<Expression>,
}

#[derive(Debug, Serialize)]
pub(crate) enum UnaryOperator {
    Positive,
    Negative,
    Address,
    Indirection,
    LogicalNot,
    Not,
}

impl From<&TokenKind> for UnaryOperator {
    fn from(t: &TokenKind) -> Self {
        match t {
            TokenKind::Plus => UnaryOperator::Positive,
            TokenKind::Minus => UnaryOperator::Negative,
            TokenKind::BitAnd => UnaryOperator::Address,
            TokenKind::Star => UnaryOperator::Indirection,
            TokenKind::BitNot => UnaryOperator::Not,
            TokenKind::Not => UnaryOperator::LogicalNot,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct PostfixExpression {
    pub(crate) operand: Box<Expression>,
    pub(crate) postfix: PostfixExpressionPostfix,
}

#[derive(Debug, Serialize)]
pub(crate) enum PostfixExpressionPostfix {
    Subscript(Box<Expression>),
    Call(Vec<Expression>),
}

#[derive(Debug, Serialize)]
pub(crate) enum AtomExpression {
    Identifier(usize),
    Integer(u32),
    Floating(f64),
}
