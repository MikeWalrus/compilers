use anyhow::anyhow;
use anyhow::Result;
use std::collections::HashSet;
use std::io;
use std::io::BufRead;
use std::io::StdinLock;
use std::ptr;
use std::slice;

#[derive(Debug)]
enum Symbol {
    Terminal(char),
    NonTerminal(char),
}

impl Symbol {
    fn from_char(c: char, v_n: &HashSet<char>) -> Self {
        if v_n.contains(&c) {
            Symbol::NonTerminal(c)
        } else {
            Symbol::Terminal(c)
        }
    }
}

fn symbol_sequence(s: &str, v_n: &HashSet<char>) -> Vec<Symbol> {
    s.chars().map(|c| Symbol::from_char(c, v_n)).collect()
}

#[derive(Debug)]
enum GrammarType {
    PhraseStructure,
    ContextSensitive,
    ContextFree,
    Regular,
}

#[derive(Debug)]
struct Production {
    head: Vec<Symbol>,
    body: Vec<Vec<Symbol>>,
}

impl Production {
    fn try_from_str(s: &str, v_n: &HashSet<char>) -> Result<Self> {
        let mut i = s.split("::=");
        let head = symbol_sequence(
            i.next().ok_or_else(|| anyhow!("Head not found: {s}"))?,
            v_n,
        );
        let body = i
            .next()
            .ok_or_else(|| anyhow!("Body not found: {s}"))?
            .split('|')
            .map(|s| symbol_sequence(s, v_n))
            .collect();
        Ok(Production { head, body })
    }
}

fn main() -> Result<()> {
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    let line = get_line(&mut buffer, &mut handle)?;
    let s = Symbol::NonTerminal(start_symbol(line)?);
    println!("S: {:?}", s);

    let line = get_line(&mut buffer, &mut handle)?;
    let v_n = non_terminal_symbols(line)?;
    println!("v_n: {:?}", v_n);

    let mut productions = Vec::new();
    loop {
        let line = get_line(&mut buffer, &mut handle)?;

        if line.is_empty() {
            break;
        }

        let p = Production::try_from_str(line, &v_n)?;
        println!("p: {:?}", p);
        productions.push(p);
    }

    println!("{:?}", classify(&productions));

    Ok(())
}

fn get_line<'a>(
    buf: &'a mut String,
    handle: &mut StdinLock<'static>,
) -> Result<&'a str> {
    buf.clear();
    handle.read_line(buf)?;
    Ok(buf.trim())
}

fn start_symbol(s: &str) -> Result<char> {
    let mut i = s.chars();
    let err = || anyhow!("Grammar expected. e.g. G[N]\nGot: {}", s);
    let left = "G[";
    let mut j = left.chars();
    loop {
        match j.next() {
            Some(c) => {
                if c != i.next().ok_or_else(err)? {
                    return Err(err());
                }
            }
            None => return i.next().ok_or_else(err),
        }
    }
}

fn non_terminal_symbols(s: &str) -> Result<HashSet<char>> {
    s.split(',')
        .map(|x| {
            let mut i = x.trim().chars();
            let c =
                i.next().ok_or_else(|| anyhow!("Empty symbol found: {s}"))?;
            if i.next().is_some() {
                Err(anyhow!("{x} is too long."))
            } else {
                Ok(c)
            }
        })
        .collect()
}

fn is_grammar(p: &[Production]) -> bool {
    p.iter()
        .all(|x| x.head.iter().any(|a| matches!(a, Symbol::NonTerminal(_))))
}

fn is_context_sensitive(p: &[Production]) -> bool {
    p.iter().all(|x| {
        let a = x.head.len();
        a >= 1
            && x.body.iter().all(|s| {
                let b = s.len();
                a <= b
            })
    })
}

fn is_context_free(p: &[Production]) -> bool {
    p.iter().all(|x| x.head.len() == 1)
}

fn is_regular(p: &[Production]) -> bool {
    let is_left_linear = is_linear(p, |i| i, <[Symbol]>::last);
    let is_right_linear = is_linear(p, |i| i.rev(), <[Symbol]>::first);
    is_left_linear || is_right_linear
}

fn classify(p: &[Production]) -> Option<GrammarType> {
    if !is_grammar(p) {
        None
    } else if !is_context_sensitive(p) {
        Some(GrammarType::PhraseStructure)
    } else if !is_context_free(p) {
        Some(GrammarType::ContextSensitive)
    } else if !is_regular(p) {
        Some(GrammarType::ContextFree)
    } else {
        Some(GrammarType::Regular)
    }
}

fn is_linear<'a, J: Iterator<Item = &'a Symbol>>(
    p: &'a [Production],
    r: fn(slice::Iter<'a, Symbol>) -> J,
    f: for<'r> fn(&'r [Symbol]) -> Option<&'r Symbol>,
) -> bool {
    p.iter().all(|production| {
        production.body.iter().all(|body| {
            r(body.iter())
                .find(|s| matches!(s, Symbol::NonTerminal(_)))
                .map_or(true, |x| ptr::eq(x, f(body).unwrap()))
        })
    })
}
