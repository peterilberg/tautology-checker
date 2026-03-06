// TODO
// - provide parser to simplify creation

use std::collections::HashSet;
use std::fmt;
use std::rc::Rc;

pub struct Prop {
    prop: Rc<Proposition>,
}

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

    pub fn is_tautology(&self) -> bool {
        is_tautology(&cnf(&nnf_positive(self)))
    }
}

fn make(p: Proposition) -> Prop {
    Prop { prop: Rc::new(p) }
}

fn clone(p: &Prop) -> Prop {
    let Prop { prop } = p;
    let prop = Rc::clone(prop);
    Prop { prop }
}

// Utility function because we cannot implement Deref trait on Prop.
// The target type is private and would have to be exposed.
fn deref(p: &Prop) -> &Proposition {
    let Prop { prop } = p;
    prop.as_ref()
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
    match deref(prop) {
        Proposition::Atom(name) => write!(f, "{}", name),
        Proposition::Negation(prop) => {
            write!(f, "¬")?;
            fmt_prop(prop, 2, f)
        }
        Proposition::Conjunction(a, b) => {
            fmt_binary(a, "∧", b, 1, precedence, f)
        }
        Proposition::Disjunction(a, b) => {
            fmt_binary(a, "∨", b, 0, precedence, f)
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

fn get_atom_names<'a>(props: &'a [&Prop]) -> HashSet<&'a str> {
    props
        .iter()
        .filter_map(|prop| {
            if let Proposition::Atom(name) = deref(prop) {
                Some(name.as_str())
            } else {
                None
            }
        })
        .collect()
}

fn evaluate(prop: &Prop, true_atoms: &HashSet<&str>) -> bool {
    match deref(prop) {
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

fn nnf_positive(prop: &Prop) -> Prop {
    match deref(prop) {
        Proposition::Atom(_) => clone(prop),
        Proposition::Negation(prop) => nnf_negative(prop),
        Proposition::Conjunction(a, b) => nnf_positive(a).and(&nnf_positive(b)),
        Proposition::Disjunction(a, b) => nnf_positive(a).or(&nnf_positive(b)),
    }
}

fn nnf_negative(prop: &Prop) -> Prop {
    match deref(prop) {
        Proposition::Atom(_) => prop.not(),
        Proposition::Negation(prop) => nnf_positive(prop),
        Proposition::Conjunction(a, b) => nnf_negative(a).or(&nnf_negative(b)),
        Proposition::Disjunction(a, b) => nnf_negative(a).and(&nnf_negative(b)),
    }
}

fn cnf(prop: &Prop) -> Prop {
    match deref(prop) {
        Proposition::Conjunction(a, b) => cnf(a).and(&cnf(b)),
        Proposition::Disjunction(a, b) => distribute(&cnf(a), &cnf(b)),
        _ => clone(prop),
    }
}

// Distribute disjunction p ∨ q into p and q.
fn distribute(p: &Prop, q: &Prop) -> Prop {
    match (deref(p), deref(q)) {
        (_p, Proposition::Conjunction(q, r)) => {
            distribute(p, q).and(&distribute(p, r))
        }
        (Proposition::Conjunction(p, r), _q) => {
            distribute(p, q).and(&distribute(r, q))
        }
        _ => p.or(q),
    }
}

fn is_tautology(prop: &Prop) -> bool {
    match deref(prop) {
        Proposition::Conjunction(a, b) => a.is_tautology() && b.is_tautology(),
        _ => {
            let mut positive = HashSet::new();
            let mut negative = HashSet::new();
            collect_atoms(prop, &mut positive, &mut negative);
            !positive.is_disjoint(&negative)
        }
    }
}

fn collect_atoms<'a>(
    prop: &'a Prop,
    positive: &mut HashSet<&'a String>,
    negative: &mut HashSet<&'a String>,
) {
    match deref(prop) {
        Proposition::Atom(name) => {
            positive.insert(name);
        }
        Proposition::Negation(prop) => {
            if let Proposition::Atom(name) = deref(prop) {
                negative.insert(name);
            }
        }
        Proposition::Disjunction(a, b) => {
            collect_atoms(a, positive, negative);
            collect_atoms(b, positive, negative);
        }
        _ => panic!("Proposition must be disjunctive clause in CNF."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn nnf_of_not_saintly_and_rich() {
        let rich = Prop::atom("rich");
        let saintly = Prop::atom("saintly");
        let nnf = nnf_positive(&saintly.and(&rich).not());

        assert_eq!(nnf.to_string(), "¬saintly ∨ ¬rich");
    }

    #[test]
    fn distribute_disjunctions() {
        let rich = Prop::atom("rich");
        let landed = Prop::atom("landed");
        let saintly = Prop::atom("saintly");
        let goal = distribute(&rich.and(&saintly), &landed.and(&rich.not()));

        assert_eq!(
            goal.to_string(),
            "(rich ∨ landed) ∧ (saintly ∨ landed) ∧ (rich ∨ ¬rich) ∧ (saintly ∨ ¬rich)"
        );
    }

    #[test]
    fn display_rich_landed_saintly() {
        assert_eq!(
            rich_landed_saintly().to_string(),
            "¬((¬landed ∨ rich) ∧ ¬(saintly ∧ rich)) ∨ ¬landed ∨ ¬saintly"
        );
    }

    #[test]
    fn evaluate_rich_landed_saintly() {
        let landed = Prop::atom("landed");

        assert!(rich_landed_saintly().evaluate(&[&landed]));
    }

    #[test]
    fn cnf_of_rich_landed_saintly() {
        assert_eq!(
            cnf(&nnf_positive(&rich_landed_saintly())).to_string(),
            "(landed ∨ saintly ∨ ¬landed ∨ ¬saintly) ∧ (¬rich ∨ saintly ∨ ¬landed ∨ ¬saintly) ∧ (landed ∨ rich ∨ ¬landed ∨ ¬saintly) ∧ (¬rich ∨ rich ∨ ¬landed ∨ ¬saintly)"
        );
    }

    #[test]
    fn are_the_landed_saintly() {
        assert!(rich_landed_saintly().is_tautology());
    }

    fn rich_landed_saintly() -> Prop {
        let rich = Prop::atom("rich");
        let landed = Prop::atom("landed");
        let saintly = Prop::atom("saintly");
        let assumption1 = landed.implies(&rich);
        let assumption2 = saintly.and(&rich).not();
        let conclusion = landed.implies(&saintly.not());

        assumption1.and(&assumption2).implies(&conclusion)
    }
}
