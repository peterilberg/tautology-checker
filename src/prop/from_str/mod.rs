use super::types::*;
use std::str::FromStr;

#[macro_use]
mod parse;

use parse::*;

impl FromStr for Prop {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let scanner = Scanner::try_from(CharSource::new(s))?;
        let lexer = Lexer::try_from(TokenSource::new(scanner))?;
        Parser::parse(lexer)
    }
}

// TokenSource for propositions.
struct TokenSource<'a> {
    scanner: Scanner<'a>,
}

impl<'a> TokenSource<'a> {
    fn new(scanner: Scanner<'a>) -> Self {
        TokenSource { scanner }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Token<'source> {
    EndOfInput,
    Name(&'source str),
    Or,
    And,
    Not,
    Implies,
    Open,
    Close,
}

impl<'a> Source for TokenSource<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Result<Next<Self::Item>, Error> {
        // skip_spaces ::= { ' ' }
        while let Some(c) = self.scanner.item()
            && c.is_ascii_whitespace()
        {
            self.scanner.advance()?;
        }

        let start = self.scanner.offset();
        match self.scanner.item() {
            // end_of_input ::= EOI
            None => accept(Next::Value(start, Token::EndOfInput)),
            Some(next) => {
                self.scanner.advance()?;
                match next {
                    // word ::= letter { letter }
                    // letter ::= 'a' | .. | 'z' | 'A' | .. | 'Z'
                    c if c.is_ascii_alphabetic() => {
                        while let Some(c) = self.scanner.item()
                            && c.is_ascii_alphabetic()
                        {
                            self.scanner.advance()?;
                        }
                        let end = self.scanner.offset();
                        let slice = self.scanner.slice(start, end);
                        accept(Next::Value(start, Token::Name(slice)))
                    }
                    // special ::= '|' | '&' | '~' | '(' | ')' | '->'
                    '|' => accept(Next::Value(start, Token::Or)),
                    '&' => accept(Next::Value(start, Token::And)),
                    '~' => accept(Next::Value(start, Token::Not)),
                    '(' => accept(Next::Value(start, Token::Open)),
                    ')' => accept(Next::Value(start, Token::Close)),
                    '-' => match self.scanner.item() {
                        Some('>') => {
                            self.scanner.advance()?;
                            accept(Next::Value(start, Token::Implies))
                        }
                        _ => reject(self.scanner.offset(), "expected ->"),
                    },
                    // Anything else is an error.
                    _ => reject(start, "invalid character"),
                }
            }
        }
    }
}

// Parser for propositions.
struct Parser<'source, Src>
where
    Src: Source<Item = Token<'source>>,
{
    lexer: Lexer<Src>,
}

impl<'source, Src> Parser<'source, Src>
where
    Src: Source<Item = Token<'source>>,
{
    // Parse a proposition with the help of a lexer.
    fn parse(lexer: Lexer<Src>) -> Result<Prop, Error> {
        Parser { lexer }.top_level()
    }

    // top_level ::= proposition EOI
    fn top_level(&mut self) -> Result<Prop, Error> {
        let proposition = self.proposition()?;
        expect!(self.lexer, Token::EndOfInput, "unexpected input");
        accept(proposition)
    }

    // proposition ::= expression { '->' expression }
    fn proposition(&mut self) -> Result<Prop, Error> {
        let expression = self.expression()?;
        let mut rest = Vec::new();
        repeat!(self.lexer, Token::Implies, {
            let f = self.expression()?;
            rest.push(f);
        });

        // Implications are right-associative: a -> (b -> c)
        rest.reverse();
        let result = match rest.into_iter().reduce(|b, a| a.implies(&b)) {
            None => expression,
            Some(conclusion) => expression.implies(&conclusion),
        };
        accept(result)
    }

    // expression ::= term { '|' term }
    fn expression(&mut self) -> Result<Prop, Error> {
        let mut term = self.term()?;
        repeat!(self.lexer, Token::Or, {
            let f = self.term()?;
            term = term.or(&f);
        });
        accept(term)
    }

    // term ::= factor { '&' factor }
    fn term(&mut self) -> Result<Prop, Error> {
        let mut factor = self.factor()?;
        repeat!(self.lexer, Token::And, {
            let f = self.factor()?;
            factor = factor.and(&f);
        });
        accept(factor)
    }

    // factor ::= atom
    //         |  negation
    //         |  parentheses
    fn factor(&mut self) -> Result<Prop, Error> {
        match self.lexer.item() {
            Some(Token::Name(_)) => self.atom(),
            Some(Token::Not) => self.negation(),
            Some(Token::Open) => self.parentheses(),
            _ => reject(self.lexer.offset(), "expected proposition"),
        }
    }

    // atom ::= word
    fn atom(&mut self) -> Result<Prop, Error> {
        expect!(self.lexer, Token::Name(name), "expected atom");
        accept(Prop::atom(name))
    }

    // negation ::= '~' factor
    fn negation(&mut self) -> Result<Prop, Error> {
        expect!(self.lexer, Token::Not, "expected ~");
        let prop = self.factor()?;
        accept(Prop::not(&prop))
    }

    // parentheses ::= '(' proposition ')'
    fn parentheses(&mut self) -> Result<Prop, Error> {
        expect!(self.lexer, Token::Open, "expected (");
        let prop = self.proposition()?;
        expect!(self.lexer, Token::Close, "expected )");
        accept(prop)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_source_empty_string() {
        let mut ts = token_source_from_string("");

        assert_eq!(ts.next(), accept(Next::Value(0, Token::EndOfInput)));
        assert_eq!(ts.next(), accept(Next::Value(0, Token::EndOfInput)));
    }

    #[test]
    fn token_source_name() {
        let mut ts = token_source_from_string("atom_something_else");

        assert_eq!(ts.next(), accept(Next::Value(0, Token::Name("atom"))));
    }

    #[test]
    fn token_source_skip_spaces() {
        let mut ts = token_source_from_string("    atom_something_else");

        assert_eq!(ts.next(), accept(Next::Value(4, Token::Name("atom"))));
    }

    #[test]
    fn token_source_or() {
        let mut ts = token_source_from_string("|");

        assert_eq!(ts.next(), accept(Next::Value(0, Token::Or)));
    }

    #[test]
    fn token_source_and() {
        let mut ts = token_source_from_string("&");

        assert_eq!(ts.next(), accept(Next::Value(0, Token::And)));
    }

    #[test]
    fn token_source_not() {
        let mut ts = token_source_from_string("~");

        assert_eq!(ts.next(), accept(Next::Value(0, Token::Not)));
    }

    #[test]
    fn token_source_open() {
        let mut ts = token_source_from_string("(");

        assert_eq!(ts.next(), accept(Next::Value(0, Token::Open)));
    }

    #[test]
    fn token_source_close() {
        let mut ts = token_source_from_string(")");

        assert_eq!(ts.next(), accept(Next::Value(0, Token::Close)));
    }

    #[test]
    fn token_source_implies() {
        let mut ts = token_source_from_string("->");

        assert_eq!(ts.next(), accept(Next::Value(0, Token::Implies)));
    }

    #[test]
    fn token_source_broken_implies() {
        let mut ts = token_source_from_string("-");

        assert_eq!(ts.next(), reject(1, "expected ->"));
    }

    #[test]
    fn token_source_invalid_character() {
        let mut ts = token_source_from_string("   123");

        assert_eq!(ts.next(), reject(3, "invalid character"));
    }

    #[test]
    fn parse_atom_valid() {
        let mut parser = parser_from_string("atom");

        assert_eq!(returned_value(parser.atom()), "atom");
    }

    #[test]
    fn parse_atom_invalid() {
        let mut parser = parser_from_string("&");

        assert_eq!(
            returned_error(parser.atom()),
            Error::new(0, "expected atom")
        );
    }

    #[test]
    fn parse_negation_valid() {
        let mut parser = parser_from_string("~atom");

        assert_eq!(returned_value(parser.negation()), "¬atom");
    }

    #[test]
    fn parse_negation_invalid() {
        let mut parser = parser_from_string("&");

        assert_eq!(
            returned_error(parser.negation()),
            Error::new(0, "expected ~")
        );
    }

    #[test]
    fn parse_negation_missing_proposition() {
        let mut parser = parser_from_string("~");

        assert_eq!(
            returned_error(parser.negation()),
            Error::new(1, "expected proposition")
        );
    }

    #[test]
    fn parse_parentheses_valid() {
        let mut parser = parser_from_string("(atom)");

        assert_eq!(returned_value(parser.parentheses()), "atom");
    }

    #[test]
    fn parse_parentheses_invalid() {
        let mut parser = parser_from_string("&");

        assert_eq!(
            returned_error(parser.parentheses()),
            Error::new(0, "expected (")
        );
    }

    #[test]
    fn parse_parentheses_missing_proposition() {
        let mut parser = parser_from_string("(");

        assert_eq!(
            returned_error(parser.parentheses()),
            Error::new(1, "expected proposition")
        );
    }

    #[test]
    fn parse_parentheses_missing_parenthesis() {
        let mut parser = parser_from_string("(atom");

        assert_eq!(
            returned_error(parser.parentheses()),
            Error::new(5, "expected )")
        );
    }

    #[test]
    fn parse_factor_atom() {
        let mut parser = parser_from_string("atom");

        assert_eq!(returned_value(parser.factor()), "atom");
    }

    #[test]
    fn parse_factor_negation() {
        let mut parser = parser_from_string("~atom");

        assert_eq!(returned_value(parser.factor()), "¬atom");
    }

    #[test]
    fn parse_factor_parentheses() {
        let mut parser = parser_from_string("(~atom)");

        assert_eq!(returned_value(parser.factor()), "¬atom");
    }

    #[test]
    fn parse_factor_invalid() {
        let mut parser = parser_from_string("&");

        assert_eq!(
            returned_error(parser.factor()),
            Error::new(0, "expected proposition")
        );
    }

    #[test]
    fn parse_term_simple() {
        let mut parser = parser_from_string("a");

        assert_eq!(returned_value(parser.term()), "a");
    }

    #[test]
    fn parse_term_and() {
        let mut parser = parser_from_string("a & b");

        assert_eq!(returned_value(parser.term()), "a ∧ b");
    }

    #[test]
    fn parse_term_and_and() {
        let mut parser = parser_from_string("a & b & c");

        assert_eq!(returned_value(parser.term()), "a ∧ b ∧ c");
    }

    #[test]
    fn parse_term_invalid() {
        let mut parser = parser_from_string("&");

        assert_eq!(
            returned_error(parser.term()),
            Error::new(0, "expected proposition")
        );
    }

    #[test]
    fn parse_term_missing_term() {
        let mut parser = parser_from_string("a &");

        assert_eq!(
            returned_error(parser.term()),
            Error::new(3, "expected proposition")
        );
    }

    #[test]
    fn parse_expression_simple() {
        let mut parser = parser_from_string("a");

        assert_eq!(returned_value(parser.expression()), "a");
    }

    #[test]
    fn parse_expression_and() {
        let mut parser = parser_from_string("a | b");

        assert_eq!(returned_value(parser.expression()), "a ∨ b");
    }

    #[test]
    fn parse_expression_and_and() {
        let mut parser = parser_from_string("a | b | c");

        assert_eq!(returned_value(parser.expression()), "a ∨ b ∨ c");
    }

    #[test]
    fn parse_expression_invalid() {
        let mut parser = parser_from_string("&");

        assert_eq!(
            returned_error(parser.expression()),
            Error::new(0, "expected proposition")
        );
    }

    #[test]
    fn parse_expression_missing_expression() {
        let mut parser = parser_from_string("a |");

        assert_eq!(
            returned_error(parser.expression()),
            Error::new(3, "expected proposition")
        );
    }

    #[test]
    fn parse_proposition_simple() {
        let mut parser = parser_from_string("a");

        assert_eq!(returned_value(parser.proposition()), "a");
    }

    #[test]
    fn parse_proposition_and() {
        let mut parser = parser_from_string("a -> b");

        assert_eq!(returned_value(parser.proposition()), "¬a ∨ b");
    }

    #[test]
    fn parse_proposition_and_and() {
        let mut parser = parser_from_string("a -> b -> c");

        assert_eq!(returned_value(parser.proposition()), "¬a ∨ ¬b ∨ c");
    }

    #[test]
    fn parse_proposition_invalid() {
        let mut parser = parser_from_string("&");

        assert_eq!(
            returned_error(parser.proposition()),
            Error::new(0, "expected proposition")
        );
    }

    #[test]
    fn parse_proposition_missing_proposition() {
        let mut parser = parser_from_string("a ->");

        assert_eq!(
            returned_error(parser.proposition()),
            Error::new(4, "expected proposition")
        );
    }

    #[test]
    fn parse_top_level_valid() {
        let mut parser = parser_from_string("atom");

        assert_eq!(returned_value(parser.top_level()), "atom");
    }

    #[test]
    fn parse_top_level_invalid() {
        let mut parser = parser_from_string("&");

        assert_eq!(
            returned_error(parser.top_level()),
            Error::new(0, "expected proposition")
        );
    }

    #[test]
    fn parse_top_level_no_end_of_input() {
        let mut parser = parser_from_string("atom atom");

        assert_eq!(
            returned_error(parser.top_level()),
            Error::new(5, "unexpected input")
        );
    }

    #[test]
    fn parse_rich_landed_saintly() {
        let result = Prop::from_str(
            "(landed -> rich) & ~(saintly & rich) -> (landed -> ~saintly)",
        );

        assert_eq!(
            returned_value(result),
            "¬((¬landed ∨ rich) ∧ ¬(saintly ∧ rich)) ∨ ¬landed ∨ ¬saintly"
        );
    }

    fn token_source_from_string<'string>(
        s: &'string str,
    ) -> TokenSource<'string> {
        let cs = CharSource::new(s);
        let scanner = Scanner::try_from(cs)
            .expect("char source should result in valid scanner");
        TokenSource::new(scanner)
    }

    fn parser_from_string<'string>(
        s: &'string str,
    ) -> Parser<'string, TokenSource<'string>> {
        let ts = token_source_from_string(s);
        let lexer = Lexer::try_from(ts)
            .expect("token source should result in valid lexer");
        Parser { lexer }
    }

    fn returned_value<Value: std::fmt::Display>(
        result: Result<Value, Error>,
    ) -> String {
        match result {
            Ok(value) => value.to_string(),
            Err(error) => {
                format!("error at {}: {}", error.offset(), error.message())
            }
        }
    }

    fn returned_error<Value: std::fmt::Display>(
        result: Result<Value, Error>,
    ) -> Error {
        match result {
            Ok(_) => Error::new(0, "expected error"),
            Err(error) => error,
        }
    }
}
