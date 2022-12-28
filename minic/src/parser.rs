use crate::error::ErrorKind;
use crate::token::*;
use crate::{ast::*, error::Error, token::Token};
use trace::trace;
use util::*;

trace::init_depth_var!();

pub(crate) fn parse(tokens: &[Token]) -> Result<TranslationUnit, Error> {
    let mut i = 0;
    parse_translation_unit(&mut i, tokens)
}

mod util {

    use super::*;

    #[trace::trace]
    pub fn parse_left<const C: char>(
        i: &mut usize,
        tokens: &[Token],
    ) -> Result<Position, Error> {
        let e = || error(*i, tokens, ErrorKind::ExpectStr(C.into()));
        let token = tokens.get(*i).ok_or_else(e)?;
        let token_kind = if C == '(' {
            TokenKind::LeftParen
        } else if C == ';' {
            TokenKind::Semicolon
        } else {
            unimplemented!()
        };
        if token_kind != token.kind {
            return Err(e());
        };
        *i += 1;
        Ok(token.pos)
    }

    #[trace]
    pub fn parse_right<const C: char>(
        i: &mut usize,
        tokens: &[Token],
        left_pos: Position,
    ) -> Result<(), Error> {
        let e = || Error {
            error_kind: ErrorKind::UnmatchedParenthesis(left_pos),
            pos: left_pos,
        };
        let token = &tokens.get(*i).ok_or_else(e)?.kind;
        let k = if C == ')' {
            TokenKind::RightParen
        } else if C == ']' {
            TokenKind::RightSqBracket
        } else if C == '}' {
            TokenKind::RightBrace
        } else {
            unreachable!()
        };
        (&k == token)
            .then(|| {
                *i += 1;
            })
            .ok_or_else(e)
    }

    pub fn parse_optional_list<
        const D: char,
        T,
        F: Fn(&mut usize, &[Token]) -> Result<T, Error>,
    >(
        i: &mut usize,
        tokens: &[Token],
        first_item: T,
        parse: F,
    ) -> Result<Vec<T>, Error> {
        let delimiter = if D == ',' {
            TokenKind::Comma
        } else {
            unreachable!()
        };

        let mut v = vec![first_item];
        loop {
            let token = tokens.get(*i);
            if let Some(d) = token.map(|t| &t.kind) {
                if d == &delimiter {
                    *i += 1;
                    let item = parse(i, tokens)?;
                    v.push(item);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        Ok(v)
    }

    pub fn parse_non_empty_list<
        const D: char,
        T,
        F: Fn(&mut usize, &[Token]) -> Result<T, Error>,
    >(
        i: &mut usize,
        tokens: &[Token],
        parse: F,
    ) -> Result<Vec<T>, Error> {
        let first_item = parse(i, tokens)?;
        parse_optional_list::<D, _, _>(i, tokens, first_item, parse)
    }
}

fn parse_translation_unit(
    i: &mut usize,
    tokens: &[Token],
) -> Result<TranslationUnit, Error> {
    let mut v = Vec::new();
    loop {
        if tokens.get(*i).is_none() {
            break;
        }
        v.push(parse_external_declaration(i, tokens)?);
    }
    Ok(TranslationUnit {
        external_declarations: v,
    })
}

#[trace]
// FunctionDefinition | Declaration
// DeclarationSpecifier Declarator CompoundStatement
// DeclarationSpecifier (Declarator | Declarator = Initializer) {, InitDeclarator} ;
fn parse_external_declaration(
    i: &mut usize,
    tokens: &[Token],
) -> Result<ExternalDeclaration, Error> {
    let specifier = parse_declaration_specifier(i, tokens)?;
    let declarator = parse_declarator(i, tokens)?;
    let token = tokens.get(*i).map(|t| &t.kind);
    match token {
        Some(TokenKind::LeftBrace) => {
            let compound_statement = stmt::parse_compound_statement(i, tokens)?;
            Ok(ExternalDeclaration::FunctionDeclaration(
                FunctionDefinition {
                    declaration_specifier: specifier,
                    declarator,
                    compound_statement,
                },
            ))
        }
        Some(_) => {
            let initializer = parse_assign_initializer(token, i, tokens)?;
            let init_declarator = InitDeclarator {
                declarator,
                initializer,
            };
            let init_declarator_list =
                InitDeclaratorList(parse_optional_list::<',', _, _>(
                    i,
                    tokens,
                    init_declarator,
                    parse_init_declarator,
                )?);
            Ok(ExternalDeclaration::Declaration(Declaration {
                declaration_specifier: specifier,
                init_declarator_list,
            }))
        }
        None => {
            Err(error(*i, tokens, ErrorKind::ExpectStr("; or {".to_owned())))
        }
    }
}

#[trace]
fn parse_declaration(
    i: &mut usize,
    tokens: &[Token],
) -> Result<Declaration, Error> {
    let declaration_specifier = parse_declaration_specifier(i, tokens)?;
    let init_declarator_list =
        InitDeclaratorList(parse_non_empty_list::<',', _, _>(
            i,
            tokens,
            parse_init_declarator,
        )?);
    parse_left::<';'>(i, tokens)?;
    Ok(Declaration {
        declaration_specifier,
        init_declarator_list,
    })
}

#[trace]
fn parse_assign_initializer(
    t: Option<&TokenKind>,
    i: &mut usize,
    tokens: &[Token],
) -> Result<Option<Initializer>, Error> {
    let initializer = if let Some(TokenKind::Relop(RelopKind::Assign)) = t {
        *i += 1;
        Some(parse_initializer(i, tokens)?)
    } else {
        None
    };
    Ok(initializer)
}

fn parse_init_declarator(
    i: &mut usize,
    tokens: &[Token],
) -> Result<InitDeclarator, Error> {
    let declarator = parse_declarator(i, tokens)?;
    let token = tokens.get(*i).map(|t| &t.kind);
    let initializer = parse_assign_initializer(token, i, tokens)?;
    Ok(InitDeclarator {
        declarator,
        initializer,
    })
}

fn parse_initializer(
    i: &mut usize,
    tokens: &[Token],
) -> Result<Initializer, Error> {
    let token = tokens.get(*i).map(|t| &t.kind);
    Ok(if let Some(TokenKind::LeftBrace) = token {
        let left_pos = tokens[*i].pos;
        *i += 1;
        let list =
            parse_non_empty_list::<',', _, _>(i, tokens, parse_initializer)?;
        parse_right::<'}'>(i, tokens, left_pos)?;
        Initializer::List(list)
    } else {
        let expr = expr::parse_expression(i, tokens)?;
        Initializer::Expression(expr)
    })
}

#[trace]
// Pointer DirectDeclarator | DirectDeclarator
fn parse_declarator(
    i: &mut usize,
    tokens: &[Token],
) -> Result<Declarator, Error> {
    let e = || error(*i, tokens, ErrorKind::ExpectDeclarator);
    let token = tokens.get(*i).ok_or_else(e)?;
    let pointer = if let TokenKind::Star = token.kind {
        parse_pointer(i, tokens)
    } else {
        0
    };
    let direct = parse_direct_declarator(i, tokens)?;
    Ok(Declarator {
        pointer: Pointer(pointer),
        direct,
    })
}

#[trace]
// Identifier DirectDeclaratorModifiers
// `(` Declarator `)` DirectDeclaratorModifiers
fn parse_direct_declarator(
    i: &mut usize,
    tokens: &[Token],
) -> Result<DirectDeclarator, Error> {
    let e = || error(*i, tokens, ErrorKind::ExpectDirectDeclarator);
    let token = tokens.get(*i).ok_or_else(e)?;
    let simple_declarator = match token.kind {
        TokenKind::LeftParen => {
            let left_pos = token.pos;
            *i += 1;
            let s = SimpleDirectDeclarator::Declarator(Box::new(
                parse_declarator(i, tokens)?,
            ));
            parse_right::<')'>(i, tokens, left_pos)?;
            s
        }
        TokenKind::Id(identifier) => {
            *i += 1;
            SimpleDirectDeclarator::Identifier(identifier)
        }
        _ => return Err(e()),
    };

    let modifiers = parse_direct_declarator_modifiers(i, tokens)?;

    Ok(DirectDeclarator {
        simple_declarator,
        modifiers,
    })
}

#[trace]
fn parse_direct_declarator_modifiers(
    i: &mut usize,
    tokens: &[Token],
) -> Result<Vec<DirectDeclaratorModifier>, Error> {
    let mut ret = Vec::new();
    loop {
        match tokens.get(*i) {
            Some(t) => match t.kind {
                TokenKind::LeftParen => {
                    let left_pos = t.pos;
                    *i += 1;
                    let parameter_list = parse_parameter_list(i, tokens)?;
                    ret.push(DirectDeclaratorModifier::Function(
                        parameter_list,
                    ));
                    parse_right::<')'>(i, tokens, left_pos)?;
                }
                TokenKind::LeftSqBracket => {
                    let left_pos = t.pos;
                    *i += 1;
                    let integer_constant = parse_integer_constant(i, tokens)?;
                    ret.push(DirectDeclaratorModifier::Array(integer_constant));
                    parse_right::<']'>(i, tokens, left_pos)?;
                }
                _ => return Ok(ret),
            },
            None => return Ok(ret),
        }
    }
}

fn parse_integer_constant(
    i: &mut usize,
    tokens: &[Token],
) -> Result<usize, Error> {
    let e = || error(*i, tokens, ErrorKind::ExpectIntegerConstant);
    let token = tokens.get(*i).ok_or_else(e)?;
    if let TokenKind::IntegerConstant(n) = token.kind {
        *i += 1;
        return Ok(n as usize);
    }
    Err(e())
}

#[trace]
// ParameterDeclaration {, ParameterDeclaration}
fn parse_parameter_list(
    i: &mut usize,
    tokens: &[Token],
) -> Result<Vec<ParameterDeclaration>, Error> {
    parse_non_empty_list::<',', _, _>(i, tokens, parse_parameter_declaration)
}

fn parse_parameter_declaration(
    i: &mut usize,
    tokens: &[Token],
) -> Result<ParameterDeclaration, Error> {
    let specifier = parse_declaration_specifier(i, tokens)?;
    let declarator = parse_declarator(i, tokens)?;
    Ok(ParameterDeclaration {
        specifier,
        declarator,
    })
}

fn parse_pointer(i: &mut usize, tokens: &[Token]) -> usize {
    let mut count = 0;
    loop {
        *i += 1;
        count += 1;
        if let Some(TokenKind::Star) = tokens.get(*i).map(|t| &t.kind) {
        } else {
            return count;
        }
    }
}

fn error(i: usize, tokens: &[Token], kind: ErrorKind) -> Error {
    let pos = if i == 0 {
        Position { line: 0, col: 0 }
    } else {
        tokens[(i as u32 - 1) as usize].pos
    };
    Error {
        pos,
        error_kind: kind,
    }
}

fn parse_declaration_specifier(
    i: &mut usize,
    tokens: &[Token],
) -> Result<DeclarationSpecifier, Error> {
    let e = || error(*i, tokens, ErrorKind::ExpectDeclarationSpecifier);
    let token = tokens.get(*i).ok_or_else(e)?;
    let specifier = match &token.kind {
        crate::token::TokenKind::Qualifier(t) => match t {
            QualifierKind::Void => TypeSpecifier::Void,
            QualifierKind::Int => TypeSpecifier::Int,
            QualifierKind::Double => TypeSpecifier::Double,
        },
        _ => return Err(e()),
    };
    *i += 1;
    Ok(DeclarationSpecifier::TypeSpecifier(specifier))
}

mod stmt {
    use std::assert_matches::assert_matches;

    use super::expr::*;
    use super::*;

    #[trace::trace]
    fn parse_statement(
        i: &mut usize,
        tokens: &[Token],
    ) -> Result<Statement, Error> {
        let e = || error(*i, tokens, ErrorKind::ExpectStatement);
        let token = tokens.get(*i).ok_or_else(e)?;
        Ok(match token.kind {
            TokenKind::LeftBrace => {
                Statement::Compound(parse_compound_statement(i, tokens)?)
            }
            TokenKind::If => {
                Statement::Selection(parse_selection_statement(i, tokens)?)
            }
            TokenKind::Do | TokenKind::For | TokenKind::While => {
                Statement::Iteration(parse_iteration_statement(i, tokens)?)
            }
            TokenKind::Continue | TokenKind::Break | TokenKind::Return => {
                Statement::Jump(parse_jump_statement(i, tokens)?)
            }
            _ => Statement::Expression(parse_expression_statement(i, tokens)?),
        })
    }
    #[trace::trace]
    pub(crate) fn parse_compound_statement(
        i: &mut usize,
        tokens: &[Token],
    ) -> Result<CompoundStatement, Error> {
        let e = || error(*i, tokens, ErrorKind::ExpectCompoundStatement);
        let token = tokens.get(*i).ok_or_else(e)?;
        if let TokenKind::LeftBrace = token.kind {
            let left_pos = token.pos;
            *i += 1;
            let mut list = vec![];
            loop {
                let token = tokens.get(*i).map(|t| &t.kind);
                if let Some(TokenKind::RightBrace) = token {
                    *i += 1;
                    break;
                } else {
                    if tokens.get(*i).is_none() {
                        return Err(error(
                            *i,
                            tokens,
                            ErrorKind::UnmatchedParenthesis(left_pos),
                        ));
                    }
                    list.push(parse_block_item(i, tokens)?);
                }
            }
            Ok(CompoundStatement(list))
        } else {
            Err(e())
        }
    }

    fn parse_block_item(
        i: &mut usize, // cannot get none
        tokens: &[Token],
    ) -> Result<BlockItem, Error> {
        Ok(if let TokenKind::Qualifier(_) = &tokens[*i].kind {
            BlockItem::Declaration(parse_declaration(i, tokens)?)
        } else {
            BlockItem::Statement(parse_statement(i, tokens)?)
        })
    }

    #[trace::trace]
    fn parse_expression_statement(
        i: &mut usize,
        tokens: &[Token],
    ) -> Result<Option<Expression>, Error> {
        let token = tokens.get(*i).unwrap();
        Ok(if let TokenKind::Semicolon = &token.kind {
            *i += 1;
            None
        } else {
            let ret = Some(parse_expression(i, tokens)?);
            parse_left::<';'>(i, tokens)?;
            ret
        })
    }

    #[trace::trace]
    fn parse_selection_statement(
        i: &mut usize,
        tokens: &[Token],
    ) -> Result<SelectionStatement, Error> {
        let token = tokens.get(*i).unwrap();
        assert_matches!(token.kind, TokenKind::If);
        *i += 1;
        let left_pos = parse_left::<'('>(i, tokens)?;
        let condition = parse_expression(i, tokens)?;
        parse_right::<')'>(i, tokens, left_pos)?;
        let consequent = Box::new(parse_statement(i, tokens)?);
        let token = tokens.get(*i);
        let alternative = if let Some(TokenKind::Else) = token.map(|t| &t.kind)
        {
            Some(Box::new(parse_statement(i, tokens)?))
        } else {
            None
        };
        Ok(SelectionStatement {
            condition,
            consequent,
            alternative,
        })
    }

    fn parse_iteration_statement(
        i: &mut usize,
        tokens: &[Token],
    ) -> Result<IterationStatement, Error> {
        let token = tokens.get(*i).unwrap();
        Ok(match token.kind {
            TokenKind::While => {
                IterationStatement::While(parse_while_statement(i, tokens)?)
            }
            TokenKind::For => {
                IterationStatement::For(parse_for_statement(i, tokens)?)
            }
            TokenKind::Do => {
                IterationStatement::Do(parse_do_statement(i, tokens)?)
            }
            _ => {
                unreachable!()
            }
        })
    }

    fn parse_while_statement(
        i: &mut usize,
        tokens: &[Token],
    ) -> Result<WhileStatement, Error> {
        *i += 1;
        let left_pos = parse_left::<'('>(i, tokens)?;
        let condition = parse_expression(i, tokens)?;
        parse_right::<')'>(i, tokens, left_pos)?;
        let body = Box::new(parse_statement(i, tokens)?);
        Ok(WhileStatement { condition, body })
    }

    #[trace::trace]
    fn parse_for_statement(
        i: &mut usize,
        tokens: &[Token],
    ) -> Result<ForStatement, Error> {
        *i += 1;
        let left_pos = parse_left::<'('>(i, tokens)?;
        let e = || error(*i, tokens, ErrorKind::ExpectForInitialization);
        let token = tokens.get(*i).ok_or_else(e)?;
        let initialization = match &token.kind {
            TokenKind::Qualifier(_) => {
                ForInitialization::Declaration(parse_declaration(i, tokens)?)
            }
            _ => ForInitialization::Expression(parse_expression_statement(
                i, tokens,
            )?),
        };
        let condition = parse_expression_statement(i, tokens)?;

        let e = || error(*i, tokens, ErrorKind::UnmatchedParenthesis(left_pos));
        let token = tokens.get(*i).ok_or_else(e)?;
        let update = if let TokenKind::RightParen = &token.kind {
            None
        } else {
            Some(parse_expression(i, tokens)?)
        };
        parse_right::<')'>(i, tokens, left_pos)?;
        let body = Box::new(parse_statement(i, tokens)?);
        Ok(ForStatement {
            initialization,
            condition,
            update,
            body,
        })
    }

    fn parse_do_statement(
        i: &mut usize,
        tokens: &[Token],
    ) -> Result<DoStatement, Error> {
        *i += 1;
        let body = Box::new(parse_statement(i, tokens)?);
        let e = || error(*i, tokens, ErrorKind::ExpectStr("while".to_owned()));
        let token = tokens.get(*i).ok_or_else(e)?;
        let TokenKind::While = &token.kind else {
            return Err(e())
        };
        let left_pos = parse_left::<'('>(i, tokens)?;
        let condition = parse_expression(i, tokens)?;
        parse_right::<')'>(i, tokens, left_pos)?;
        Ok(DoStatement { body, condition })
    }

    #[trace::trace]
    fn parse_jump_statement(
        i: &mut usize,
        tokens: &[Token],
    ) -> Result<JumpStatement, Error> {
        let token = tokens.get(*i).unwrap();
        Ok(match &token.kind {
            TokenKind::Continue => {
                *i += 1;
                parse_left::<';'>(i, tokens)?;
                JumpStatement::Continue
            }
            TokenKind::Break => {
                *i += 1;
                parse_left::<';'>(i, tokens)?;
                JumpStatement::Break
            }
            TokenKind::Return => {
                *i += 1;
                JumpStatement::Return(parse_expression_statement(i, tokens)?)
            }
            _ => unreachable!(),
        })
    }
}

mod expr {

    use super::*;

    fn parse_left_associate_binary_expr<
        F: Fn(&mut usize, &[Token]) -> Result<Box<Expression>, Error>,
    >(
        i: &mut usize,
        tokens: &[Token],
        operators: &'static [TokenKind],
        parse: F,
    ) -> Result<Box<Expression>, Error> {
        let mut ret = parse(i, tokens)?;
        loop {
            let token = tokens.get(*i);
            if let Some(k) = token.map(|t| &t.kind) {
                if let Some(op) = operators.iter().find(|&op| op == k) {
                    let operator: BinaryOperator = op.into();
                    *i += 1;
                    let right = parse(i, tokens)?;
                    ret = Box::new(Expression::Binary(BinaryExpression {
                        operator,
                        left: ret,
                        right,
                    }))
                } else {
                    return Ok(ret);
                }
            } else {
                return Ok(ret);
            }
        }
    }

    #[trace::trace]
    pub(crate) fn parse_expression(
        i: &mut usize,
        tokens: &[Token],
    ) -> Result<Expression, Error> {
        parse_left_associate_binary_expr(
            i,
            tokens,
            &[TokenKind::Comma],
            parse_assignment_expression,
        )
        .map(|e| *e)
    }

    pub(crate) fn parse_assignment_expression(
        i: &mut usize,
        tokens: &[Token],
    ) -> Result<Box<Expression>, Error> {
        let i_saved = *i;
        let left = parse_unary_expression(i, tokens)?;
        let e = || error(*i, tokens, ErrorKind::ExpectStr('='.into()));
        let token = tokens.get(*i).ok_or_else(e)?; //todo
        println!("=? {token:?}");
        let TokenKind::Relop(RelopKind::Assign) = &token.kind else {
                *i = i_saved;
                return parse_logical_or_expression(i, tokens);
            };
        *i += 1;
        let right = parse_assignment_expression(i, tokens)?;
        Ok(Box::new(Expression::Assignment(AssignmentExpression {
            left,
            right,
        })))
    }

    #[trace::trace]
    fn parse_logical_or_expression(
        i: &mut usize,
        tokens: &[Token],
    ) -> Result<Box<Expression>, Error> {
        parse_left_associate_binary_expr(
            i,
            tokens,
            &[TokenKind::Or],
            parse_logical_and_expression,
        )
    }

    fn parse_logical_and_expression(
        i: &mut usize,
        tokens: &[Token],
    ) -> Result<Box<Expression>, Error> {
        parse_left_associate_binary_expr(
            i,
            tokens,
            &[TokenKind::And],
            parse_or_expression,
        )
    }

    fn parse_or_expression(
        i: &mut usize,
        tokens: &[Token],
    ) -> Result<Box<Expression>, Error> {
        parse_left_associate_binary_expr(
            i,
            tokens,
            &[TokenKind::BitOr],
            parse_and_expression,
        )
    }

    fn parse_and_expression(
        i: &mut usize,
        tokens: &[Token],
    ) -> Result<Box<Expression>, Error> {
        parse_left_associate_binary_expr(
            i,
            tokens,
            &[TokenKind::BitAnd],
            parse_equality_expression,
        )
    }

    fn parse_equality_expression(
        i: &mut usize,
        tokens: &[Token],
    ) -> Result<Box<Expression>, Error> {
        parse_left_associate_binary_expr(
            i,
            tokens,
            &[
                TokenKind::Relop(RelopKind::Eq),
                TokenKind::Relop(RelopKind::Neq),
            ],
            parse_relational_expression,
        )
    }

    #[trace::trace]
    fn parse_relational_expression(
        i: &mut usize,
        tokens: &[Token],
    ) -> Result<Box<Expression>, Error> {
        parse_left_associate_binary_expr(
            i,
            tokens,
            &[
                TokenKind::Relop(RelopKind::Le),
                TokenKind::Relop(RelopKind::Lt),
                TokenKind::Relop(RelopKind::Ge),
                TokenKind::Relop(RelopKind::Gt),
            ],
            parse_additive_expression,
        )
    }

    #[trace::trace]
    fn parse_additive_expression(
        i: &mut usize,
        tokens: &[Token],
    ) -> Result<Box<Expression>, Error> {
        parse_left_associate_binary_expr(
            i,
            tokens,
            &[TokenKind::Plus, TokenKind::Minus],
            parse_multipplicative_expression,
        )
    }

    fn parse_multipplicative_expression(
        i: &mut usize,
        tokens: &[Token],
    ) -> Result<Box<Expression>, Error> {
        parse_left_associate_binary_expr(
            i,
            tokens,
            &[TokenKind::Star, TokenKind::Divide],
            parse_unary_expression,
        )
    }

    fn first_unary_operator(token: &Token) -> bool {
        [
            TokenKind::BitAnd,
            TokenKind::Star,
            TokenKind::Plus,
            TokenKind::Minus,
            TokenKind::Not,
            TokenKind::Not,
        ]
        .iter()
        .any(|t| t == &token.kind)
    }

    #[trace::trace]
    pub(crate) fn parse_unary_expression(
        i: &mut usize,
        tokens: &[Token],
    ) -> Result<Box<Expression>, Error> {
        let e = || error(*i, tokens, ErrorKind::ExpectExpression);
        let token = tokens.get(*i).ok_or_else(e)?;
        if first_unary_operator(token) {
            let operator: UnaryOperator = (&token.kind).into();
            *i += 1;
            let operand = parse_unary_expression(i, tokens)?;
            return Ok(Box::new(Expression::Unary(UnaryExpression {
                operator,
                operand,
            })));
        }
        parse_postfix_expression(i, tokens)
    }

    #[trace::trace]
    fn parse_postfix_expression(
        i: &mut usize,
        tokens: &[Token],
    ) -> Result<Box<Expression>, Error> {
        let mut ret = parse_primary_expression(i, tokens)?;

        loop {
            let token = tokens.get(*i);
            let left_pos = token.map(|t| t.pos);
            match token.map(|t| &t.kind) {
                Some(TokenKind::LeftSqBracket) => {
                    *i += 1;
                    let subscript = Box::new(parse_expression(i, tokens)?);
                    ret = Box::new(Expression::Postfix(PostfixExpression {
                        operand: ret,
                        postfix: PostfixExpressionPostfix::Subscript(subscript),
                    }));
                    parse_right::<']'>(i, tokens, left_pos.unwrap())?;
                }
                Some(TokenKind::LeftParen) => {
                    *i += 1;
                    let arguments = parse_argument_expression_list(i, tokens)?;
                    let token = tokens.get(*i);
                    if let Some(TokenKind::RightParen) = token.map(|t| &t.kind)
                    {
                        ret =
                            Box::new(Expression::Postfix(PostfixExpression {
                                operand: ret,
                                postfix: PostfixExpressionPostfix::Call(vec![]),
                            }));
                        continue;
                    }
                    ret = Box::new(Expression::Postfix(PostfixExpression {
                        operand: ret,
                        postfix: PostfixExpressionPostfix::Call(arguments),
                    }));
                    parse_right::<']'>(i, tokens, left_pos.unwrap())?;
                }
                _ => return Ok(ret),
            }
        }
    }

    fn parse_argument_expression_list(
        i: &mut usize,
        tokens: &[Token],
    ) -> Result<Vec<Expression>, Error> {
        parse_non_empty_list::<',', _, _>(
            i,
            tokens,
            parse_assignment_expression,
        )
        .map(|v| v.into_iter().map(|b| *b).collect())
    }

    #[trace::trace]
    fn parse_primary_expression(
        i: &mut usize,
        tokens: &[Token],
    ) -> Result<Box<Expression>, Error> {
        let e = || error(*i, tokens, ErrorKind::ExpectExpression);
        let token = tokens.get(*i).ok_or_else(e)?;
        Ok(Box::new(match &token.kind {
            TokenKind::Id(id) => {
                *i += 1;
                Expression::Atom(AtomExpression::Identifier(*id))
            }
            TokenKind::IntegerConstant(n) => {
                *i += 1;
                Expression::Atom(AtomExpression::Integer(*n))
            }
            TokenKind::FloatingConstant(n) => {
                *i += 1;
                Expression::Atom(AtomExpression::Floating(*n))
            }
            TokenKind::LeftParen => {
                *i += 1;
                let left_pos = token.pos;
                let expression = parse_expression(i, tokens)?;
                parse_right::<')'>(i, tokens, left_pos)?;
                expression
            }
            _ => return Err(e()),
        }))
    }
}

#[cfg(test)]
mod test {
    use crate::{lexer::scan, preprocess::preprocess, token::Token};
    use anyhow::Result;

    use super::parse_translation_unit;
    fn str_to_tokens(s: &str) -> Vec<Token> {
        let s = preprocess(s.char_indices()).unwrap();
        scan(&s).unwrap().tokens
    }

    #[test]
    fn test() -> Result<()> {
        let tokens = str_to_tokens("");
        let mut i = 0;
        parse_translation_unit(&mut i, &tokens)?;
        Ok(())
    }
}
