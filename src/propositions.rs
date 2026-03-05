// TODO
// - provide parser to simplify creation

use std::collections::HashSet;
use std::fmt;
use std::rc::Rc;

pub struct Prop(Rc<Proposition>);

enum Proposition {
    Atom(String),
    Negation(Prop),
    Conjunction(Prop, Prop),
    Disjunction(Prop, Prop),
}

impl Prop {
    pub fn atom(name: &str) -> Prop {
        make(Proposition::Atom(name.to_owned()))
    }

    pub fn not(&self) -> Prop {
        make(Proposition::Negation(clone(self)))
    }

    pub fn and(&self, p: &Prop) -> Prop {
        make(Proposition::Conjunction(clone(self), clone(p)))
    }

    pub fn or(&self, p: &Prop) -> Prop {
        make(Proposition::Disjunction(clone(self), clone(p)))
    }

    pub fn implies(&self, p: &Prop) -> Prop {
        self.not().or(p)
    }

    pub fn evaluate(&self, true_atoms: &[&Prop]) -> bool {
        let true_atoms = get_atom_names(true_atoms);
        evaluate(self, &true_atoms)
    }
}

fn make(p: Proposition) -> Prop {
    Prop(Rc::new(p))
}

fn clone(p: &Prop) -> Prop {
    Prop(Rc::clone(&p.0))
}

impl fmt::Display for Prop {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt_prop(self, 0, f)
    }
}

fn fmt_prop(
    prop: &Prop,
    precedence: usize,
    f: &mut fmt::Formatter,
) -> fmt::Result {
    let Prop(prop) = prop;
    match prop.as_ref() {
        Proposition::Atom(name) => write!(f, "{}", name),
        Proposition::Negation(prop) => {
            write!(f, "¬")?;
            fmt_prop(prop, 3, f)
        }
        Proposition::Conjunction(a, b) => {
            fmt_binary(a, "∧", b, 2, precedence, f)
        }
        Proposition::Disjunction(a, b) => {
            if let Some(a) = extract_negated_prop(a) {
                fmt_binary(a, "→", b, 0, precedence, f)
            } else {
                fmt_binary(a, "∨", b, 1, precedence, f)
            }
        }
    }
}

fn fmt_binary(
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
    fmt_prop(a, level, f)?;
    write!(f, " {} ", op)?;
    fmt_prop(b, level, f)?;
    write!(f, "{}", right)
}

fn extract_negated_prop(p: &Prop) -> Option<&Prop> {
    let Prop(prop) = p;
    match prop.as_ref() {
        Proposition::Negation(p) => Some(p),
        _ => None,
    }
}

fn get_atom_names<'a>(props: &'a [&Prop]) -> HashSet<&'a str> {
    props
        .iter()
        .filter_map(|prop| {
            if let Proposition::Atom(name) = prop.0.as_ref() {
                Some(name.as_str())
            } else {
                None
            }
        })
        .collect()
}

fn evaluate(prop: &Prop, true_atoms: &HashSet<&str>) -> bool {
    let Prop(prop) = prop;
    match prop.as_ref() {
        Proposition::Atom(name) => true_atoms.contains(name.as_str()),
        Proposition::Negation(prop) => !evaluate(prop, true_atoms),
        Proposition::Conjunction(a, b) => {
            evaluate(a, true_atoms) && evaluate(b, true_atoms)
        }
        Proposition::Disjunction(a, b) => {
            evaluate(a, true_atoms) || evaluate(b, true_atoms)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rich_landed_saintly() {
        let rich = Prop::atom("rich");
        let landed = Prop::atom("landed");
        let saintly = Prop::atom("saintly");
        let assumption1 = landed.implies(&rich);
        let assumption2 = saintly.and(&rich).not();
        let conclusion = landed.implies(&saintly.not());

        assert_eq!(
            assumption1
                .and(&assumption2)
                .implies(&conclusion)
                .to_string(),
            "(landed → rich) ∧ ¬(saintly ∧ rich) → landed → ¬saintly"
        );
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
    fn the_landed_are_not_saintly() {
        let rich = Prop::atom("rich");
        let landed = Prop::atom("landed");
        let saintly = Prop::atom("saintly");
        let assumption1 = landed.implies(&rich);
        let assumption2 = saintly.and(&rich).not();
        let conclusion = landed.implies(&saintly.not());

        assert!(
            assumption1
                .and(&assumption2)
                .implies(&conclusion)
                .evaluate(&[&landed])
        );
    }
}
