//! Parser combinators.

use std::str::Chars;

/// Parsers read input and return the remaining input and a value.
pub type Parser<Value> =
    for<'a> fn(Input<'a>) -> Result<Output<'a, Value>, Error>;

/// The input to a parser.
#[derive(Clone)]
pub struct Input<'a> {
    posn: usize,        // Offset into input string.
    next: Option<char>, // The next character from the input string.
    rest: Chars<'a>,    // The remaining characters in the input.
}

/// The output of a parser consists of the remaining input and a value.
pub type Output<'a, Value> = (Input<'a>, Value);

/// Parser errors.
pub struct Error {
    pub position: usize, // Offset of the error in the input string.
    pub message: String, // Description of the error.
}

/// Parse the input `test` with `parser`.
pub fn parse<Value>(text: &str, parser: Parser<Value>) -> Result<Value, Error> {
    let mut rest = text.chars();
    let (_, value) = parser(Input {
        posn: 0,
        next: rest.next(),
        rest,
    })?;
    Ok(value)
}

/// Accept the specified value, that is, always succeed with `value`.
pub fn accept<'a, Value>(
    input: Input<'a>,
    value: Value,
) -> Result<Output<'a, Value>, Error> {
    Ok((input, value))
}

/// Reject the input with the provided explanation `message`.
pub fn reject<'a, Value>(
    input: Input<'a>,
    message: &str,
) -> Result<Output<'a, Value>, Error> {
    Err(Error {
        position: input.posn,
        message: message.to_owned(),
    })
}

/// Accept the next character in the input if it passes `test`, otherwise
/// reject the input.
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

/// Expect the end of input.
pub fn end_of_input<'a>(input: Input<'a>) -> Result<Output<'a, ()>, Error> {
    match input.next {
        None => accept(input, ()),
        Some(_) => reject(input, "unexpected input"),
    }
}

/// Parse `input` with `parsers`, one after another. Accept the first
/// result from the first parser that succeeds. Reject the input with
/// `description` if none of them succeed.
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

/// Parse `input` as many times as possible with `parser`.
/// Collect the results in a vector.
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
