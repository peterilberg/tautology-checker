use super::types::*;

impl Prop {
    /// Calculate the conjunctive normal form of a proposition.
    /// The proposition must be in negated normal form.
    pub fn cnf(&self) -> Prop {
        match &**self {
            Term::Conjunction(a, b) => a.cnf().and(&b.cnf()),
            Term::Disjunction(a, b) => distribute(&a.cnf(), &b.cnf()),
            _ => self.clone(),
        }
    }
}

// Distribute disjunction
// p ∨ (q ∧ r) => (p ∨ q) ∧ (p ∨ r)
// (p ∧ r) ∨ q => (p ∨ q) ∧ (r ∨ q)
fn distribute(p: &Prop, q: &Prop) -> Prop {
    match (&**p, &**q) {
        (_p, Term::Conjunction(q, r)) => {
            distribute(p, q).and(&distribute(p, r))
        }
        (Term::Conjunction(p, r), _q) => {
            distribute(p, q).and(&distribute(r, q))
        }
        _ => p.or(q),
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::tests::rich_landed_saintly;
    use super::*;

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
    fn cnf_of_rich_landed_saintly() {
        assert_eq!(
            rich_landed_saintly().nnf().cnf().to_string(),
            "(landed ∨ saintly ∨ ¬landed ∨ ¬saintly) ∧ (¬rich ∨ saintly ∨ ¬landed ∨ ¬saintly) ∧ (landed ∨ rich ∨ ¬landed ∨ ¬saintly) ∧ (¬rich ∨ rich ∨ ¬landed ∨ ¬saintly)"
        );
    }
}
