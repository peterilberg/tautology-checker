use super::types::*;
use std::collections::HashSet;

impl Prop {
    pub fn is_tautology(&self) -> bool {
        is_tautology(&self.nnf().cnf())
    }
}

fn is_tautology(prop: &Prop) -> bool {
    match &**prop {
        Term::Conjunction(a, b) => a.is_tautology() && b.is_tautology(),
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
    match &**prop {
        Term::Atom(name) => {
            positive.insert(name);
        }
        Term::Negation(prop) => {
            if let Term::Atom(name) = &**prop {
                negative.insert(name);
            }
        }
        Term::Disjunction(a, b) => {
            collect_atoms(a, positive, negative);
            collect_atoms(b, positive, negative);
        }
        _ => panic!("Term must be disjunctive clause in CNF."),
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::tests::rich_landed_saintly;

    #[test]
    fn are_the_landed_saintly() {
        assert!(rich_landed_saintly().is_tautology());
    }
}
