use super::types::{Prop, Term};
use std::fmt;
use std::str;

mod pretty;

use pretty::Text;

impl fmt::Display for Prop {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = term(self, 0);
        let width = f.width().unwrap_or(60);
        let lines = pretty::print(text, width);
        write!(f, "{}", lines.join("\n"))
    }
}

fn term<'prop>(prop: &'prop Prop, precedence: usize) -> Text<'prop> {
    match &**prop {
        Term::Atom(name) => Text::string(name),
        Term::Negation(prop) => {
            // Wrap negated propositions in a block with indentation 1
            // to account for the "¬" character.
            Text::block(1, [Text::string("¬"), term(prop, 2)])
        }
        Term::Conjunction(a, b) => binary(a, "∧", b, 1, precedence),
        Term::Disjunction(a, b) => binary(a, "∨", b, 0, precedence),
    }
}

fn binary<'prop>(
    a: &'prop Prop,
    op: &'prop str,
    b: &'prop Prop,
    level: usize,
    precedence: usize,
) -> Text<'prop> {
    let (left, right) = if precedence <= level {
        ("", "")
    } else {
        ("(", ")")
    };

    // Wrap the binary operator in a block.
    Text::block(
        // Indent by the width of the optional left parenthesis,
        // so that the right argument aligns with the left one
        // if there is a link-break.
        left.len(),
        [
            Text::string(left),
            term(a, level),
            Text::string(" "),
            Text::string(op),
            // Permit line-breaks only between the binary operator
            // and the right argument.
            Text::string_or_break(" "),
            term(b, level),
            Text::string(right),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::super::types::tests::rich_landed_saintly;
    use super::*;

    #[test]
    fn display_atom() {
        let a = Prop::atom("a");

        assert_eq!(a.to_string(), "a");
    }

    #[test]
    fn display_negation() {
        let a = Prop::atom("a");

        assert_eq!(a.not().to_string(), "¬a");
    }

    #[test]
    fn display_disjunction() {
        let a = Prop::atom("a");
        let b = Prop::atom("b");

        assert_eq!(a.or(&b).to_string(), "a ∨ b");
    }

    #[test]
    fn display_conjunction() {
        let a = Prop::atom("a");
        let b = Prop::atom("b");

        assert_eq!(a.and(&b).to_string(), "a ∧ b");
    }

    #[test]
    fn display_implication() {
        let a = Prop::atom("a");
        let b = Prop::atom("b");

        assert_eq!(a.implies(&b).to_string(), "¬a ∨ b");
    }

    #[test]
    fn no_parentheses_on_different_operator_precedences() {
        let a = Prop::atom("a");
        let b = Prop::atom("b");
        let c = Prop::atom("c");

        assert_eq!(a.not().and(&b).or(&c).to_string(), "¬a ∧ b ∨ c");
    }

    #[test]
    fn no_parentheses_on_same_operator_precedence() {
        let a = Prop::atom("a");
        let b = Prop::atom("b");
        let c = Prop::atom("c");
        let d = Prop::atom("d");

        assert_eq!(a.and(&b).and(&c.and(&d)).to_string(), "a ∧ b ∧ c ∧ d");
    }

    #[test]
    fn display_rich_landed_saintly() {
        assert_eq!(
            rich_landed_saintly().to_string(),
            "¬((¬landed ∨ rich) ∧ ¬(saintly ∧ rich)) ∨ ¬landed ∨ ¬saintly"
        );
    }

    #[test]
    fn display_rich_landed_saintly_narrowly() {
        let prop = rich_landed_saintly();
        let text = term(&prop, 0);
        let lines = pretty::print(text, 30);

        assert_eq!(
            lines,
            vec![
                "¬((¬landed ∨ rich) ∧",
                "  ¬(saintly ∧ rich)) ∨",
                "¬landed ∨ ¬saintly"
            ]
        );
    }
}
