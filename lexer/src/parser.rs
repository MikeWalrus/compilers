use crate::error::ErrorKind;
use crate::token::{self, *};
use crate::{ast::*, error::Error, token::Token};
use util::*;

mod util {
    use super::*;

    pub fn parse_right<const T: char>(
        i: &mut usize,
        tokens: &[Token],
        left_pos: Position,
    ) -> Result<(), Error> {
        let e = || Error {
            error_kind: ErrorKind::UnmatchedParenthesis(left_pos),
            pos: left_pos,
        };
        let token = &tokens.get(*i).ok_or_else(e)?.kind;
        if T == ')' {
            matches!(TokenKind::RightParen, token)
        } else if T == ']' {
            matches!(TokenKind::RightSqBracket, token)
        } else if T == '}' {
            matches!(TokenKind::RightBrace, token)
        } else {
            unreachable!()
        }
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
        let token = tokens.get(*i);
        while let Some(t) = token {
            if matches!(&t.kind, delimiter) {
                *i += 1;
                let item = parse(i, tokens)?;
                v.push(item);
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
    v.push(parse_external_declaration(i, tokens)?);
    Ok(TranslationUnit {
        external_declarations: v,
    })
}

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
        Some(TokenKind::LeftParen) => {
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
        None => Err(error(*i, tokens, ErrorKind::ExpectChar("; or {"))),
    }
}

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
    Ok(Declaration {
        declaration_specifier,
        init_declarator_list,
    })
}

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
        let expr = expr::parse_assignment_expr(i, tokens)?;
        Initializer::Expression(expr)
    })
}

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

// Identifier DirectDeclaratorModifiers
// `(` Declarator `)` DirectDeclaratorModifiers
fn parse_direct_declarator(
    i: &mut usize,
    tokens: &[Token],
) -> Result<DirectDeclarator, Error> {
    let e = || error(*i, tokens, ErrorKind::ExpectDirectDeclarator);
    let token = tokens.get(*i).ok_or_else(e)?;
    let simple_declarator = match token.kind {
        TokenKind::LeftParen => SimpleDirectDeclarator::Declarator(Box::new(
            parse_declarator(i, tokens)?,
        )),
        TokenKind::Id(identifier) => {
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
    use super::*;
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
        Ok(if let TokenKind::Qualifier(q) = &tokens[*i].kind {
            BlockItem::Declaration(parse_declaration(i, tokens)?)
        } else {
            BlockItem::Statement(parse_statement(i, tokens)?)
        })
    }

    fn parse_statement(
        i: &mut usize,
        tokens: &[Token],
    ) -> Result<Statement, Error> {
        todo!()
    }
}

mod expr {
    use super::*;
    pub(crate) fn parse_assignment_expr(
        i: &mut usize,
        tokens: &[Token],
    ) -> Result<Expression, Error> {
        todo!()
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
