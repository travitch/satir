use std::collections::BTreeMap;

use combine::error::ParseError;
use combine::stream::position;
use combine::parser::char;
use combine::parser::choice;
use combine::parser::repeat;
use combine::parser::token;
use combine::{Parser,Stream,EasyParser};

use crate::satir::core;
use crate::satir::core::Variable;
use crate::satir::clause;

/// A parser for whitespace between tokens
///
/// This does not cover comments, which are a bit special in DIMACS (they are
/// handled at the top level)
fn whitespace<Input>() -> impl Parser<Input, Output = ()>
where
    Input : Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    repeat::skip_many(char::space())
}

fn line_end<Input>() -> impl Parser<Input, Output = ()>
where
    Input : Stream<Token = char>
{
    choice::or(char::newline(), char::crlf()).map(|_| ())
}

/// Parse a vector of char into a u32; fails loudly if they are not actually digits
fn digits_to_u32(digits : Vec<char>) -> u32 {
    digits.into_iter().collect::<String>().parse().unwrap()
}

fn number<Input>() -> impl Parser<Input, Output = u32>
where
    Input : Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    // As long as the many1 combined succeeds, digitsToU32 cannot fail
    repeat::many1::<Vec<_>, _, _>(char::digit()).map(digits_to_u32)
}

#[derive(Debug, PartialEq, Eq)]
struct CNFProblem {
    num_variables : u32,
    num_clauses : u32
}

/// Parse the problem description line
fn problem<Input>() -> impl Parser<Input, Output = CNFProblem>
where
    Input : Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    (char::char('p'),
     whitespace(),
     char::string("cnf"),
     whitespace(),
     number(),
     whitespace(),
     number(),
    ).map(|(_, _, _, _, nvar, _, nclause)| CNFProblem { num_variables : nvar, num_clauses : nclause })
}

/// In DIMACS, comments are a line that starts with the character 'c' until the end of the line
fn comment<Input>() -> impl Parser<Input, Output = ()>
where
    Input : Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>
{
    (char::char('c'),
     repeat::skip_until(line_end())
     ).map(|(_, _)| ())
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
struct ParsedVar(u32);

/// Literals parsed from DIMACS
///
/// Note that these are not necessarily sequential, so we need to map them to
/// our internal sequential literals.  This new type helps ensure we keep the
/// two separate
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
enum ParsedLit {
    PosLit(ParsedVar),
    NegLit(ParsedVar)
}

fn parsed_lit_var(l : &ParsedLit) -> ParsedVar {
    match l {
        ParsedLit::PosLit(v) => *v,
        ParsedLit::NegLit(v) => *v
    }
}

fn literal<Input>() -> impl Parser<Input, Output = ParsedLit>
where
    Input : Stream<Token = char>
{
    (choice::optional(char::char('-')),
     number()
     ).map(|(neg, num)| match neg {
         None => ParsedLit::PosLit(ParsedVar(num)),
         Some(_) => ParsedLit::NegLit(ParsedVar(num))
     })
}

/// A CNF clause is a sequence of whitespace-separated literals ending with a 0
///
/// Note that they may span multiple lines, and a single line may define
/// multiple clauses.
fn clause<Input>() -> impl Parser<Input, Output = Vec<ParsedLit>>
where
    Input : Stream<Token = char>
{
    // Note the extra ship of 0 at the end; repeat_until does not consume the
    // token that causes it to stop
    (choice::optional(whitespace()),
     repeat::repeat_until(literal().skip(whitespace()), char::char('0')).skip(char::char('0'))
     ).map(|(_, lits)| lits)
}

#[derive(Debug, PartialEq, Eq)]
struct ParsedDIMACS {
    cnf_problem : CNFProblem,
    clauses : Vec<Vec<ParsedLit>>
}

/// Parse an entire DIMACS file
///
/// Comment lines can be interspersed arbitrarily
///
/// The program declaration must come before the clauses
///
/// Clauses do not have to be one per line
fn dimacs<Input>() -> impl Parser<Input, Output = ParsedDIMACS>
where
    Input : Stream<Token = char>
{
    (repeat::many::<Vec<_>, _, _>(comment().skip(line_end()).with(token::value(()))),
     problem().skip(line_end()),
     repeat::many::<Vec<_>, _, _>(comment().skip(line_end()).with(token::value(()))),
     repeat::many1(clause().skip(repeat::many::<Vec<_>, _, _>(line_end()))),
     token::eof()
    ).map(|(_, cnf, _, cs, _)| ParsedDIMACS { cnf_problem : cnf, clauses : cs })
}


fn to_core_lit(pl : &ParsedLit, cv : &core::Variable) -> core::Literal {
    match pl {
        ParsedLit::PosLit(_) => cv.to_positive_literal(),
        ParsedLit::NegLit(_) => cv.to_negative_literal()
    }
}

struct Env {
    var_map : BTreeMap<ParsedVar, core::Variable>,
    next_var : core::Variable,
    next_id : i64
}

fn intern_lit(env : &mut Env, pl : &ParsedLit) -> core::Literal {
    match env.var_map.get(&parsed_lit_var(pl)) {
        Some(cv) => to_core_lit(pl, cv),
        None => {
            let this_var = env.next_var;
            env.next_var = this_var.next_variable();
            env.var_map.insert(parsed_lit_var(pl), this_var);
            to_core_lit(pl, &this_var)
        }
    }
}

pub struct DIMACS {
    pub next_var : core::Variable,
    pub clauses : Vec<clause::Clause>
}

#[derive(Debug, thiserror::Error)]
enum Error<E> {
    Io(std::io::Error),
    Parse(E),
}

impl<E> std::fmt::Display for Error<E>
where
    E: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::Io(ref err) => write!(f, "{}", err),
            Error::Parse(ref err) => write!(f, "{}", err),
        }
    }
}

pub fn parse_dimacs<'a>(input : &'a str) -> anyhow::Result<DIMACS> {
    let (res, _rest) = dimacs().easy_parse(position::Stream::new(input))
        .map_err(|err| Error::Parse(err.map_range(|s| s.to_string())))?;
    let mut env = Env {
        var_map : BTreeMap::new(),
        next_var : Variable::FIRST_VARIABLE,
        next_id : 0
    };

    let mut interned_clauses = Vec::new();

    let mut clause_iter = res.clauses.iter();
    while let Some(parsed_clause) = clause_iter.next() {
        let hdr = clause::ClauseHeader {
            id : clause::ClauseId(env.next_id),
            lit_count : parsed_clause.len(),
            activity : 0.0
        };

        env.next_id = env.next_id + 1;

        let mut lits = Vec::new();
        let mut lit_iter = parsed_clause.iter();
        while let Some(parsed_lit) = lit_iter.next() {
            let core_lit = intern_lit(&mut env, parsed_lit);
            lits.push(core_lit);
        }

        interned_clauses.push(clause::Clause::new(hdr, lits));
    }

    Ok(DIMACS {
        clauses : interned_clauses,
        next_var : env.next_var
    })
}


#[test]
fn test_program_decl() {
    let result = problem().parse("p cnf 5 10").map(|t| t.0);
    let expected = CNFProblem {
        num_variables : 5,
        num_clauses : 10
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn test_program_decl_extra_ws() {
    let result = problem().parse("p  cnf\t 5 10").map(|t| t.0);
    let expected = CNFProblem {
        num_variables : 5,
        num_clauses : 10
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn test_program_decl_extra_line_end() {
    let result = problem().skip(line_end()).parse("p cnf 5 10\n").map(|t| t.0);
    let expected = CNFProblem {
        num_variables : 5,
        num_clauses : 10
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn test_clause() {
    let result = clause().parse("1 -5 \t11   2 0").map(|t| t.0);
    let expected = vec![ParsedLit::PosLit(ParsedVar(1)),
                        ParsedLit::NegLit(ParsedVar(5)),
                        ParsedLit::PosLit(ParsedVar(11)),
                        ParsedLit::PosLit(ParsedVar(2))
    ];

    assert_eq!(result, Ok(expected));
}

#[test]
fn test_clause_leading_ws() {
    let result = clause().parse("  1 -5 \t11   2 0").map(|t| t.0);
    let expected = vec![ParsedLit::PosLit(ParsedVar(1)),
                        ParsedLit::NegLit(ParsedVar(5)),
                        ParsedLit::PosLit(ParsedVar(11)),
                        ParsedLit::PosLit(ParsedVar(2))
    ];

    assert_eq!(result, Ok(expected));
}

#[test]
fn test_dimacs1() {
    let result = dimacs().parse("c Header\n\
p cnf 5 2\n\
c commentary\n\
1 5 2 -1 0\n\
-5 3 0\n").map(|t| t.0);
    let expected = ParsedDIMACS {
        cnf_problem : CNFProblem {
            num_variables : 5,
            num_clauses : 2
        },
        clauses : vec![
            vec![ParsedLit::PosLit(ParsedVar(1)),
                 ParsedLit::PosLit(ParsedVar(5)),
                 ParsedLit::PosLit(ParsedVar(2)),
                 ParsedLit::NegLit(ParsedVar(1))],
            vec![ParsedLit::NegLit(ParsedVar(5)),
                 ParsedLit::PosLit(ParsedVar(3))
            ]
        ]
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn test_dimacs_empty_comment() {
    let result = dimacs().parse("c Header\n\
c\n\
p cnf 5 2\n\
c .commentary\n\
1 5 2     -1 0\n\
-5 3 0\n").map(|t| t.0);
    let expected = ParsedDIMACS {
        cnf_problem : CNFProblem {
            num_variables : 5,
            num_clauses : 2
        },
        clauses : vec![
            vec![ParsedLit::PosLit(ParsedVar(1)),
                 ParsedLit::PosLit(ParsedVar(5)),
                 ParsedLit::PosLit(ParsedVar(2)),
                 ParsedLit::NegLit(ParsedVar(1))],
            vec![ParsedLit::NegLit(ParsedVar(5)),
                 ParsedLit::PosLit(ParsedVar(3))
            ]
        ]
    };

    assert_eq!(result, Ok(expected));
}

#[test]
fn test_dimacs_trailing_newline() {
    let result = dimacs().parse("c Header\n\
p cnf 5 2\n\
c commentary\n\
1 5 2 -1 0\n\
-5 3 0\n\n\n").map(|t| t.0);
    let expected = ParsedDIMACS {
        cnf_problem : CNFProblem {
            num_variables : 5,
            num_clauses : 2
        },
        clauses : vec![
            vec![ParsedLit::PosLit(ParsedVar(1)),
                 ParsedLit::PosLit(ParsedVar(5)),
                 ParsedLit::PosLit(ParsedVar(2)),
                 ParsedLit::NegLit(ParsedVar(1))],
            vec![ParsedLit::NegLit(ParsedVar(5)),
                 ParsedLit::PosLit(ParsedVar(3))
            ]
        ]
    };

    assert_eq!(result, Ok(expected));
}
