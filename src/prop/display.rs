use super::types::*;

use std::fmt;
use std::str;

impl fmt::Display for Prop {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        term(self, 0, f)
    }
}

fn term(prop: &Prop, precedence: usize, f: &mut fmt::Formatter) -> fmt::Result {
    match &**prop {
        Term::Atom(name) => write!(f, "{}", name),
        Term::Negation(prop) => {
            write!(f, "¬")?;
            term(prop, 2, f)
        }
        Term::Conjunction(a, b) => binary(a, "∧", b, 1, precedence, f),
        Term::Disjunction(a, b) => binary(a, "∨", b, 0, precedence, f),
    }
}

fn binary(
    a: &Prop,
    op: &str,
    b: &Prop,
    level: usize,
    precedence: usize,
    f: &mut fmt::Formatter,
) -> fmt::Result {
    let (left, right) = if precedence <= level {
        ("", "")
    } else {
        ("(", ")")
    };

    write!(f, "{}", left)?;
    term(a, level, f)?;
    write!(f, " {} ", op)?;
    term(b, level, f)?;
    write!(f, "{}", right)
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
}
