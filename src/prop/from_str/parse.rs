use std::str::Chars;

#[derive(Clone)]
pub struct Input<'a> {
    posn: usize,
    next: Option<char>,
    rest: Chars<'a>,
}

pub type Output<'a, Value> = (Input<'a>, Value);

pub struct Error {
    pub position: usize,
    pub message: String,
}

pub type Parser<Value> =
    for<'a> fn(Input<'a>) -> Result<Output<'a, Value>, Error>;

pub fn parse<Value>(text: &str, parser: Parser<Value>) -> Result<Value, Error> {
    let mut rest = text.chars();
    let (_, value) = parser(Input {
        posn: 0,
        next: rest.next(),
        rest,
    })?;
    Ok(value)
}

pub fn accept<'a, Value>(
    input: Input<'a>,
    value: Value,
) -> Result<Output<'a, Value>, Error> {
    Ok((input, value))
}

pub fn reject<'a, Value>(
    input: Input<'a>,
    message: &str,
) -> Result<Output<'a, Value>, Error> {
    Err(Error {
        position: input.posn,
        message: message.to_owned(),
    })
}

pub fn expect<'a>(
    input: Input<'a>,
    test: fn(char) -> bool,
    description: &str,
) -> Result<Output<'a, char>, Error> {
    match input.next {
        Some(c) if test(c) => {
            let mut input = input;
            input.posn += 1;
            input.next = input.rest.next();
            accept(input, c)
        }
        Some(c) => {
            reject(input, &format!("unexpected {c}, expected {description}"))
        }
        None => reject(
            input,
            &format!("unexpected end of input, expected {description}"),
        ),
    }
}

pub fn end_of_input<'a>(input: Input<'a>) -> Result<Output<'a, ()>, Error> {
    match input.next {
        None => accept(input, ()),
        Some(_) => reject(input, "unexpected input"),
    }
}

pub fn choose<'a, Value>(
    input: Input<'a>,
    parsers: &[Parser<Value>],
    description: &str,
) -> Result<Output<'a, Value>, Error> {
    for parser in parsers {
        match parser(input.clone()) {
            Err(error) if error.position == input.posn => continue,
            value => return value,
        }
    }
    reject(input, &format!("expected {description}"))
}

pub fn repeat<'a, Value>(
    input: Input<'a>,
    parser: Parser<Value>,
) -> Result<Output<'a, Vec<Value>>, Error> {
    let mut values = Vec::new();
    let mut input = input;
    loop {
        match parser(input.clone()) {
            Ok((new_input, value)) => {
                input = new_input;
                values.push(value);
            }
            Err(error) if error.position == input.posn => {
                return Ok((input, values));
            }
            Err(error) => return Err(error),
        }
    }
}
