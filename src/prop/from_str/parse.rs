//! Utilities for writing lexers and parsers.

use std::str::CharIndices;

/// `Error` represents an error in the source text.
/// Error messages must be static strings `&'static str`.
#[derive(Debug, PartialEq)]
pub struct Error {
    /// The offset of the error in the source text.
    offset: usize,
    /// The error message.
    message: &'static str,
}

impl Error {
    /// Create an error at `offset` with a descriptive `message`.
    pub fn new(offset: usize, message: &'static str) -> Self {
        Error { offset, message }
    }

    /// Return the `offset` of the error in the source text.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Return the error `message`.
    pub fn message(&self) -> &str {
        self.message
    }
}

/// `Source` represents an abstract source of `Item`.
pub trait Source {
    /// The item type must be cheap to copy.
    type Item: Copy;
    /// Get the `next` item from the source, if any.
    fn next(&mut self) -> Result<Next<Self::Item>, Error>;
}

/// `Next` represents the next item from a `Source`.
#[derive(Debug, PartialEq)]
pub enum Next<Value> {
    /// A value with its offset.
    Value(usize, Value),
    /// There are no further items in the `Source`.
    End(usize),
}

/// `CharSource` is a source of characters derived from a string `&str`.
pub struct CharSource<'string> {
    /// The source string.
    source: &'string str,
    /// An iterator over all characters in the string and their offsets.
    chars: CharIndices<'string>,
}

impl<'string> CharSource<'string> {
    /// Create a new character source from a `&str`.
    pub fn new(source: &'string str) -> Self {
        CharSource {
            source,
            chars: source.char_indices(),
        }
    }
}

impl<'string> Source for CharSource<'string> {
    /// The items of a CharSource are characters.
    type Item = char;

    /// Get the `next` item from the source.
    fn next(&mut self) -> Result<Next<Self::Item>, Error> {
        let offset = self.chars.offset();

        // Calling `Iterator::next()` after reaching the end of the
        // iterator is not defined. Manually check we're done.
        if offset == self.source.len() {
            return accept(Next::End(offset));
        }
        match self.chars.next() {
            // Get the next character if we're not yet at the end.
            Some((offset, c)) => accept(Next::Value(offset, c)),
            // We've reached the end of the string.
            None => accept(Next::End(offset)),
        }
    }
}

/// A `Stream` of items from a `Source` `Src`.
pub struct Stream<Src>
where
    Src: Source,
    Src::Item: Copy,
{
    source: Src,
    offset: usize,
    next: Option<Src::Item>,
}

impl<Src> Stream<Src>
where
    Src: Source,
    Src::Item: Copy,
{
    /// Create a new stream from a `Source`.
    ///
    /// Note: `Stream<Src>` does not implement the `TryFrom` trait, because
    /// a generic implementation for all types that satisfy the `Source`
    /// trait conflicts with the default implementation in `core`.
    pub fn try_from(source: Src) -> Result<Self, Error> {
        let mut stream = Stream {
            source,
            offset: 0,
            next: None,
        };
        stream.advance_to_next_item()?;
        accept(stream)
    }

    /// Get the current item in the stream.
    pub fn item(&self) -> Option<Src::Item> {
        self.next
    }

    /// Get the offset of the current item in the stream.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Advance the stream to the next item.
    pub fn advance(&mut self) -> Result<(), Error> {
        match self.next {
            // Get next item if we're not yet at the end.
            Some(_) => self.advance_to_next_item(),
            // Error if we're advancing past the end of the stream.
            None => reject(self.offset, "unexpected end of input"),
        }
    }

    // Synchronize the stream to the next item.
    fn advance_to_next_item(&mut self) -> Result<(), Error> {
        match self.source.next() {
            // Update fields with next item and offset.
            Ok(Next::Value(offset, value)) => {
                self.offset = offset;
                self.next = Some(value);
                accept(())
            }
            // Update fields to the end of the stream.
            Ok(Next::End(offset)) => {
                self.offset = offset;
                self.next = None;
                accept(())
            }
            // Forward errors to the calling function.
            Err(error) => Err(error),
        }
    }
}

/// A `Scanner` is a stream of characters from a `CharSource`.
pub type Scanner<'string> = Stream<CharSource<'string>>;

impl<'a> Scanner<'a> {
    /// Get the slice of the source text between `begin` and `end`.
    pub fn slice(&self, begin: usize, end: usize) -> &'a str {
        &self.source.source[begin..end]
    }
}

/// A `Lexer` is a stream of tokens from a token source.
pub type Lexer<TokenSource> = Stream<TokenSource>;

/// Utility function to `accept` a clause in a parsing function with `value`.
pub fn accept<Value>(value: Value) -> Result<Value, Error> {
    Ok(value)
}

/// Utility function to `reject` a clause in a parsing function
/// with explanation `message` at `offset` in the source text.
pub fn reject<Value>(
    offset: usize,
    message: &'static str,
) -> Result<Value, Error> {
    Err(Error::new(offset, message))
}

/// Utility macro to `expect!` a token in the input stream.
/// Parsing fails if a different token is found.
///
/// # Example
///
/// ```
/// expect!(lexer, Token::Name(name), "expected name");
/// ```
#[macro_export]
macro_rules! expect {
    ( $e:expr, $p:pat, $m:literal ) => {
        let Some($p) = $e.item() else {
            return reject($e.offset(), $m);
        };
        $e.advance()?;
    };
}

/// Utility macro to `repeat!` a clause in the grammar.
/// for as long as the next token matches the pattern.
///
/// # Example
///
/// ```
/// let mut a = factor(lexer)?;
/// repeat!(lexer, Token::Add, {
///     let b = factor(lexer)?;
///     a = Expr(Op::Add, a, b);
/// });
/// ```
#[macro_export]
macro_rules! repeat {
    ( $e:expr, $p:pat, $b:block ) => {
        while let Some($p) = $e.item() {
            $e.advance()?;
            $b
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn char_source_empty_string() {
        let mut cs = CharSource::new("");

        assert_eq!(cs.next(), accept(Next::End(0)));
        assert_eq!(cs.next(), accept(Next::End(0)));
    }

    #[test]
    fn char_source_not_empty_string() {
        let mut cs = CharSource::new("abc");

        assert_eq!(cs.next(), accept(Next::Value(0, 'a')));
        assert_eq!(cs.next(), accept(Next::Value(1, 'b')));
        assert_eq!(cs.next(), accept(Next::Value(2, 'c')));
        assert_eq!(cs.next(), accept(Next::End(3)));
        assert_eq!(cs.next(), accept(Next::End(3)));
    }

    #[test]
    fn stream_try_from_empty_string() {
        let cs = CharSource::new("");
        let stream = Stream::try_from(cs)
            .expect("char source should result in valid stream");

        assert_eq!(stream.offset(), 0);
        assert_eq!(stream.item(), None);
    }

    #[test]
    fn stream_try_from_not_empty_string() {
        let cs = CharSource::new("abc");
        let stream = Stream::try_from(cs)
            .expect("char source should result in valid stream");

        assert_eq!(stream.offset(), 0);
        assert_eq!(stream.item(), Some('a'));
    }

    #[test]
    fn stream_advance_empty_string() {
        let cs = CharSource::new("");
        let mut stream = Stream::try_from(cs)
            .expect("char source should result in valid stream");

        assert_eq!(stream.advance(), reject(0, "unexpected end of input"));
    }

    #[test]
    fn stream_advance_not_empty_string() {
        let cs = CharSource::new("abc");
        let mut stream = Stream::try_from(cs)
            .expect("char source should result in valid stream");

        assert_eq!(stream.offset(), 0);
        assert_eq!(stream.item(), Some('a'));
        assert_eq!(stream.advance(), accept(()));

        assert_eq!(stream.offset(), 1);
        assert_eq!(stream.item(), Some('b'));
        assert_eq!(stream.advance(), accept(()));

        assert_eq!(stream.offset(), 2);
        assert_eq!(stream.item(), Some('c'));
        assert_eq!(stream.advance(), accept(()));

        assert_eq!(stream.offset(), 3);
        assert_eq!(stream.item(), None);
        assert_eq!(stream.advance(), reject(3, "unexpected end of input"));
    }

    #[test]
    fn expect_char_in_input() {
        let cs = CharSource::new("abc");
        let mut scanner = Scanner::try_from(cs)
            .expect("char source should result in valid stream");

        assert_eq!(helper_function_for_expect_macro(&mut scanner), accept(()));
    }

    #[test]
    fn repeat_char_in_input() {
        let cs = CharSource::new("abc");
        let mut scanner = Scanner::try_from(cs)
            .expect("char source should result in valid stream");

        assert_eq!(
            helper_function_for_repeat_macro(&mut scanner),
            accept(vec!['a', 'b', 'c'])
        );
    }

    fn helper_function_for_expect_macro(
        scanner: &mut Scanner,
    ) -> Result<(), Error> {
        expect!(scanner, 'a', "expected 'a'");
        expect!(scanner, 'b', "expected 'a'");
        expect!(scanner, 'c', "expected 'a'");
        accept(())
    }

    fn helper_function_for_repeat_macro(
        scanner: &mut Scanner,
    ) -> Result<Vec<char>, Error> {
        let mut result = Vec::new();
        repeat!(scanner, ch, {
            result.push(ch);
        });
        accept(result)
    }
}
