mod parse;

use parse::*;

use super::types::*;
use std::str;
use std::str::FromStr;

pub use parse::Error;

impl FromStr for Prop {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse(s, top_level)
    }
}

// top_level ::= proposition EOI
fn top_level<'a>(input: Input<'a>) -> Result<Output<'a, Prop>, Error> {
    let (input, ()) = skip_spaces(input)?;
    let (input, proposition) = proposition(input)?;
    let (input, ()) = end_of_input(input)?;
    accept(input, proposition)
}

// proposition ::= expression { implication }
fn proposition<'a>(input: Input<'a>) -> Result<Output<'a, Prop>, Error> {
    let (input, expression) = expression(input)?;
    let (input, rest) = repeat(input, implication)?;

    let result = match rest.into_iter().rev().reduce(|b, a| a.implies(&b)) {
        None => expression,
        Some(conclusion) => expression.implies(&conclusion),
    };
    accept(input, result)
}

// implication ::= '->' expression
fn implication<'a>(input: Input<'a>) -> Result<Output<'a, Prop>, Error> {
    let (input, _) = expect(input, |c| c == '-', "-")?;
    let (input, _) = expect(input, |c| c == '>', ">")?;
    let (input, ()) = skip_spaces(input)?;
    expression(input)
}

// expression ::= term { disjuction }
fn expression<'a>(input: Input<'a>) -> Result<Output<'a, Prop>, Error> {
    let (input, term) = term(input)?;
    let (input, rest) = repeat(input, disjuction)?;
    accept(input, rest.into_iter().fold(term, |a, b| a.or(&b)))
}

// disjuction ::= '|' term
fn disjuction<'a>(input: Input<'a>) -> Result<Output<'a, Prop>, Error> {
    let (input, _) = expect(input, |c| c == '|', "|")?;
    let (input, ()) = skip_spaces(input)?;
    term(input)
}

// term ::= factor { conjunction }
fn term<'a>(input: Input<'a>) -> Result<Output<'a, Prop>, Error> {
    let (input, factor) = factor(input)?;
    let (input, rest) = repeat(input, conjuction)?;
    accept(input, rest.into_iter().fold(factor, |a, b| a.and(&b)))
}

// conjuction ::= '&' factor
fn conjuction<'a>(input: Input<'a>) -> Result<Output<'a, Prop>, Error> {
    let (input, _) = expect(input, |c| c == '&', "&")?;
    let (input, ()) = skip_spaces(input)?;
    factor(input)
}

// factor ::= atom
//         |  negation
//         |  parentheses
fn factor<'a>(input: Input<'a>) -> Result<Output<'a, Prop>, Error> {
    choose(input, &[atom, negation, parentheses], "proposition")
}

// atom ::= word
fn atom<'a>(input: Input<'a>) -> Result<Output<'a, Prop>, Error> {
    let (input, word) = word(input)?;
    accept(input, Prop::atom(&word))
}

// negation ::= '~' factor
fn negation<'a>(input: Input<'a>) -> Result<Output<'a, Prop>, Error> {
    let (input, _) = expect(input, |c| c == '~', "~")?;
    let (input, ()) = skip_spaces(input)?;
    let (input, prop) = factor(input)?;
    accept(input, Prop::not(&prop))
}

// parentheses ::= '(' proposition ')'
fn parentheses<'a>(input: Input<'a>) -> Result<Output<'a, Prop>, Error> {
    let (input, _) = expect(input, |c| c == '(', "(")?;
    let (input, ()) = skip_spaces(input)?;
    let (input, prop) = proposition(input)?;
    let (input, _) = expect(input, |c| c == ')', ")")?;
    let (input, ()) = skip_spaces(input)?;
    accept(input, prop)
}

// word ::= letter { letter }
fn word<'a>(input: Input<'a>) -> Result<Output<'a, String>, Error> {
    let (input, l) = letter(input)?;
    let (input, ls) = repeat(input, letter)?;
    let (input, ()) = skip_spaces(input)?;
    accept(input, [l].into_iter().chain(ls).collect())
}

// letter ::= 'a' | .. | 'z' | 'A' | .. | 'Z'
fn letter<'a>(input: Input<'a>) -> Result<Output<'a, char>, Error> {
    expect(input, |c| c.is_ascii_alphabetic(), "letter")
}

// skip_spaces ::= { space }
fn skip_spaces<'a>(input: Input<'a>) -> Result<Output<'a, ()>, Error> {
    let (input, _) = repeat(input, space)?;
    accept(input, ())
}

// space ::= ' '
fn space<'a>(input: Input<'a>) -> Result<Output<'a, char>, Error> {
    expect(input, |c| c.is_whitespace(), "whitespace")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str_atom() {
        assert_eq!(round_trip("a"), "a");
    }

    #[test]
    fn from_str_negation() {
        assert_eq!(round_trip("~a"), "¬a");
    }

    #[test]
    fn from_str_disjunction() {
        assert_eq!(round_trip("a | b"), "a ∨ b");
    }

    #[test]
    fn from_str_conjunction() {
        assert_eq!(round_trip("a & b"), "a ∧ b");
    }

    #[test]
    fn from_str_implication() {
        assert_eq!(round_trip("a -> b"), "¬a ∨ b");
    }

    #[test]
    fn from_str_implications() {
        assert_eq!(round_trip("a -> b -> c -> d"), "¬a ∨ ¬b ∨ ¬c ∨ d");
    }

    #[test]
    fn from_str_parentheses() {
        assert_eq!(round_trip("(a)"), "a");
    }

    #[test]
    fn from_str_proposition() {
        assert_eq!(round_trip("~a -> b & (f | c)"), "¬¬a ∨ b ∧ (f ∨ c)");
    }

    #[test]
    fn from_str_complicated_proposition() {
        assert_eq!(
            round_trip("~a -> b & (f | ~c) -> d & ~~e"),
            "¬¬a ∨ ¬(b ∧ (f ∨ ¬c)) ∨ d ∧ ¬¬e"
        );
    }

    #[test]
    fn from_str_rich_landed_saintly() {
        assert_eq!(
            round_trip(
                "(landed -> rich) & ~(saintly & rich) -> (landed -> ~saintly)"
            ),
            "¬((¬landed ∨ rich) ∧ ¬(saintly ∧ rich)) ∨ ¬landed ∨ ¬saintly"
        );
    }

    #[test]
    fn from_str_error_end_of_input() {
        match Prop::from_str("a b") {
            Err(Error { position, message }) => {
                assert_eq!(position, 2);
                assert_eq!(message, "unexpected input");
            }
            Ok(_) => unreachable!("expected error in input string"),
        }
    }

    #[test]
    fn from_str_error_expected_closing_parenthesis() {
        match Prop::from_str("(a") {
            Err(Error { position, message }) => {
                assert_eq!(position, 2);
                assert_eq!(message, "unexpected end of input, expected )");
            }
            Ok(_) => unreachable!("expected error in input string"),
        }
    }

    #[test]
    fn from_str_error_expected_operator() {
        match Prop::from_str("(a b") {
            Err(Error { position, message }) => {
                assert_eq!(position, 3);
                assert_eq!(message, "unexpected b, expected )");
            }
            Ok(_) => unreachable!("expected error in input string"),
        }
    }

    #[test]
    fn from_str_error_expected_proposition() {
        match Prop::from_str("a | b &") {
            Err(Error { position, message }) => {
                assert_eq!(position, 7);
                assert_eq!(message, "expected proposition");
            }
            Ok(_) => unreachable!("expected error in input string"),
        }
    }

    #[test]
    fn from_str_error_implication() {
        match Prop::from_str("a - b") {
            Err(Error { position, message }) => {
                assert_eq!(position, 3);
                assert_eq!(message, "unexpected  , expected >");
            }
            Ok(_) => unreachable!("expected error in input string"),
        }
    }

    fn round_trip(text: &str) -> String {
        Prop::from_str(text)
            .into_iter()
            .fold(String::new(), |_, prop| prop.to_string())
    }
}
