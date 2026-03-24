//! Utilities for pretty-printing values.

use unicode_segmentation::UnicodeSegmentation;

/// The text that we want to pretty-print consists of a set of `Item`s.
enum Item<'item> {
    /// A simple string that will be printed verbatim.
    String { value: &'item str },
    /// A potential line-break if the current line gets too long.
    /// Either print the `value` if it fits, or continue on a new line.
    Break { value: &'item str },
    /// A block of text `Item`s that will be indented.
    Block {
        indentation: usize,
        items: Vec<Text<'item>>,
    },
}

/// A `Text` element consists of the `Item` that describes what kind of
/// text element it represents. We precompute the total length of the
/// text item in characters. We also precompute the distance to the
/// next potential line-break, if any.
pub struct Text<'text> {
    /// The text `Item`.
    item: Item<'text>,
    /// Its length in characters.
    length: usize,
    /// Can we break the current line in the current block?
    followed_by_break: bool,
    /// Where can we break the line in the current block?
    distance_to_break: usize,
}

impl<'text> Text<'text> {
    /// Create a new literal `string` that will be printed verbatim.
    pub fn string(value: &'text str) -> Text<'text> {
        Text {
            item: Item::String { value },
            length: number_of_graphemes(value),
            followed_by_break: false,
            distance_to_break: 0,
        }
    }

    /// Create a potential line-break. Either we print the
    /// `value` if it fits on the current line or we continue
    /// on a new line if it doesn't fit.
    pub fn string_or_break(value: &'text str) -> Text<'text> {
        Text {
            item: Item::Break { value },
            length: number_of_graphemes(value),
            followed_by_break: true,
            distance_to_break: 0,
        }
    }

    /// Create an indented block of text that consists of more
    /// text elements.
    pub fn block<const N: usize>(
        indentation: usize,
        items: [Text<'text>; N],
    ) -> Text<'text> {
        let mut items = Vec::from(items);
        // Calculate the distance to the next potential line-break
        // in the current block. Start at the end of the block and
        // work backwards.
        let _ = items.iter_mut().rfold(
            (false, 0),
            |(found_break, distance), item| {
                item.followed_by_break = found_break;
                item.distance_to_break = distance;
                match item.item {
                    Item::Break { .. } => (true, 0),
                    _ => (found_break, distance + item.length),
                }
            },
        );

        Text {
            length: items.iter().map(|item| item.length).sum(),
            item: Item::Block { indentation, items },
            followed_by_break: false,
            distance_to_break: 0,
        }
    }
}

/// Pretty-print the provided `text`. Try to make it fit into `line_width`.
pub fn print(text: Text, line_width: usize) -> Vec<String> {
    let output = Output::new(line_width);
    let output = pretty(output, &text, line_width, 0);
    output.finish()
}

/// Pretty-print a text element in the `available_space`.
fn pretty(
    mut output: Output,
    text: &Text,
    available_space: usize,
    distance_to_break: usize,
) -> Output {
    match &text.item {
        // Print a string by appending it verbatim to the output.
        Item::String { value } => output.append_string(value, text.length),
        // Print a text block by printing its items.
        Item::Block {
            indentation, items, ..
        } => {
            let space_for_block = output.space_for_block(*indentation);
            for item in items {
                output = pretty(
                    output,
                    item,
                    space_for_block,
                    next_potential_line_break(item, distance_to_break),
                );
            }
        }
        // Print a potential line-break either by printing its value
        // verbatim on the current line, or by continuing on a new line.
        Item::Break { value } => {
            if text.length + distance_to_break <= output.space_left {
                output.append_string(value, text.length)
            } else {
                let len = output.line_width - available_space;
                output.line_break(len);
            }
        }
    }
    output
}

/// Utility function to compute the number of characters (graphemes)
/// in a string.
fn number_of_graphemes(string: &str) -> usize {
    string.graphemes(true).count()
}

/// Utility function to find the next potential line-break in the `text`.
/// We can break at the earliest at the next line-break in the current
/// block, if any, or at the next line-break `after` the current block.
fn next_potential_line_break(text: &Text, after: usize) -> usize {
    text.distance_to_break + if text.followed_by_break { 0 } else { after }
}

/// Utility struct to collect the output of the pretty-printing process.
struct Output {
    /// The current line.
    current_line: String,
    /// How much space is left on the current line in characters.
    space_left: usize,
    /// Previously printed lines.
    lines: Vec<String>,
    /// The maximum width or length of a line in characters.
    line_width: usize,
}

impl Output {
    /// Create a new output struct.
    fn new(line_width: usize) -> Self {
        Output {
            space_left: line_width,
            current_line: String::new(),
            lines: Vec::new(),
            line_width,
        }
    }

    /// Finish and return what we have printed.
    fn finish(mut self) -> Vec<String> {
        self.lines.push(self.current_line);
        self.lines
    }

    /// Append a string verbatim to the current line.
    fn append_string(&mut self, s: &str, len: usize) {
        self.current_line.push_str(s);
        self.space_left = self.space_left.saturating_sub(len);
    }

    /// End the current line and start a new one.
    fn line_break(&mut self, indent: usize) {
        self.lines.push(self.current_line.clone());
        self.space_left = self.line_width;
        self.current_line = String::new();
        self.append_string(&" ".repeat(indent), indent)
    }

    /// Calculate how much space is available for a text block
    /// with `indentation` on the current line. That is, how
    /// wide can the lines of the text block be.
    fn space_for_block(&mut self, indentation: usize) -> usize {
        self.space_left - indentation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_string() {
        let text = Text::string("1234567890");
        let lines = print(text, 30);

        assert_eq!(lines, vec!["1234567890"]);
    }

    #[test]
    fn print_string_or_break() {
        let text = Text::string_or_break("1234567890");
        let lines = print(text, 30);

        assert_eq!(lines, vec!["1234567890"]);
    }

    #[test]
    fn print_block_that_fits() {
        let text = Text::block(0, [Text::string("1234567890")]);
        let lines = print(text, 30);

        assert_eq!(lines, vec!["1234567890"]);
    }

    #[test]
    fn print_block_thats_too_wide() {
        let text = Text::block(0, [Text::string("1234567890")]);
        let lines = print(text, 5);

        assert_eq!(lines, vec!["1234567890"]);
    }

    #[test]
    fn print_block_with_break_that_fits() {
        let text = Text::block(
            0,
            [
                Text::string("1234567890"),
                Text::string_or_break(" "),
                Text::string("1234567890"),
            ],
        );
        let lines = print(text, 30);

        assert_eq!(lines, vec!["1234567890 1234567890"]);
    }

    #[test]
    fn print_block_with_break_thats_too_wide() {
        let text = Text::block(
            0,
            [
                Text::string("1234567890"),
                Text::string_or_break(" "),
                Text::string("1234567890"),
            ],
        );
        let lines = print(text, 15);

        assert_eq!(lines, vec!["1234567890", "1234567890"]);
    }

    #[test]
    fn print_block_with_break_and_indentation_that_fits() {
        let text = Text::block(
            4,
            [
                Text::string("1234567890"),
                Text::string_or_break(" "),
                Text::string("1234567890"),
            ],
        );
        let lines = print(text, 30);

        assert_eq!(lines, vec!["1234567890 1234567890"]);
    }

    #[test]
    fn print_block_with_break_and_indentation_thats_too_wide() {
        let text = Text::block(
            4,
            [
                Text::string("1234567890"),
                Text::string_or_break(" "),
                Text::string("1234567890"),
            ],
        );
        let lines = print(text, 15);

        assert_eq!(lines, vec!["1234567890", "    1234567890"]);
    }
}
