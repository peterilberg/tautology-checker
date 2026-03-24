use super::types::{Prop, Term};

impl Prop {
    /// Calculate the negated normal form of a proposition.
    pub fn nnf(&self) -> Prop {
        positive(self)
    }
}

fn positive(prop: &Prop) -> Prop {
    match &**prop {
        Term::Atom(_) => prop.clone(),
        Term::Negation(prop) => negative(prop),
        Term::Conjunction(a, b) => positive(a).and(&positive(b)),
        Term::Disjunction(a, b) => positive(a).or(&positive(b)),
    }
}

fn negative(prop: &Prop) -> Prop {
    match &**prop {
        Term::Atom(_) => prop.not(),
        Term::Negation(prop) => positive(prop),
        Term::Conjunction(a, b) => negative(a).or(&negative(b)),
        Term::Disjunction(a, b) => negative(a).and(&negative(b)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn of_not_saintly_and_rich() {
        let rich = Prop::atom("rich");
        let saintly = Prop::atom("saintly");
        let nnf = positive(&saintly.and(&rich).not());

        assert_eq!(nnf.to_string(), "¬saintly ∨ ¬rich");
    }
}
