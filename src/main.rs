mod prop;
use prop::Prop;
use std::{env, str::FromStr};

fn main() {
    let args: Vec<String> = env::args().collect();
    if process_all(&args[1..]).is_err() {
        println!();
        usage(&args[0]);
    }
}

fn process_all(propositions: &[String]) -> Result<(), ()> {
    process_one(&propositions[0])?;
    for proposition in &propositions[1..] {
        println!();
        process_one(proposition)?;
    }
    Ok(())
}

fn process_one(proposition: &String) -> Result<(), ()> {
    println!("text: {}", proposition);
    match Prop::from_str(proposition) {
        Ok(prop) => {
            println!("prop: {}", prop);
            println!("taut: {}", prop.is_tautology());
            Ok(())
        }
        Err(error) => {
            println!("pos : {:>width$}", "^", width = error.offset() + 1);
            println!("err : {}", error.message());
            Err(())
        }
    }
}

fn usage(executable: &str) {
    println!(
        "usage: {} proposition ...
where proposition = name
                  | ~ proposition
                  | proposition | proposition
                  | proposition & proposition
                  | proposition -> proposition
                  | ( proposition )",
        executable
    );
}
