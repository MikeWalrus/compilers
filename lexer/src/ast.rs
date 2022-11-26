use crate::token::Identifier;

#[derive(Debug)]
struct TranslationUnit {
    external_declarations: Vec<ExternalDeclaration>,
}

#[derive(Debug)]
enum ExternalDeclaration {
    FunctionDeclaration(FunctionDefinition),
    Declaration(Declaration),
}

#[derive(Debug)]
struct FunctionDefinition {
    declaration_specifier: DeclarationSpecifier,
    declarator: Declarator,
    compound_statement: CompoundStatement,
}

#[derive(Debug)]
struct Declaration {
    declaration_specifier: DeclarationSpecifier,
    init_declarator_list: InitDeclaratorList,
}

#[derive(Debug)]
struct InitDeclaratorList(Vec<InitDeclarator>);

#[derive(Debug)]
enum DeclarationSpecifier {
    TypeSpecifier(TypeSpecifier),
}

#[derive(Debug)]
enum TypeSpecifier {
    Void,
    Int,
    Double,
}

#[derive(Debug)]
struct InitDeclarator {
    declarator: Declarator,
    initializer: Option<Initializer>,
}

#[derive(Debug)]
struct Declarator {
    pointer: Pointer,
    direct: DirectDeclarator,
}

#[derive(Debug)]
struct Pointer {
    num: usize,
}

#[derive(Debug)]
struct DirectDeclarator {
    simple_declarator: DirectSimpleDeclarator,
    modifiers: Vec<DirectDeclaratorModifier>,
}

#[derive(Debug)]
enum DirectDeclaratorModifier {
    Array(usize),
    Function(Vec<ParameterDeclaration>),
}

#[derive(Debug)]
enum DirectSimpleDeclarator {
    Identifier(Identifier),
    Declarator(Box<Declarator>),
}

#[derive(Debug)]
struct ParameterDeclaration {
    specifier: DeclarationSpecifier,
    declarator: Declarator,
}

#[derive(Debug)]
enum Initializer {
    Expression(Expression),
    List(Vec<Initializer>),
}

#[derive(Debug)]
enum Statement {
    Compound(CompoundStatement),
    Expression(Expression),
    Selection(SelectionStatement),
    Iteration(IterationStatement),
    Jump(JumpStatement),
}

#[derive(Debug)]
struct CompoundStatement(Vec<BlockItem>);

#[derive(Debug)]
enum BlockItem {
    Declaration(Declaration),
    Statement(Statement),
}

#[derive(Debug)]
struct SelectionStatement {
    condition: Expression,
    consequent: Box<Statement>,
    alternative: Option<Box<Statement>>,
}

#[derive(Debug)]
enum IterationStatement {
    While(WhileStatement),
    Do(DoStatement),
    For(ForStatement),
}

#[derive(Debug)]
struct WhileStatement {
    condition: Expression,
    body: Box<Statement>,
}

#[derive(Debug)]
struct DoStatement {
    body: Box<Statement>,
    condition: Expression,
}

#[derive(Debug)]
struct ForStatement {
    initialization: Option<ForInitialization>,
    condition: Option<Expression>,
    update: Option<Expression>,
}

#[derive(Debug)]
enum ForInitialization {
    Expression(Expression),
    Declaration(Declaration),
}

#[derive(Debug)]
enum JumpStatement {
    Continue,
    Break,
    Return(ReturnStatement),
}

#[derive(Debug)]
struct ReturnStatement {
    value: Option<Expression>,
}

#[derive(Debug)]
enum Expression {
    Assignment(AssignmentExpression),
    Binary(BinaryExpression),
    Unary(UnaryExpression),
    Postfix(PostfixExpression),
    Atom(AtomExpression),
}

#[derive(Debug)]
struct AssignmentExpression {
    left: Box<Expression>,
    right: Box<Expression>,
}

#[derive(Debug)]
struct BinaryExpression {
    operator: BinaryOperator,
    left: Box<Expression>,
    right: Box<Expression>,
}

#[derive(Debug)]
enum BinaryOperator {
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
struct UnaryExpression {
    operator: UnaryOperator,
    left: Box<Expression>,
}

#[derive(Debug)]
enum UnaryOperator {
    Positive,
    Negative,
    Address,
    Indirection,
    LogicalNot,
    Not,
}

#[derive(Debug)]
struct PostfixExpression {
    prefix: Box<Expression>,
    postfix: PostfixExpressionPostfix,
}

#[derive(Debug)]
enum PostfixExpressionPostfix {
    Subscript(Box<Expression>),
    Call(Vec<Expression>),
}

#[derive(Debug)]
enum AtomExpression {
    Identifier(Identifier),
    Integer(u32),
    Floating(f64),
}
