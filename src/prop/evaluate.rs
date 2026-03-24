use super::types::{Prop, Term};

use std::collections::HashSet;
use std::str;

impl Prop {
    /// Evaluate a proposition under the assumption that the atoms in
    /// `true_atoms` are true.
    #[allow(dead_code)]
    pub fn evaluate(&self, true_atoms: &[&Prop]) -> bool {
        let true_atoms = get_atoms(true_atoms);
        eval(self, &true_atoms)
    }
}

fn eval(prop: &Prop, true_atoms: &HashSet<&str>) -> bool {
    match &**prop {
        Term::Atom(name) => true_atoms.contains(name.as_str()),
        Term::Negation(prop) => !eval(prop, true_atoms),
        Term::Conjunction(a, b) => eval(a, true_atoms) && eval(b, true_atoms),
        Term::Disjunction(a, b) => eval(a, true_atoms) || eval(b, true_atoms),
    }
}

// Extract the names from atomic propositions.
fn get_atoms<'a>(props: &'a [&Prop]) -> HashSet<&'a str> {
    props
        .iter()
        .filter_map(|prop| {
            if let Term::Atom(name) = &***prop {
                Some(name.as_str())
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::types::tests::rich_landed_saintly;
    use super::*;

    #[test]
    fn evaluate_false_atom() {
        let a = Prop::atom("a");

        assert!(!a.evaluate(&[]));
    }

    #[test]
    fn evaluate_true_atom() {
        let a = Prop::atom("a");

        assert!(a.evaluate(&[&a]));
    }

    #[test]
    fn evaluate_negation_of_false_atom() {
        let a = Prop::atom("a");

        assert!(a.not().evaluate(&[]));
    }

    #[test]
    fn evaluate_negation_of_true_atom() {
        let a = Prop::atom("a");

        assert!(!a.not().evaluate(&[&a]));
    }

    #[test]
    fn evaluate_conjunction_not_a_not_b() {
        let a = Prop::atom("a");
        let b = Prop::atom("b");

        assert!(!a.and(&b).evaluate(&[]));
    }

    #[test]
    fn evaluate_conjunction_not_a_b() {
        let a = Prop::atom("a");
        let b = Prop::atom("b");

        assert!(!a.and(&b).evaluate(&[&b]));
    }

    #[test]
    fn evaluate_conjunction_a_not_b() {
        let a = Prop::atom("a");
        let b = Prop::atom("b");

        assert!(!a.and(&b).evaluate(&[&a]));
    }

    #[test]
    fn evaluate_conjunction_a_b() {
        let a = Prop::atom("a");
        let b = Prop::atom("b");

        assert!(a.and(&b).evaluate(&[&a, &b]));
    }

    #[test]
    fn evaluate_disjunction_not_a_not_b() {
        let a = Prop::atom("a");
        let b = Prop::atom("b");

        assert!(!a.or(&b).evaluate(&[]));
    }

    #[test]
    fn evaluate_disjunction_not_a_b() {
        let a = Prop::atom("a");
        let b = Prop::atom("b");

        assert!(a.or(&b).evaluate(&[&b]));
    }

    #[test]
    fn evaluate_disjunction_a_not_b() {
        let a = Prop::atom("a");
        let b = Prop::atom("b");

        assert!(a.or(&b).evaluate(&[&a]));
    }

    #[test]
    fn evaluate_disjunction_a_b() {
        let a = Prop::atom("a");
        let b = Prop::atom("b");

        assert!(a.or(&b).evaluate(&[&a, &b]));
    }

    #[test]
    fn evaluate_rich_landed_saintly() {
        let landed = Prop::atom("landed");

        assert!(rich_landed_saintly().evaluate(&[&landed]));
    }
}
