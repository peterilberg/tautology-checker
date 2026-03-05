mod propositions;
use propositions::Prop;

fn main() {
    let a = Prop::atom("A");
    let b = Prop::atom("B");
    let goal = a.implies(&b);
    println!(
        "When A is false and B is true, {} is {}.",
        goal,
        goal.evaluate(&[&b])
    )
}
