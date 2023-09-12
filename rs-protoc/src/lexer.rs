use std::str::CharIndices;

use byteyarn::YarnBox;

use crate::source_text::SourceBuffer;

#[derive(Clone)]
pub enum TokenKind<'storage> {
    Identifier(YarnBox<'storage, str>),
    IntegerLiteral(&'storage str),
    FloatLiteral(&'storage str),
    StringLiteral(YarnBox<'storage, str>),
    Semicolon,
    Colon,
    LParen,
    LBracket,
    Comma,
    Equals,
    RParen,
    RBracket,
    Dot,
    Minus,
    LBrace,
    LAngle,
    Slash,
    Plus,
    RBrace,
    RAngle,
    Syntax,
    Float,
    OneOf,
    Import,
    Double,
    Map,
    Weak,
    Int32,
    Extensions,
    Public,
    Int64,
    To,
    Package,
    Uint32,
    Max,
    Option,
    Uint64,
    Reserved,
    Inf,
    Sint32,
    Enum,
    Repeated,
    Sint64,
    Message,
    Optional,
    Fixed32,
    Extend,
    Required,
    Fixed64,
    Service,
    Bool,
    SFixed32,
    Rpc,
    String,
    SFixed64,
    Stream,
    Bytes,
    Group,
    Returns,
    Error(String),
}

pub struct Token<'storage> {
    kind: TokenKind<'storage>,
    line_number: usize,  // 1 based line number
    column_index: usize, // 1 based column index
    character_index: usize,
}

struct Span {
    start: usize,
    end: usize,
}

struct Cursor<'source> {
    source_text: &'source str,
    iter: CharIndices<'source>,
}

impl<'source> Cursor<'source> {
    fn new(source_text: &'source str) -> Self {
        Self {
            source_text: source_text,
            iter: source_text.char_indices(),
        }
    }

    fn next_with_index(&mut self) -> Option<(usize, char)> {
        self.iter.next()
    }

    fn peek(&self) -> Option<char> {
        match self.iter.clone().next() {
            Some((_, ch)) => Some(ch),
            None => None,
        }
    }

    fn peek_next(&self) -> Option<char> {
        let mut iter = self.iter.clone();
        match iter.next() {
            Some(_) => match iter.next() {
                Some((_, ch)) => Some(ch),
                None => None,
            },
            None => None,
        }
    }
}

struct Lexer<'storage> {
    source_text: &'storage str,
    cursor: Cursor<'storage>,
    current_line_column: usize,
    current_line_number: usize,
    current_line_start_char_offset: usize,
    number_of_characters_consumed: usize,
    seen_error: bool,
}

impl<'storage> Lexer<'storage> {
    pub fn new(source_text: &'storage str) -> Self {
        Lexer {
            source_text,
            cursor: Cursor::new(source_text),
            current_line_column: 0,
            current_line_number: 0,
            number_of_characters_consumed: 0,
            current_line_start_char_offset: 0,
            seen_error: false,
        }
    }

    fn identifier_or_keyword(&mut self) -> Option<Token<'storage>> {
        todo!()
    }

    fn numeric_literal(&mut self) -> Option<Token<'storage>> {
        todo!()
    }

    fn string_literal(&mut self, string_literal_header: char) -> Option<Token<'storage>> {
        // We've already consumed the quote
        debug_assert!(string_literal_header == '\'' || string_literal_header == '\"');
        let string_literal_start_index = self.number_of_characters_consumed;
        let mut escaped_sequence = String::new();
        loop {
            if let Some((index, ch)) = self.next_char_with_index() {
                match ch {
                    '\n' => {
                        return Some(Token {
                            kind: self.get_error_token(
                                "Unterminated string literal",
                                Some(Span {
                                    start: string_literal_start_index,
                                    end: index,
                                }),
                            ),
                            line_number: self.current_line_number,
                            column_index: self.current_line_column,
                            character_index: string_literal_start_index,
                        });
                    }
                    '\x00' => {
                        return Some(Token {
                            kind: self.get_error_token(
                                "Unterminated string literal",
                                Some(Span {
                                    start: string_literal_start_index,
                                    end: index,
                                }),
                            ),
                            line_number: self.current_line_number,
                            column_index: self.current_line_column,
                            character_index: string_literal_start_index,
                        });
                    }
                    '\\' => {
                        // Start of escape sequence
                        if escaped_sequence.is_empty() {
                            // Trigger a dynamic allocation and capture all the characters until the start of the escape sequence
                            escaped_sequence
                                .push_str(&self.source_text[string_literal_start_index..index]);
                        }
                        if !self.consume_escape_sequence(&mut escaped_sequence) {
                            return Some(Token {
                                kind: self.get_error_token(
                                    "Invalid escape sequence in string literal",
                                    Some(Span {
                                        start: string_literal_start_index,
                                        end: index,
                                    }),
                                ),
                                line_number: self.current_line_number,
                                column_index: self.current_line_column,
                                character_index: string_literal_start_index,
                            });
                        }
                    }
                    ch if ch == string_literal_header => {
                        // '\'' OR '\"'
                        if escaped_sequence.len() > 0 {
                            return Some(Token {
                                kind: TokenKind::StringLiteral(YarnBox::from_string(
                                    escaped_sequence,
                                )),
                                line_number: self.current_line_number,
                                column_index: self.current_line_column,
                                character_index: string_literal_start_index,
                            });
                        } else {
                            return Some(Token {
                                kind: TokenKind::StringLiteral(YarnBox::new(
                                    &self.source_text[string_literal_start_index..index],
                                )),
                                line_number: self.current_line_number,
                                column_index: self.current_line_column,
                                character_index: string_literal_start_index,
                            });
                        }
                    }
                    ch => {
                        if escaped_sequence.len() > 0 {
                            // We've already triggered an allocation previously when we came across an escape sequence
                            escaped_sequence.push(ch);
                        }
                    }
                }
            } else {
                return Some(Token {
                    kind: self.get_error_token(
                        "Unterminated string literal",
                        Some(Span {
                            start: string_literal_start_index,
                            end: self.source_text.len(),
                        }),
                    ),
                    line_number: self.current_line_number,
                    column_index: self.current_line_column,
                    character_index: string_literal_start_index,
                });
            }
        }
    }

    fn next_token(&mut self) -> Option<Token<'storage>> {
        self.consume_whitespace_and_comments();
        while let Some((index, ch)) = self.next_char_with_index() {
            match ch {
                ';' => {
                    return Some(Token {
                        kind: TokenKind::Semicolon,
                        line_number: self.current_line_number,
                        column_index: self.current_line_column,
                        character_index: index,
                    })
                }
                ':' => {
                    return Some(Token {
                        kind: TokenKind::Colon,
                        line_number: self.current_line_number,
                        column_index: self.current_line_column,
                        character_index: index,
                    })
                }
                '(' => {
                    return Some(Token {
                        kind: TokenKind::LParen,
                        line_number: self.current_line_number,
                        column_index: self.current_line_column,
                        character_index: index,
                    })
                }
                '[' => {
                    return Some(Token {
                        kind: TokenKind::LBracket,
                        line_number: self.current_line_number,
                        column_index: self.current_line_column,
                        character_index: index,
                    })
                }
                ',' => {
                    return Some(Token {
                        kind: TokenKind::Comma,
                        line_number: self.current_line_number,
                        column_index: self.current_line_column,
                        character_index: index,
                    })
                }
                '=' => {
                    return Some(Token {
                        kind: TokenKind::Equals,
                        line_number: self.current_line_number,
                        column_index: self.current_line_column,
                        character_index: index,
                    })
                }
                ')' => {
                    return Some(Token {
                        kind: TokenKind::RParen,
                        line_number: self.current_line_number,
                        column_index: self.current_line_column,
                        character_index: index,
                    })
                }
                ']' => {
                    return Some(Token {
                        kind: TokenKind::RBracket,
                        line_number: self.current_line_number,
                        column_index: self.current_line_column,
                        character_index: index,
                    })
                }
                '.' => {
                    if let Some(ch) = self.cursor.peek() {
                        if ch.is_numeric() {
                            return self.numeric_literal();
                        }
                    }
                    return Some(Token {
                        kind: TokenKind::Dot,
                        line_number: self.current_line_number,
                        column_index: self.current_line_column,
                        character_index: index,
                    });
                }
                '-' => {
                    return Some(Token {
                        kind: TokenKind::Minus,
                        line_number: self.current_line_number,
                        column_index: self.current_line_column,
                        character_index: index,
                    })
                }
                '{' => {
                    return Some(Token {
                        kind: TokenKind::LBrace,
                        line_number: self.current_line_number,
                        column_index: self.current_line_column,
                        character_index: index,
                    })
                }
                '<' => {
                    return Some(Token {
                        kind: TokenKind::LAngle,
                        line_number: self.current_line_number,
                        column_index: self.current_line_column,
                        character_index: index,
                    })
                }
                '/' => {
                    return Some(Token {
                        kind: TokenKind::Slash,
                        line_number: self.current_line_number,
                        column_index: self.current_line_column,
                        character_index: index,
                    });
                }
                '+' => {
                    return Some(Token {
                        kind: TokenKind::Plus,
                        line_number: self.current_line_number,
                        column_index: self.current_line_column,
                        character_index: index,
                    })
                }
                '}' => {
                    return Some(Token {
                        kind: TokenKind::RBrace,
                        line_number: self.current_line_number,
                        column_index: self.current_line_column,
                        character_index: index,
                    })
                }
                '>' => {
                    return Some(Token {
                        kind: TokenKind::RAngle,
                        line_number: self.current_line_number,
                        column_index: self.current_line_column,
                        character_index: index,
                    })
                }
                '\'' | '"' => return self.string_literal(ch),
                '0'..='9' => return self.numeric_literal(),
                'a'..='z' | 'A'..='Z' | '_' => return self.identifier_or_keyword(),
                _ => {
                    return Some(Token {
                        kind: self.get_error_token(
                            "Unknown character",
                            Some(Span {
                                start: index,
                                end: index + 1,
                            }),
                        ),
                        line_number: self.current_line_number,
                        column_index: self.current_line_column,
                        character_index: index,
                    })
                }
            }
        }
        None
    }

    fn get_current_line(&self) -> &'storage str {
        if self.current_line_start_char_offset < self.source_text.len() {
            if let Some(end) = self.source_text[self.current_line_start_char_offset..]
                .chars()
                .position(|ch| ch == '\n')
            {
                return &self.source_text[self.current_line_start_char_offset..end];
            }
        }
        ""
    }

    fn get_source_filename(&self) -> Option<String> {
        None
    }

    fn get_error_token(&mut self, message: &str, _span: Option<Span>) -> TokenKind<'storage> {
        self.seen_error = true;
        let mut error_message = format!("Lexer error {}\n", message);
        error_message += format!(
            "{}:{}",
            self.get_source_filename().unwrap_or("Line".to_string()),
            self.current_line_number
        )
        .as_str();

        TokenKind::Error(error_message)
    }

    /// https://protobuf.com/docs/language-spec#whitespace-and-comments
    fn consume_whitespace_and_comments(&mut self) {
        let is_start_of_single_line_comment = |cursor: &mut Cursor| -> bool {
            if let Some(char_0) = cursor.peek() {
                if char_0 == '/' {
                    if let Some(char_1) = cursor.peek_next() {
                        if char_1 == '/' {
                            return true;
                        }
                    }
                }
            }
            false
        };
        let is_start_of_block_comment = |cursor: &mut Cursor| -> bool {
            if let Some(char_0) = cursor.peek() {
                if char_0 == '/' {
                    if let Some(char_1) = cursor.peek_next() {
                        if char_1 == '*' {
                            return true;
                        }
                    }
                }
            }
            false
        };
        loop {
            if is_start_of_block_comment(&mut self.cursor) {
                self.consume_block_comment();
                continue;
            }
            if is_start_of_single_line_comment(&mut self.cursor) {
                self.consume_single_line_comment();
                continue;
            }
            if let Some(ch) = self.cursor.peek() {
                if is_whitespace(ch) {
                    _ = self.next_char_with_index(); // Consume the whitespace and move ahead
                    continue;
                }
            }
            // At the first non-whitespace/non-comment character
            break;
        }
    }

    fn next_char_with_index(&mut self) -> Option<(usize, char)> {
        match self.cursor.next_with_index() {
            Some((index, ch)) => {
                self.number_of_characters_consumed += 1;
                if ch == '\n' {
                    self.current_line_number += 1;
                    self.current_line_column = 1;
                    self.current_line_start_char_offset = index + 1;
                } else if ch == '\t' {
                    self.current_line_column += 4;
                } else {
                    self.current_line_column += 1;
                }
                return Some((index, ch));
            }
            None => return None,
        }
    }

    /// https://protobuf.com/docs/language-spec#whitespace-and-comments
    fn consume_single_line_comment(&mut self) {
        debug_assert!(self.cursor.peek().is_some());
        debug_assert!(self.cursor.peek().unwrap() == '/');
        _ = self.next_char_with_index(); // Consume the "/"
        debug_assert!(self.cursor.peek().is_some());
        debug_assert!(self.cursor.peek().unwrap() == '/');
        _ = self.next_char_with_index(); // Consume the "/"
        while let Some((_, ch)) = self.next_char_with_index() {
            if ch == '\n' || ch == '\x00' {
                break;
            }
        }
    }
    /// https://protobuf.com/docs/language-spec#whitespace-and-comments
    fn consume_block_comment(&mut self) {
        debug_assert!(self.cursor.peek().is_some());
        debug_assert!(self.cursor.peek().unwrap() == '/');
        _ = self.next_char_with_index(); // Consume the "/"
        debug_assert!(self.cursor.peek().is_some());
        debug_assert!(self.cursor.peek().unwrap() == '*');
        _ = self.next_char_with_index(); // Consume the "*"
        while let Some((_, ch)) = self.next_char_with_index() {
            if ch == '*' {
                if let Some((_, next_ch)) = self.next_char_with_index() {
                    if next_ch == '/' {
                        break;
                    }
                }
            }
        }
    }

    fn consume_hex_escape_sequence(&mut self, escaped_string: &mut String) -> bool {
        let mut hex_digits_as_bytes: [Option<u8>; 2] = [None, None];
        if let Some((_, first_required_char)) = self.next_char_with_index() {
            if first_required_char.is_ascii_hexdigit() {
                hex_digits_as_bytes[0] = Some(first_required_char.to_digit(16).unwrap() as u8);
                if let Some(second_optional_char) = self.cursor.peek() {
                    if second_optional_char.is_ascii_hexdigit() {
                        _ = self.next_char_with_index(); // Consume the second optional char

                        hex_digits_as_bytes[1] =
                            Some(second_optional_char.to_digit(16).unwrap() as u8);
                    }
                }
                if hex_digits_as_bytes[1].is_none() {
                    // SAFETY: We just checked above that the character is a valid ascii hex digit
                    escaped_string.push_str(unsafe {
                        std::str::from_utf8_unchecked(&[hex_digits_as_bytes[0].unwrap()])
                    });
                } else {
                    // SAFETY: We just checked above that the extracted 2 characters are valid ascii hex digit
                    escaped_string.push_str(unsafe {
                        std::str::from_utf8_unchecked(&[
                            hex_digits_as_bytes[0].unwrap(),
                            hex_digits_as_bytes[1].unwrap(),
                        ])
                    });
                }
                return true;
            }
        }
        return false;
    }

    fn consume_octal_escape_sequence(&mut self, escaped_string: &mut String) {
        let is_octal_digit = |ch: char| match ch {
            '0'..='7' => true,
            _ => false,
        };
        if let Some(first_optional_char) = self.cursor.peek() {
            if is_octal_digit(first_optional_char) {
                escaped_string.push(first_optional_char);
                _ = self.next_char_with_index(); // Consume the first optional char
                if let Some(second_optional_char) = self.cursor.peek() {
                    if is_octal_digit(second_optional_char) {
                        _ = self.next_char_with_index(); // Consume the second optional char
                        escaped_string.push(second_optional_char);
                    }
                }
            }
        }
    }

    fn consume_unicode_escape_sequence(
        &mut self,
        escaped_string: &mut String,
        header: char,
    ) -> bool {
        debug_assert!(header == 'u' || header == 'U');
        let mut consume_n_hex_digits = |n: usize| -> bool {
            for _ in 0..n {
                match self.next_char_with_index() {
                    Some((_, ch)) => {
                        if ch.is_ascii_hexdigit() {
                            escaped_string.push(ch);
                        } else {
                            // Found non hex digit
                            return false;
                        }
                    }
                    None => return false, // Ran out of digits
                }
            }
            return true;
        };

        match header {
            'u' => consume_n_hex_digits(4),
            'U' => consume_n_hex_digits(8),
            _ => unreachable!(),
        }
    }

    fn consume_escape_sequence(&mut self, escaped_string: &mut String) -> bool {
        match self.next_char_with_index() {
            Some((_, ch)) => match ch {
                'a' => {
                    escaped_string.push('\x07'); // Alert bell
                    return true;
                }
                'b' => {
                    escaped_string.push('\x08'); // Back space
                    return true;
                }
                'f' => {
                    escaped_string.push('\x0c'); // Form feed
                    return true;
                }
                'n' => {
                    escaped_string.push('\n'); // New line
                    return true;
                }
                'r' => {
                    escaped_string.push('\x0d'); // Carriage return
                    return true;
                }
                't' => {
                    escaped_string.push('\t'); // Horizontal tab
                    return true;
                }
                'v' => {
                    escaped_string.push('\x0b'); // Vertical tab
                    return true;
                }
                '\"' => {
                    escaped_string.push('\"');
                    return true;
                }
                '\'' => {
                    escaped_string.push('\'');
                    return true;
                }
                '?' => {
                    escaped_string.push('?');
                    return true;
                }
                'x' | 'X' => self.consume_hex_escape_sequence(escaped_string),
                '0'..='7' => {
                    self.consume_octal_escape_sequence(escaped_string);
                    return true;
                }
                'u' | 'U' => self.consume_unicode_escape_sequence(escaped_string, ch),
                _ => false,
            },
            None => false,
        }
    }
}

impl<'storage> Iterator for Lexer<'storage> {
    type Item = Token<'storage>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

fn is_whitespace(ch: char) -> bool {
    // https://protobuf.com/docs/language-spec#whitespace-and-comments
    match ch {
        ' ' | '\n' | '\r' | '\t' => true,
        '\x0c' => true, // Form-feed
        '\x0b' => true, // Vertical-tab
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consume_whitespace_and_comments() {
        {
            let mut lexer = Lexer::new("");
            assert!(lexer.next().is_none());
        }
        {
            let mut lexer = Lexer::new("             //");
            assert!(lexer.next().is_none());
        }
        {
            let mut lexer = Lexer::new("/* Comment */ // // // // // // // // ");
            assert!(lexer.next().is_none());
        }
        {
            let source_text = r#"
                // Single Line Comment
                /* Multi-line comment line 1
                 * Multi-line comment line 2
                 * Multi-line comment line 3
                 */
            "#;
            let mut lexer = Lexer::new(source_text);
            assert!(lexer.next().is_none());
        }
    }

    #[test]
    fn test_string_literal_1() {
        let mut lexer = Lexer::new("\"StringLiteral\"");
        let result = lexer.next();
        assert!(result.is_some());
        let token = result.unwrap();
        match token.kind {
            TokenKind::StringLiteral(string) => {
                assert!(string == "StringLiteral");
            }
            _ => assert!(false),
        }
    }
    #[test]
    fn test_string_literal_2() {
        let mut lexer = Lexer::new("\"String\\nLiteral\"");
        let result = lexer.next();
        assert!(result.is_some());
        let token = result.unwrap();
        match token.kind {
            TokenKind::StringLiteral(string) => {
                assert!(string == "String\nLiteral");
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_string_literal_3() {
        let mut lexer = Lexer::new("'First\\x09Second'");
        let result = lexer.next();
        assert!(result.is_some());
        let token = result.unwrap();
        match token.kind {
            TokenKind::StringLiteral(string) => {
                let strref = "First\tSecond";
                println!("{}", strref.len());
                println!("{}", string.len());
                assert!(string == "First\tSecond");
            }
            _ => assert!(false),
        }
    }
}
