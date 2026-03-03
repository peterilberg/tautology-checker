// TODO
// - working with propositions is clumsy
//   - cannot nest creation because of mut ref to self
//     - keep it but also provide parser to simplify creation
//     - interior mutability with RefCell<Vec<Proposition>>
//   - printing needs a helper tuple
//     - hide precedence inside implementation (helper function)
//   - index should be hidden and not exposed as usize
//   - index is not self-sufficient, needs propositions struct
//     - it's supposed to abstract references / pointers into vector
//     - pair of int and ref to struct?
//   - print implications →
//   - move propositions into library

struct Propositions {
    propositions: Vec<Proposition>,
}

type Index = usize;

enum Proposition {
    Atom { name: String },
    Negation { prop: Index },
    Conjunction { a: Index, b: Index },
    Disjunction { a: Index, b: Index },
}

impl Propositions {
    fn new() -> Propositions {
        Propositions {
            propositions: Vec::new(),
        }
    }

    fn atom(&mut self, name: &str) -> Index {
        self.add(Proposition::Atom {
            name: name.to_string(),
        })
    }

    fn negation(&mut self, p: Index) -> Index {
        self.add(Proposition::Negation { prop: p })
    }

    fn conjunction(&mut self, p: Index, q: Index) -> Index {
        self.add(Proposition::Conjunction { a: p, b: q })
    }

    fn disjunction(&mut self, p: Index, q: Index) -> Index {
        self.add(Proposition::Disjunction { a: p, b: q })
    }

    fn implies(&mut self, p: Index, q: Index) -> Index {
        let negation = Propositions::negation(self, p);
        Propositions::disjunction(self, negation, q)
    }

    fn get(&self, p: Index) -> &Proposition {
        &self.propositions[p]
    }

    fn add(&mut self, p: Proposition) -> Index {
        self.propositions.push(p);
        self.propositions.len() - 1
    }
}

fn main() {
    let mut props = Propositions::new();
    let a = props.atom("A");
    let b = props.atom("A");
    let a_b = props.implies(a, b);
    println!("Hello, world!");
}

use std::fmt;

struct Pair<'a>(&'a Propositions, &'a Index, usize);

impl<'a> fmt::Display for Pair<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let props = self.0;
        let i = self.1;
        let prec = self.2;
        match props.get(*i) {
            Proposition::Atom { name } => write!(f, "{}", name),
            Proposition::Negation { prop } => {
                write!(f, "¬{}", Pair(props, prop, 2))
            }
            Proposition::Conjunction { a, b } => {
                if prec <= 1 {
                    write!(f, "{} ∧ {}", Pair(props, a, 1), Pair(props, b, 1))
                } else {
                    write!(f, "({} ∧ {})", Pair(props, a, 1), Pair(props, b, 1))
                }
            }
            Proposition::Disjunction { a, b } => {
                if prec == 0 {
                    write!(f, "{} ∨ {}", Pair(props, a, 0), Pair(props, b, 0))
                } else {
                    write!(f, "({} ∨ {})", Pair(props, a, 0), Pair(props, b, 0))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut props = Propositions::new();
        let rich = props.atom("rich");
        let landed = props.atom("landed");
        let saintly = props.atom("saintly");
        let assumption1 = props.implies(landed, rich);
        let assumption2_a = props.conjunction(saintly, rich);
        let assumption2 = props.negation(assumption2_a);
        let conclusion_a = props.negation(saintly);
        let conclusion = props.implies(landed, conclusion_a);
        let goal_a = props.conjunction(assumption1, assumption2);
        let goal = props.implies(goal_a, conclusion);

        let goal_fmt = format!("{}", Pair(&props, &goal, 0));
        assert_eq!(
            goal_fmt,
            "¬((¬landed ∨ rich) ∧ ¬(saintly ∧ rich)) ∨ ¬landed ∨ ¬saintly"
        );
    }

    #[test]
    fn test_a() {
        let mut props = Propositions::new();
        let a = props.atom("a");
        let b = props.atom("b");
        let c = props.atom("c");
        let neg_a = props.negation(a);
        let conj = props.conjunction(neg_a, b);
        let disj = props.disjunction(conj, c);

        let disj_fmt = format!("{}", Pair(&props, &disj, 0));
        assert_eq!(disj_fmt, "¬a ∧ b ∨ c");
    }

    #[test]
    fn test_b() {
        let mut props = Propositions::new();
        let a = props.atom("a");
        let b = props.atom("b");
        let c = props.atom("c");
        let d = props.atom("d");
        let conj_ab = props.conjunction(a, b);
        let conj_cd = props.conjunction(c, d);
        let conj = props.conjunction(conj_ab, conj_cd);

        let conj_fmt = format!("{}", Pair(&props, &conj, 0));
        assert_eq!(conj_fmt, "a ∧ b ∧ c ∧ d");
    }
}
