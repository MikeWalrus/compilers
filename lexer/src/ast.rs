use crate::token::Identifier;

#[derive(Debug)]
pub(crate) struct TranslationUnit {
    pub(crate) external_declarations: Vec<ExternalDeclaration>,
}

#[derive(Debug)]
pub(crate) enum ExternalDeclaration {
    FunctionDeclaration(FunctionDefinition),
    Declaration(Declaration),
}

#[derive(Debug)]
pub(crate) struct FunctionDefinition {
    pub(crate) declaration_specifier: DeclarationSpecifier,
    pub(crate) declarator: Declarator,
    pub(crate) compound_statement: CompoundStatement,
}

#[derive(Debug)]
pub(crate) struct Declaration {
    pub(crate) declaration_specifier: DeclarationSpecifier,
    pub(crate) init_declarator_list: InitDeclaratorList,
}

#[derive(Debug)]
pub(crate) struct InitDeclaratorList(pub(crate) Vec<InitDeclarator>);

#[derive(Debug)]
pub(crate) enum DeclarationSpecifier {
    TypeSpecifier(TypeSpecifier),
}

#[derive(Debug)]
pub(crate) enum TypeSpecifier {
    Void,
    Int,
    Double,
}

#[derive(Debug)]
pub(crate) struct InitDeclarator {
    pub(crate) declarator: Declarator,
    pub(crate) initializer: Option<Initializer>,
}

#[derive(Debug)]
pub(crate) struct Declarator {
    pub(crate) pointer: Pointer,
    pub(crate) direct: DirectDeclarator,
}

#[derive(Debug)]
pub(crate) struct Pointer(pub(crate) usize);

#[derive(Debug)]
pub(crate) struct DirectDeclarator {
    pub(crate) simple_declarator: SimpleDirectDeclarator,
    pub(crate) modifiers: Vec<DirectDeclaratorModifier>,
}

#[derive(Debug)]
pub(crate) enum DirectDeclaratorModifier {
    Array(usize),
    Function(Vec<ParameterDeclaration>),
}

#[derive(Debug)]
pub(crate) enum SimpleDirectDeclarator {
    Identifier(usize),
    Declarator(Box<Declarator>),
}

#[derive(Debug)]
pub(crate) struct ParameterDeclaration {
    pub(crate) specifier: DeclarationSpecifier,
    pub(crate) declarator: Declarator,
}

#[derive(Debug)]
pub(crate) enum Initializer {
    Expression(Expression),
    List(Vec<Initializer>),
}

#[derive(Debug)]
pub(crate) enum Statement {
    Compound(CompoundStatement),
    Expression(Expression),
    Selection(SelectionStatement),
    Iteration(IterationStatement),
    Jump(JumpStatement),
}

#[derive(Debug)]
pub(crate) struct CompoundStatement(pub(crate) Vec<BlockItem>);

#[derive(Debug)]
pub(crate) enum BlockItem {
    Declaration(Declaration),
    Statement(Statement),
}

#[derive(Debug)]
pub(crate) struct SelectionStatement {
    pub(crate) condition: Expression,
    pub(crate) consequent: Box<Statement>,
    pub(crate) alternative: Option<Box<Statement>>,
}

#[derive(Debug)]
pub(crate) enum IterationStatement {
    While(WhileStatement),
    Do(DoStatement),
    For(ForStatement),
}

#[derive(Debug)]
pub(crate) struct WhileStatement {
    pub(crate) condition: Expression,
    pub(crate) body: Box<Statement>,
}

#[derive(Debug)]
pub(crate) struct DoStatement {
    pub(crate) body: Box<Statement>,
    pub(crate) condition: Expression,
}

#[derive(Debug)]
pub(crate) struct ForStatement {
    pub(crate) initialization: Option<ForInitialization>,
    pub(crate) condition: Option<Expression>,
    pub(crate) update: Option<Expression>,
}

#[derive(Debug)]
pub(crate) enum ForInitialization {
    Expression { field1: Expression },
    Declaration { field1: Declaration },
}

#[derive(Debug)]
pub(crate) enum JumpStatement {
    Continue,
    Break,
    Return(ReturnStatement),
}

#[derive(Debug)]
pub(crate) struct ReturnStatement {
    pub(crate) value: Option<Expression>,
}

#[derive(Debug)]
pub(crate) enum Expression {
    Assignment(AssignmentExpression),
    Binary(BinaryExpression),
    Unary(UnaryExpression),
    Postfix(PostfixExpression),
    Atom(AtomExpression),
}

#[derive(Debug)]
pub(crate) struct AssignmentExpression {
    left: Box<Expression>,
    right: Box<Expression>,
}

#[derive(Debug)]
pub(crate) struct BinaryExpression {
    operator: BinaryOperator,
    left: Box<Expression>,
    right: Box<Expression>,
}

#[derive(Debug)]
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
    Comma,
}

#[derive(Debug)]
pub(crate) struct UnaryExpression {
    pub(crate) operator: UnaryOperator,
    pub(crate) left: Box<Expression>,
}

#[derive(Debug)]
pub(crate) enum UnaryOperator {
    Positive,
    Negative,
    Address,
    Indirection,
    LogicalNot,
    Not,
}

#[derive(Debug)]
pub(crate) struct PostfixExpression {
    prefix: Box<Expression>,
    postfix: PostfixExpressionPostfix,
}

#[derive(Debug)]
pub(crate) enum PostfixExpressionPostfix {
    Subscript(Box<Expression>),
    Call(Vec<Expression>),
}

#[derive(Debug)]
pub(crate) enum AtomExpression {
    Identifier(Identifier),
    Integer(u32),
    Floating(f64),
}
