use std::ops::Deref;
use std::rc::Rc;

pub struct Prop {
    prop: Rc<Term>,
}

pub enum Term {
    Atom(String),
    Negation(Prop),
    Conjunction(Prop, Prop),
    Disjunction(Prop, Prop),
}

impl Prop {
    pub fn atom(name: &str) -> Prop {
        Prop::new(Term::Atom(name.to_owned()))
    }

    pub fn not(&self) -> Prop {
        Prop::new(Term::Negation(self.clone()))
    }

    pub fn and(&self, p: &Prop) -> Prop {
        Prop::new(Term::Conjunction(self.clone(), p.clone()))
    }

    pub fn or(&self, p: &Prop) -> Prop {
        Prop::new(Term::Disjunction(self.clone(), p.clone()))
    }

    pub fn implies(&self, p: &Prop) -> Prop {
        self.not().or(p)
    }

    fn new(p: Term) -> Prop {
        Prop { prop: Rc::new(p) }
    }
}

impl Deref for Prop {
    type Target = Term;

    fn deref(&self) -> &Self::Target {
        let Prop { prop } = self;
        prop.as_ref()
    }
}

impl Clone for Prop {
    fn clone(&self) -> Self {
        let Prop { prop } = self;
        let prop = Rc::clone(prop);
        Prop { prop }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn rich_landed_saintly() -> Prop {
        let rich = Prop::atom("rich");
        let landed = Prop::atom("landed");
        let saintly = Prop::atom("saintly");
        let assumption1 = landed.implies(&rich);
        let assumption2 = saintly.and(&rich).not();
        let conclusion = landed.implies(&saintly.not());

        assumption1.and(&assumption2).implies(&conclusion)
    }
}
