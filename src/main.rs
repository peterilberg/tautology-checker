mod prop;
use prop::Prop;
use std::{env, str::FromStr};

fn main() {
    let mut show_usage = false;
    let mut print_newline = false;

    let args: Vec<String> = env::args().collect();
    for arg in args.iter().skip(1) {
        if print_newline {
            println!();
        }
        print_newline = true;

        let prop = Prop::from_str(arg);
        println!("text: {}", arg);
        match prop {
            Ok(prop) => {
                println!("prop: {}", prop);
                println!("taut: {}", prop.is_tautology());
            }
            Err(prop::Error { position, message }) => {
                println!("pos : {:>width$}", "^", width = position + 1);
                println!("err : {}", message);
                show_usage = true;
            }
        }
    }

    if show_usage {
        println!();
        usage(&args[0]);
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
