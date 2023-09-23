use crate::error::{Result, RsProtocError};

use std::str::Chars;
use std::str::FromStr;

use byteyarn::YarnBox;

#[derive(Clone, PartialEq)]
pub enum TokenKind<'storage> {
    Identifier(YarnBox<'storage, str>),
    IntegerLiteral(u64),
    FloatLiteral(f64),
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
    Error(String), /*TODO Add more information here for better diagnostics */
}

#[derive(Clone)]
pub struct TokenMetadata {
    span: Span,
    line_info: LineInfo,
}

#[derive(Clone)]
pub struct Token<'storage> {
    pub kind: TokenKind<'storage>,
    pub metadata: TokenMetadata,
}

#[derive(Clone)]
pub struct Lexer<'storage> {
    source_text: &'storage str,
    cursor: Cursor<'storage>,
    current_line_column: usize,
    current_line_number: usize,
    current_line_start_char_offset: usize,
    seen_error: bool,
}

impl<'storage> Lexer<'storage> {
    /// Create a lexer to generate tokens for the provided source text
    pub fn new(source_text: &'storage str) -> Self {
        Lexer {
            source_text,
            cursor: Cursor::new(source_text),
            current_line_column: 0,
            current_line_number: 0,
            current_line_start_char_offset: 0,
            seen_error: false,
        }
    }

    /// Print the token in the context of the line it's part of in the source text
    pub fn print_token_in_line(&self, metadata: &TokenMetadata) {
        println!(
            "Line {}:{}",
            metadata.line_info.line_number, metadata.line_info.column_number
        );
        println!("{}", self.get_token_line(metadata));
        for _ in 0..metadata.line_info.column_number - 2 {
            print!(" ");
        }
        print!("↑");
        if metadata.span.len() > 1 {
            for _ in metadata.line_info.column_number
                ..(metadata.line_info.column_number - 2 + metadata.span.len())
            {
                print!(" ");
            }
            println!("↑");
        } else {
            println!("");
        }
    }

    fn get_token_line(&self, metadata: &TokenMetadata) -> String {
        let line_start = metadata.line_info.line_start_offset_into_source;
        let offset = self.source_text[line_start + 1..].find('\n');
        if let Some(offset) = offset {
            format!("{}", &self.source_text[line_start..line_start + offset + 1])
        } else {
            format!("{}", &self.source_text[line_start..])
        }
    }

    fn get_token_metadata(&self, span: Span) -> TokenMetadata {
        TokenMetadata {
            span: span.clone(),
            line_info: LineInfo {
                line_start_offset_into_source: self.current_line_start_char_offset,
                line_number: self.current_line_number,
                column_number: self.current_line_column - span.len() + 1,
            },
        }
    }

    fn identifier_or_keyword(&mut self, header: char) -> Option<Token<'storage>> {
        debug_assert!(header.is_alphabetic() || header == '_');
        let start = self.cursor.get_current_index() - 1;
        loop {
            if let Some(ch) = self.cursor.peek() {
                if ch.is_alphanumeric() || ch == '_' {
                    _ = self.next_char();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        let end = self.cursor.get_current_index();
        let idententifier_or_keyword = &self.source_text[start..end];
        if let Some(keyword) = get_keyword_token_kind(idententifier_or_keyword) {
            return Some(Token {
                kind: keyword,
                metadata: self.get_token_metadata(Span { start, end }),
            });
        }
        return Some(Token {
            kind: TokenKind::Identifier(YarnBox::from(idententifier_or_keyword)),
            metadata: self.get_token_metadata(Span { start, end }),
        });
    }

    fn consume_decimal_digits(&mut self) {
        loop {
            if let Some(ch) = self.cursor.peek() {
                if ch.is_ascii_digit() {
                    _ = self.next_char();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }
    fn consume_hex_digits(&mut self) {
        loop {
            if let Some(ch) = self.cursor.peek() {
                if ch.is_ascii_hexdigit() {
                    _ = self.next_char();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn consume_octal_digits(&mut self) {
        loop {
            if let Some(ch) = self.cursor.peek() {
                match ch {
                    '0'..='7' => {
                        _ = self.next_char();
                    }
                    _ => {
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }

    fn determine_radix(&mut self, header: char) -> Radix {
        debug_assert!(header.is_numeric() || header == '.');
        let mut radix = Radix::Decimal; // Default to a decimal radix for the integral part
        if header == '0' {
            radix = Radix::Octal;
            if let Some(ch) = self.cursor.peek() {
                if ch == 'X' || ch == 'x' {
                    radix = Radix::Hexadecimal;
                    _ = self.next_char();
                }
            }
        }
        return radix;
    }

    fn extract_integral_part(&mut self, header: char, radix: Radix) -> Result<Span> {
        debug_assert!(header.is_numeric() || header == '.');
        if header == '.' {
            // Example case: ".123" Integral part = ""
            Ok(Span {
                start: self.cursor.get_current_index(),
                end: self.cursor.get_current_index(),
            })
        } else {
            // Example case: "1.123" Integral part = "1"
            let mut start = self.cursor.get_current_index() - 1;
            match radix {
                Radix::Decimal => self.consume_decimal_digits(),
                Radix::Hexadecimal => {
                    start += 1; // Move past the 'x'/'X'
                    let cached_index = self.cursor.get_current_index();
                    self.consume_hex_digits();
                    if cached_index == self.cursor.get_current_index() {
                        return Err(RsProtocError::LexError(
                            "Expected hexadecimal digits after the \"0x\"/\"0X\"".to_string(),
                        ));
                    }
                }
                Radix::Octal => self.consume_octal_digits(),
            }
            Ok(Span {
                start,
                end: self.cursor.get_current_index(),
            })
        }
    }

    fn extract_fractional_part(&mut self, integral_part: &Span, header: char) -> Span {
        // ".<FRACTIONAL_PART>"
        debug_assert!(header.is_numeric() || header == '.');
        if integral_part.is_empty() {
            // This means that the header == '.'
            assert!(header == '.');
            // Example case: ".123" Fractional part = .123
            let start = self.cursor.get_current_index() - 1;
            self.consume_decimal_digits();
            let end = self.cursor.get_current_index();
            // We should have had decimal digits after the '.'
            // Assert this
            assert!(end > start);
            Span { start, end }
        } else {
            // Default: Assume we don't have any fractional part
            // Example case: "123" Fractional part = ""
            let mut start = self.cursor.get_current_index();
            let mut end = self.cursor.get_current_index();
            if let Some(ch) = self.cursor.peek() {
                if ch == '.' {
                    // Example case: "123.666" Fractional part = .666
                    start = self.cursor.get_current_index();
                    _ = self.next_char();
                    self.consume_decimal_digits();
                    end = self.cursor.get_current_index();
                }
            }
            Span { start, end }
        }
    }

    fn extract_exponent(&mut self) -> Result<Span> {
        // exponent  = ( "e" | "E" ) [ "+" | "-" ] <EXPONENT_PART>
        // Default to empty span
        let mut span = Span {
            start: self.cursor.get_current_index(),
            end: self.cursor.get_current_index(),
        };
        if let Some(ch) = self.cursor.peek() {
            match ch {
                'e' | 'E' => {
                    _ = self.next_char();
                    span.start += 1;
                    if let Some(ch) = self.cursor.peek() {
                        match ch {
                            '+' | '-' => {
                                // Consume optional '+'/'-' after the 'e'/'E'
                                _ = self.next_char();
                            }
                            _ => {}
                        }
                    }
                    let cached_index = self.cursor.get_current_index();
                    self.consume_decimal_digits();
                    if cached_index == self.cursor.get_current_index() {
                        return Err(RsProtocError::LexError(
                            "Expected decimal digits in exponent part of numeric literal"
                                .to_string(),
                        ));
                    }
                    span.end = self.cursor.get_current_index();
                }
                _ => {}
            }
        }
        Ok(span)
    }

    fn numeric_literal(&mut self, header: char) -> Option<Token<'storage>> {
        debug_assert!(header.is_numeric() || header == '.');
        // Note: At this point we've already consumed 1 character of the numeric literal from the cursor
        // The various components of a numeric literal:
        // [radix] int_part [. fract_part [[ep] [+-] exponent_part]]
        let start = self.cursor.get_current_index() - 1;
        let radix = self.determine_radix(header);
        let integral_part = match self.extract_integral_part(header, radix) {
            Ok(integral_part) => integral_part,
            Err(err) => {
                return Some(Token {
                    kind: self.get_error_token(err.to_string().as_str()),
                    metadata: self.get_token_metadata(Span {
                        start,
                        end: self.cursor.get_current_index(),
                    }),
                });
            }
        };
        let fractional_part: Span = self.extract_fractional_part(&integral_part, header);
        let exponent_part = match self.extract_exponent() {
            Ok(exponent_part) => exponent_part,
            Err(err) => {
                return Some(Token {
                    kind: self.get_error_token(err.to_string().as_str()),
                    metadata: self.get_token_metadata(Span {
                        start,
                        end: self.cursor.get_current_index(),
                    }),
                });
            }
        };
        let integral_value = {
            if !integral_part.is_empty() {
                match u64::from_str_radix(
                    integral_part.extract_from_source(self.source_text),
                    u32::from(radix),
                ) {
                    Ok(value) => value,
                    Err(err) => {
                        return Some(Token {
                            kind: self.get_error_token(err.to_string().as_str()),
                            metadata: self.get_token_metadata(Span {
                                start,
                                end: self.cursor.get_current_index(),
                            }),
                        });
                    }
                }
            } else {
                0u64
            }
        };
        if fractional_part.is_empty() && exponent_part.is_empty() && !integral_part.is_empty() {
            return Some(Token {
                kind: TokenKind::IntegerLiteral(integral_value),
                metadata: self.get_token_metadata(Span {
                    start,
                    end: self.cursor.get_current_index(),
                }),
            });
        }
        let mut floating_point_number = integral_value as f64;
        let fractional_value = {
            if fractional_part.is_empty() || fractional_part.len() == 1 {
                0f64
            } else {
                match f64::from_str(fractional_part.extract_from_source(self.source_text)) {
                    Ok(value) => value,
                    Err(err) => {
                        return Some(Token {
                            kind: self.get_error_token(err.to_string().as_str()),
                            metadata: self.get_token_metadata(Span {
                                start,
                                end: self.cursor.get_current_index(),
                            }),
                        });
                    }
                }
            }
        };
        floating_point_number += fractional_value;
        if exponent_part.is_empty() {
            return Some(Token {
                kind: TokenKind::FloatLiteral(floating_point_number),
                metadata: self.get_token_metadata(Span {
                    start,
                    end: self.cursor.get_current_index(),
                }),
            });
        }

        let exponent_value: i32 = {
            match i32::from_str_radix(exponent_part.extract_from_source(self.source_text), 10) {
                Ok(value) => value,
                Err(err) => {
                    return Some(Token {
                        kind: self.get_error_token(err.to_string().as_str()),
                        metadata: self.get_token_metadata(Span {
                            start,
                            end: self.cursor.get_current_index(),
                        }),
                    });
                }
            }
        };

        floating_point_number = floating_point_number * 10f64.powi(exponent_value);

        return Some(Token {
            kind: TokenKind::FloatLiteral(floating_point_number),
            metadata: self.get_token_metadata(Span {
                start,
                end: self.cursor.get_current_index(),
            }),
        });
    }

    fn string_literal(&mut self, string_literal_header: char) -> Option<Token<'storage>> {
        // We've already consumed the quote
        debug_assert!(string_literal_header == '\'' || string_literal_header == '\"');
        let string_literal_start_index = self.cursor.get_current_index();
        let mut escaped_sequence = String::new();
        loop {
            if let Some(ch) = self.next_char() {
                match ch {
                    '\n' => {
                        return Some(Token {
                            kind: self.get_error_token("Unterminated string literal"),
                            metadata: self.get_token_metadata(Span {
                                start: string_literal_start_index,
                                end: self.cursor.get_current_index(),
                            }),
                        });
                    }
                    '\x00' => {
                        return Some(Token {
                            kind: self.get_error_token("Unterminated string literal"),
                            metadata: self.get_token_metadata(Span {
                                start: string_literal_start_index,
                                end: self.cursor.get_current_index(),
                            }),
                        });
                    }
                    '\\' => {
                        // Start of escape sequence
                        if escaped_sequence.is_empty() {
                            // Trigger a dynamic allocation and capture all the characters until the start of the escape sequence
                            escaped_sequence.push_str(
                                &self.source_text[string_literal_start_index
                                    ..self.cursor.get_current_index() - 1],
                            );
                        }
                        if !self.consume_escape_sequence(&mut escaped_sequence) {
                            return Some(Token {
                                kind: self
                                    .get_error_token("Invalid escape sequence in string literal"),
                                metadata: self.get_token_metadata(Span {
                                    start: string_literal_start_index,
                                    end: self.cursor.get_current_index(),
                                }),
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
                                metadata: self.get_token_metadata(Span {
                                    start: string_literal_start_index,
                                    end: self.cursor.get_current_index(),
                                }),
                            });
                        } else {
                            return Some(Token {
                                kind: TokenKind::StringLiteral(YarnBox::new(
                                    &self.source_text[string_literal_start_index
                                        ..self.cursor.get_current_index() - 1],
                                )),
                                metadata: self.get_token_metadata(Span {
                                    start: string_literal_start_index,
                                    end: self.cursor.get_current_index(),
                                }),
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
                    kind: self.get_error_token("Unterminated string literal"),
                    metadata: self.get_token_metadata(Span {
                        start: string_literal_start_index,
                        end: self.cursor.get_current_index(),
                    }),
                });
            }
        }
    }

    fn next_token(&mut self) -> Option<Token<'storage>> {
        self.consume_whitespace_and_comments();
        while let Some(ch) = self.next_char() {
            match ch {
                ';' => {
                    return Some(Token {
                        kind: TokenKind::Semicolon,
                        metadata: self.get_token_metadata(Span {
                            start: self.cursor.get_current_index() - 1,
                            end: self.cursor.get_current_index(),
                        }),
                    })
                }
                ':' => {
                    return Some(Token {
                        kind: TokenKind::Colon,
                        metadata: self.get_token_metadata(Span {
                            start: self.cursor.get_current_index() - 1,
                            end: self.cursor.get_current_index(),
                        }),
                    })
                }
                '(' => {
                    return Some(Token {
                        kind: TokenKind::LParen,
                        metadata: self.get_token_metadata(Span {
                            start: self.cursor.get_current_index() - 1,
                            end: self.cursor.get_current_index(),
                        }),
                    })
                }
                '[' => {
                    return Some(Token {
                        kind: TokenKind::LBracket,
                        metadata: self.get_token_metadata(Span {
                            start: self.cursor.get_current_index() - 1,
                            end: self.cursor.get_current_index(),
                        }),
                    })
                }
                ',' => {
                    return Some(Token {
                        kind: TokenKind::Comma,
                        metadata: self.get_token_metadata(Span {
                            start: self.cursor.get_current_index() - 1,
                            end: self.cursor.get_current_index(),
                        }),
                    })
                }
                '=' => {
                    return Some(Token {
                        kind: TokenKind::Equals,
                        metadata: self.get_token_metadata(Span {
                            start: self.cursor.get_current_index() - 1,
                            end: self.cursor.get_current_index(),
                        }),
                    })
                }
                ')' => {
                    return Some(Token {
                        kind: TokenKind::RParen,
                        metadata: self.get_token_metadata(Span {
                            start: self.cursor.get_current_index() - 1,
                            end: self.cursor.get_current_index(),
                        }),
                    })
                }
                ']' => {
                    return Some(Token {
                        kind: TokenKind::RBracket,
                        metadata: self.get_token_metadata(Span {
                            start: self.cursor.get_current_index() - 1,
                            end: self.cursor.get_current_index(),
                        }),
                    })
                }
                '.' => {
                    if let Some(next_ch) = self.cursor.peek() {
                        if next_ch.is_numeric() {
                            return self.numeric_literal(ch);
                        }
                    }
                    return Some(Token {
                        kind: TokenKind::Dot,
                        metadata: self.get_token_metadata(Span {
                            start: self.cursor.get_current_index() - 1,
                            end: self.cursor.get_current_index(),
                        }),
                    });
                }
                '-' => {
                    return Some(Token {
                        kind: TokenKind::Minus,
                        metadata: self.get_token_metadata(Span {
                            start: self.cursor.get_current_index() - 1,
                            end: self.cursor.get_current_index(),
                        }),
                    })
                }
                '{' => {
                    return Some(Token {
                        kind: TokenKind::LBrace,
                        metadata: self.get_token_metadata(Span {
                            start: self.cursor.get_current_index() - 1,
                            end: self.cursor.get_current_index(),
                        }),
                    })
                }
                '<' => {
                    return Some(Token {
                        kind: TokenKind::LAngle,
                        metadata: self.get_token_metadata(Span {
                            start: self.cursor.get_current_index() - 1,
                            end: self.cursor.get_current_index(),
                        }),
                    })
                }
                '/' => {
                    return Some(Token {
                        kind: TokenKind::Slash,
                        metadata: self.get_token_metadata(Span {
                            start: self.cursor.get_current_index() - 1,
                            end: self.cursor.get_current_index(),
                        }),
                    });
                }
                '+' => {
                    return Some(Token {
                        kind: TokenKind::Plus,
                        metadata: self.get_token_metadata(Span {
                            start: self.cursor.get_current_index() - 1,
                            end: self.cursor.get_current_index(),
                        }),
                    })
                }
                '}' => {
                    return Some(Token {
                        kind: TokenKind::RBrace,
                        metadata: self.get_token_metadata(Span {
                            start: self.cursor.get_current_index() - 1,
                            end: self.cursor.get_current_index(),
                        }),
                    })
                }
                '>' => {
                    return Some(Token {
                        kind: TokenKind::RAngle,
                        metadata: self.get_token_metadata(Span {
                            start: self.cursor.get_current_index() - 1,
                            end: self.cursor.get_current_index(),
                        }),
                    })
                }
                '\'' | '"' => return self.string_literal(ch),
                '0'..='9' => return self.numeric_literal(ch),
                'a'..='z' | 'A'..='Z' | '_' => return self.identifier_or_keyword(ch),
                _ => {
                    return Some(Token {
                        kind: self.get_error_token("Unknown character"),
                        metadata: self.get_token_metadata(Span {
                            start: self.cursor.get_current_index() - 1,
                            end: self.cursor.get_current_index(),
                        }),
                    })
                }
            }
        }
        None
    }

    fn get_error_token(&mut self, message: &str) -> TokenKind<'storage> {
        self.seen_error = true;
        let mut error_message = format!("Lexer error {}\n", message);
        error_message += format!("{}", self.current_line_number).as_str();

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
                    _ = self.next_char(); // Consume the whitespace and move ahead
                    continue;
                }
            }
            // At the first non-whitespace/non-comment character
            break;
        }
    }

    fn next_char(&mut self) -> Option<char> {
        match self.cursor.next_with_index() {
            Some((_, ch)) => {
                if ch == '\n' {
                    self.current_line_number += 1;
                    self.current_line_column = 1;
                    self.current_line_start_char_offset = self.cursor.get_current_index();
                } else if ch == '\t' {
                    self.current_line_column += 4;
                } else {
                    self.current_line_column += 1;
                }
                return Some(ch);
            }
            None => return None,
        }
    }

    /// https://protobuf.com/docs/language-spec#whitespace-and-comments
    fn consume_single_line_comment(&mut self) {
        debug_assert!(self.cursor.peek().is_some());
        debug_assert!(self.cursor.peek().unwrap() == '/');
        _ = self.next_char(); // Consume the "/"
        debug_assert!(self.cursor.peek().is_some());
        debug_assert!(self.cursor.peek().unwrap() == '/');
        _ = self.next_char(); // Consume the "/"
        while let Some(ch) = self.next_char() {
            if ch == '\n' || ch == '\x00' {
                break;
            }
        }
    }
    /// https://protobuf.com/docs/language-spec#whitespace-and-comments
    fn consume_block_comment(&mut self) {
        debug_assert!(self.cursor.peek().is_some());
        debug_assert!(self.cursor.peek().unwrap() == '/');
        _ = self.next_char(); // Consume the "/"
        debug_assert!(self.cursor.peek().is_some());
        debug_assert!(self.cursor.peek().unwrap() == '*');
        _ = self.next_char(); // Consume the "*"
        while let Some(ch) = self.next_char() {
            if ch == '*' {
                if let Some(next_ch) = self.next_char() {
                    if next_ch == '/' {
                        break;
                    }
                }
            }
        }
    }

    fn consume_hex_escape_sequence(&mut self, escaped_string: &mut String) -> bool {
        let mut decoded_char: u32;
        if let Some(first_required_char) = self.next_char() {
            if first_required_char.is_ascii_hexdigit() {
                let digit = first_required_char.to_digit(16).unwrap(); // SAFETY: We  just checked above that the character is a valid hex digit.
                decoded_char = digit;
                if let Some(second_optional_character) = self.cursor.peek() {
                    if second_optional_character.is_ascii_hexdigit() {
                        _ = self.next_char(); // Consume the second hex digit
                        let lower_nibble = second_optional_character.to_digit(16).unwrap(); // SAFETY: We  just checked above that the character is a valid hex digit.
                        let upper_nibble = decoded_char << 4;
                        decoded_char = upper_nibble | lower_nibble;
                    }
                }
                escaped_string.push(std::char::from_u32(decoded_char).unwrap()/*Unwrap here as we've validated above that we are combing two valid nibbles*/);
                return true;
            }
        }
        return false;
    }

    fn consume_octal_escape_sequence(
        &mut self,
        first_octal_digit: char,
        escaped_string: &mut String,
    ) {
        let is_octal_digit = |ch: char| match ch {
            '0'..='7' => true,
            _ => false,
        };
        assert!(is_octal_digit(first_octal_digit));
        let mut decoded_byte: u32 = first_octal_digit.to_digit(8).unwrap(); // SAFETY: We  just checked above that the character is a valid hex digit.;
        for _ in 1..=2 {
            if let Some(optional_digit) = self.cursor.peek() {
                if is_octal_digit(optional_digit) {
                    let optional_digit = optional_digit.to_digit(8).unwrap();
                    decoded_byte = decoded_byte << 3 | optional_digit;
                    _ = self.next_char(); // Consume the digit
                }
            }
        }
        escaped_string.push(std::char::from_u32(decoded_byte).unwrap());
    }

    fn consume_unicode_escape_sequence(
        &mut self,
        escaped_string: &mut String,
        header: char,
    ) -> bool {
        debug_assert!(header == 'u' || header == 'U');
        let mut consume_n_hex_digits = |n: usize| -> bool {
            let mut decoded_value: u32 = 0;
            for _ in 0..n {
                match self.next_char() {
                    Some(ch) => {
                        if ch.is_ascii_hexdigit() {
                            let nibble = ch.to_digit(16).unwrap();
                            decoded_value = (decoded_value << 4) | nibble; // SAFETY: We  just checked above that the character is a valid hex digit.
                        } else {
                            // Found non hex digit
                            return false;
                        }
                    }
                    None => return false, // Ran out of digits
                }
            }
            let decode_result = std::char::from_u32(decoded_value);
            if decode_result.is_none() {
                return false;
            }
            escaped_string.push(decode_result.unwrap());
            return true;
        };

        match header {
            'u' => consume_n_hex_digits(4),
            'U' => consume_n_hex_digits(8),
            _ => unreachable!(),
        }
    }

    fn consume_escape_sequence(&mut self, escaped_string: &mut String) -> bool {
        match self.next_char() {
            Some(ch) => match ch {
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
                    self.consume_octal_escape_sequence(ch, escaped_string);
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

fn get_keyword_token_kind<'a>(text: &'a str) -> Option<TokenKind<'a>> {
    const TABLE: [(&str, TokenKind); 39] = [
        ("import", TokenKind::Import),
        ("syntax", TokenKind::Syntax),
        ("bool", TokenKind::Bool),
        ("to", TokenKind::To),
        ("oneOf", TokenKind::OneOf),
        ("float", TokenKind::Float),
        ("double", TokenKind::Double),
        ("map", TokenKind::Map),
        ("weak", TokenKind::Weak),
        ("int32", TokenKind::Int32),
        ("extensions", TokenKind::Extensions),
        ("public", TokenKind::Public),
        ("int64", TokenKind::Int64),
        ("package", TokenKind::Package),
        ("uint32", TokenKind::Uint32),
        ("max", TokenKind::Max),
        ("option", TokenKind::Option),
        ("uint64", TokenKind::Uint64),
        ("reserved", TokenKind::Reserved),
        ("inf", TokenKind::Inf),
        ("sint32", TokenKind::Sint32),
        ("enum", TokenKind::Enum),
        ("repeated", TokenKind::Repeated),
        ("sint64", TokenKind::Sint64),
        ("message", TokenKind::Message),
        ("optional", TokenKind::Optional),
        ("fixed32", TokenKind::Fixed32),
        ("extend", TokenKind::Extend),
        ("required", TokenKind::Required),
        ("fixed64", TokenKind::Fixed64),
        ("service", TokenKind::Service),
        ("sfixed32", TokenKind::SFixed32),
        ("rpc", TokenKind::Rpc),
        ("string", TokenKind::String),
        ("sfixed64", TokenKind::SFixed64),
        ("stream", TokenKind::Stream),
        ("bytes", TokenKind::Bytes),
        ("group", TokenKind::Group),
        ("returns", TokenKind::Returns),
    ];
    match TABLE
        .into_iter()
        .find(|(keyword_string, _)| *keyword_string == text)
    {
        Some((_, kind)) => Some(kind),
        None => None,
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Radix {
    Decimal,
    Hexadecimal,
    Octal,
}

impl From<Radix> for u32 {
    fn from(value: Radix) -> Self {
        match value {
            Radix::Decimal => 10,
            Radix::Hexadecimal => 16,
            Radix::Octal => 8,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
struct LineInfo {
    line_start_offset_into_source: usize,
    line_number: usize,
    column_number: usize,
}

#[derive(Clone)]
struct Span {
    start: usize,
    end: usize,
}

impl Span {
    fn len(&self) -> usize {
        debug_assert!(self.end >= self.start);
        self.end - self.start
    }
    fn is_empty(&self) -> bool {
        debug_assert!(self.end >= self.start);
        self.end == self.start
    }
    fn extract_from_source<'a>(&self, source: &'a str) -> &'a str {
        debug_assert!(self.end >= self.start);
        if self.is_empty() {
            ""
        } else {
            &source[self.start..self.end]
        }
    }
}

#[derive(Clone)]
struct Cursor<'source> {
    iter: Chars<'source>,
    number_of_chars_consumed: usize,
}

impl<'source> Cursor<'source> {
    fn new(source_text: &'source str) -> Self {
        Self {
            iter: source_text.chars(),
            number_of_chars_consumed: 0,
        }
    }

    fn next_with_index(&mut self) -> Option<(usize, char)> {
        if let Some(ch) = self.iter.next() {
            let index = self.number_of_chars_consumed;
            self.number_of_chars_consumed += 1;
            Some((index, ch))
        } else {
            None
        }
    }

    fn get_current_index(&self) -> usize {
        self.number_of_chars_consumed
    }

    fn peek(&self) -> Option<char> {
        self.iter.clone().next()
    }

    fn peek_next(&self) -> Option<char> {
        let mut iter = self.iter.clone();
        match iter.next() {
            Some(_) => match iter.next() {
                Some(ch) => Some(ch),
                None => None,
            },
            None => None,
        }
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
    fn test_string_literal() {
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
    fn test_string_literal_newline_escape() {
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
    fn test_string_literal_hex_escape() {
        let mut lexer = Lexer::new("'First\\x09Second'");
        let result = lexer.next();
        assert!(result.is_some());
        let token = result.unwrap();
        match token.kind {
            TokenKind::StringLiteral(string) => {
                assert!(string == "First\tSecond");
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_string_literal_octal_escape() {
        {
            let mut lexer = Lexer::new("'First\\011Second'");
            let result = lexer.next();
            assert!(result.is_some());
            let token = result.unwrap();
            match token.kind {
                TokenKind::StringLiteral(string) => {
                    assert!(string == "First\tSecond");
                }
                _ => assert!(false),
            }
        }
        {
            let mut lexer = Lexer::new("'First\\12Second'");
            let result = lexer.next();
            assert!(result.is_some());
            let token = result.unwrap();
            match token.kind {
                TokenKind::StringLiteral(string) => {
                    assert!(string == "First\nSecond");
                }
                _ => assert!(false),
            }
        }
    }

    #[test]
    fn test_string_literal_unicode() {
        {
            let mut lexer = Lexer::new(
                "'Long unicode escape can represent emojis \\U0001F389 but isn\\'t necessary 🎉'",
            );
            let result = lexer.next();
            assert!(result.is_some());
            let token = result.unwrap();
            match token.kind {
                TokenKind::StringLiteral(string) => {
                    println!("{}", string);
                    assert!(
                        string
                            == "Long unicode escape can represent emojis 🎉 but isn't necessary 🎉"
                    );
                }
                _ => assert!(false),
            }
        }
        {
            let mut lexer =
                Lexer::new("'A unicode right arrow can use unicode escape \\u2192 or not →'");
            let result = lexer.next();
            assert!(result.is_some());
            let token = result.unwrap();
            match token.kind {
                TokenKind::StringLiteral(string) => {
                    println!("{}", string);
                    assert!(string == "A unicode right arrow can use unicode escape → or not →");
                }
                _ => assert!(false),
            }
        }
    }

    #[test]
    fn test_numerical_literal_floats() {
        let mut lexer = Lexer::new("12.56e-12 .5 1e3 1. 0.0 .123 555.555 1.234e-12 .953e20 5E+40");
        {
            let result = lexer.next();
            assert!(result.is_some());
            let token = result.unwrap();
            match token.kind {
                TokenKind::FloatLiteral(float_value) => {
                    const GROUND_TRUTH: f64 = 12.56e-12;
                    assert!((GROUND_TRUTH - float_value).abs() < 2.0f64 * &f64::EPSILON);
                }
                _ => assert!(false),
            }
        }
        {
            let result = lexer.next();
            assert!(result.is_some());
            let token = result.unwrap();
            match token.kind {
                TokenKind::FloatLiteral(float_value) => {
                    const GROUND_TRUTH: f64 = 0.5;
                    assert!((GROUND_TRUTH - float_value).abs() < 2.0f64 * &f64::EPSILON);
                }
                _ => assert!(false),
            }
        }

        {
            let result = lexer.next();
            assert!(result.is_some());
            let token = result.unwrap();
            match token.kind {
                TokenKind::FloatLiteral(float_value) => {
                    const GROUND_TRUTH: f64 = 1e3;
                    assert!((GROUND_TRUTH - float_value).abs() < 2.0f64 * &f64::EPSILON);
                }
                _ => assert!(false),
            }
        }

        {
            let result = lexer.next();
            assert!(result.is_some());
            let token = result.unwrap();
            match token.kind {
                TokenKind::FloatLiteral(float_value) => {
                    const GROUND_TRUTH: f64 = 1.0;
                    assert!((GROUND_TRUTH - float_value).abs() < 2.0f64 * &f64::EPSILON);
                }
                _ => assert!(false),
            }
        }

        {
            let result = lexer.next();
            assert!(result.is_some());
            let token = result.unwrap();
            match token.kind {
                TokenKind::FloatLiteral(float_value) => {
                    const GROUND_TRUTH: f64 = 0.0;
                    assert!((GROUND_TRUTH - float_value).abs() < 2.0f64 * &f64::EPSILON);
                }
                _ => assert!(false),
            }
        }

        {
            let result = lexer.next();
            assert!(result.is_some());
            let token = result.unwrap();
            match token.kind {
                TokenKind::FloatLiteral(float_value) => {
                    const GROUND_TRUTH: f64 = 0.123;
                    assert!((GROUND_TRUTH - float_value).abs() < 2.0f64 * &f64::EPSILON);
                }
                _ => assert!(false),
            }
        }

        {
            let result = lexer.next();
            assert!(result.is_some());
            let token = result.unwrap();
            match token.kind {
                TokenKind::FloatLiteral(float_value) => {
                    const GROUND_TRUTH: f64 = 555.555;
                    assert!((GROUND_TRUTH - float_value).abs() < 2.0f64 * &f64::EPSILON);
                }
                _ => assert!(false),
            }
        }

        {
            let result = lexer.next();
            assert!(result.is_some());
            let token = result.unwrap();
            match token.kind {
                TokenKind::FloatLiteral(float_value) => {
                    const GROUND_TRUTH: f64 = 1.234e-12;
                    assert!((GROUND_TRUTH - float_value).abs() < 2.0f64 * &f64::EPSILON);
                }
                _ => assert!(false),
            }
        }

        {
            let result = lexer.next();
            assert!(result.is_some());
            let token = result.unwrap();
            match token.kind {
                TokenKind::FloatLiteral(float_value) => {
                    const GROUND_TRUTH: f64 = 0.953e20;
                    assert!((GROUND_TRUTH - float_value).abs() < 2.0f64 * &f64::EPSILON);
                }
                _ => assert!(false),
            }
        }

        {
            let result = lexer.next();
            assert!(result.is_some());
            let token = result.unwrap();
            match token.kind {
                TokenKind::FloatLiteral(float_value) => {
                    const GROUND_TRUTH: f64 = 5E+40;
                    assert!((GROUND_TRUTH - float_value).abs() < 2.0f64 * &f64::EPSILON);
                }
                _ => assert!(false),
            }
        }
    }

    #[test]
    fn test_numerical_literal_integers1() {
        let mut lexer = Lexer::new("184467440737095516151 123 0123 0x123");
        {
            let result = lexer.next();
            assert!(result.is_some());
            let token = result.unwrap();
            match token.kind {
                TokenKind::Error(_) => { /*We expect an error here as  184467440737095516151 is > u64::MAX*/
                }
                _ => assert!(false),
            }
        }
        {
            let result = lexer.next();
            assert!(result.is_some());
            let token = result.unwrap();
            match token.kind {
                TokenKind::IntegerLiteral(value) => {
                    assert!(value == 123)
                }
                _ => assert!(false),
            }
        }

        {
            let result = lexer.next();
            assert!(result.is_some());
            let token = result.unwrap();
            match token.kind {
                TokenKind::IntegerLiteral(value) => {
                    assert!(value == 0o123)
                }
                _ => assert!(false),
            }
        }

        {
            let result = lexer.next();
            assert!(result.is_some());
            let token = result.unwrap();
            match token.kind {
                TokenKind::IntegerLiteral(value) => {
                    assert!(value == 0x123)
                }
                _ => assert!(false),
            }
        }
        {
            let mut lexer = Lexer::new("08");
            let result = lexer.next();
            assert!(result.is_some());
            let token = result.unwrap();
            match token.kind {
                TokenKind::IntegerLiteral(value) => {
                    assert!(value == 0)
                }
                _ => assert!(false),
            }
        }

        {
            let mut lexer = Lexer::new("0xz");
            let result = lexer.next();
            assert!(result.is_some());
            let token = result.unwrap();
            match token.kind {
                TokenKind::Error(_) => { /*We expect an error here as  0xz is an invalid hex literal*/
                }
                _ => assert!(false),
            }
        }
    }

    #[test]
    fn test_identifier() {
        let mut lexer = Lexer::new("_test_variable1 test_variable2");
        let result = lexer.next();
        assert!(result.is_some());
        let token = result.unwrap();
        match token.kind {
            TokenKind::Identifier(value) => {
                assert!(value == "_test_variable1")
            }
            _ => assert!(false),
        }

        let result = lexer.next();
        assert!(result.is_some());
        let token = result.unwrap();
        match token.kind {
            TokenKind::Identifier(value) => {
                assert!(value == "test_variable2")
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_keywords() {
        let source_text = r#"
        import
        syntax
        bool
        to
        oneOf
        float
        double
        map
        weak
        int32
        extensions
        public
        int64
        package
        uint32
        max
        option
        uint64
        reserved
        inf
        sint32
        enum
        repeated
        sint64
        message
        optional
        fixed32
        extend
        required
        fixed64
        service
        sfixed32
        rpc
        string
        sfixed64
        stream
        bytes
        group
        returns
        "#;
        // Intentionally creating a separate mapping compared to the implementation
        fn get_keyword_token_kind<'a>(text: &'a str) -> Option<TokenKind<'a>> {
            const TABLE: [(&str, TokenKind); 39] = [
                ("import", TokenKind::Import),
                ("syntax", TokenKind::Syntax),
                ("bool", TokenKind::Bool),
                ("to", TokenKind::To),
                ("oneOf", TokenKind::OneOf),
                ("float", TokenKind::Float),
                ("double", TokenKind::Double),
                ("map", TokenKind::Map),
                ("weak", TokenKind::Weak),
                ("int32", TokenKind::Int32),
                ("extensions", TokenKind::Extensions),
                ("public", TokenKind::Public),
                ("int64", TokenKind::Int64),
                ("package", TokenKind::Package),
                ("uint32", TokenKind::Uint32),
                ("max", TokenKind::Max),
                ("option", TokenKind::Option),
                ("uint64", TokenKind::Uint64),
                ("reserved", TokenKind::Reserved),
                ("inf", TokenKind::Inf),
                ("sint32", TokenKind::Sint32),
                ("enum", TokenKind::Enum),
                ("repeated", TokenKind::Repeated),
                ("sint64", TokenKind::Sint64),
                ("message", TokenKind::Message),
                ("optional", TokenKind::Optional),
                ("fixed32", TokenKind::Fixed32),
                ("extend", TokenKind::Extend),
                ("required", TokenKind::Required),
                ("fixed64", TokenKind::Fixed64),
                ("service", TokenKind::Service),
                ("sfixed32", TokenKind::SFixed32),
                ("rpc", TokenKind::Rpc),
                ("string", TokenKind::String),
                ("sfixed64", TokenKind::SFixed64),
                ("stream", TokenKind::Stream),
                ("bytes", TokenKind::Bytes),
                ("group", TokenKind::Group),
                ("returns", TokenKind::Returns),
            ];
            match TABLE
                .into_iter()
                .find(|(keyword_string, _)| *keyword_string == text)
            {
                Some((_, kind)) => Some(kind),
                None => None,
            }
        }
        let lexeme_text_vector: Vec<&str> = source_text.split_ascii_whitespace().collect();
        let mut lexer = Lexer::new(source_text);
        for keyword_text in lexeme_text_vector {
            let result = lexer.next();
            assert!(result.is_some());
            let result_token_kind_from_lexer = result.unwrap().kind;
            let expected_token_kind = get_keyword_token_kind(keyword_text).unwrap();
            assert!(result_token_kind_from_lexer == expected_token_kind);
        }
    }

    #[test]
    fn test_actual_proto() {
        let source_text = r#"
        message Person {
            optional string name = 1;
            optional int32 id = 2;
            optional string email = 3;
        }
        "#;
        let expected_token_kinds = vec![
            TokenKind::Message,
            TokenKind::Identifier(YarnBox::new("Person")),
            TokenKind::LBrace,
            TokenKind::Optional,
            TokenKind::String,
            TokenKind::Identifier(YarnBox::new("name")),
            TokenKind::Equals,
            TokenKind::IntegerLiteral(1),
            TokenKind::Semicolon,
            TokenKind::Optional,
            TokenKind::Int32,
            TokenKind::Identifier(YarnBox::new("id")),
            TokenKind::Equals,
            TokenKind::IntegerLiteral(2),
            TokenKind::Semicolon,
            TokenKind::Optional,
            TokenKind::String,
            TokenKind::Identifier(YarnBox::new("email")),
            TokenKind::Equals,
            TokenKind::IntegerLiteral(3),
            TokenKind::Semicolon,
            TokenKind::RBrace,
        ];
        let actual_token_kinds: Vec<TokenKind> = Lexer::new(source_text)
            .into_iter()
            .map(|token| token.kind)
            .collect();
        assert!(expected_token_kinds == actual_token_kinds);
    }
    #[test]
    fn test_token_metadata() {
        let source_text = r#"
        message Person {
            optional string name = 1;
            optional int32 id = 2;
            optional string email = 3;
        }
        "#;
        let lexer = Lexer::new(source_text);
        let tokens: Vec<Token> = lexer.clone().collect();
        for token in tokens {
            lexer.print_token_in_line(&token.metadata);
        }
    }
}
