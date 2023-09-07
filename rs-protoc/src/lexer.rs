use std::str::CharIndices;

use crate::source_text::SourceBuffer;

#[derive(Clone, Copy)]
pub enum TokenKind<'storage> {
    Identifier(&'storage str),
    IntegerLiteral(&'storage str),
    FloatLiteral(&'storage str),
    StringLiteral(&'storage str),
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
    Eof,
    Error, /* Not a valid token that can be consumed by the parser */
}

pub struct Token<'storage> {
    kind: TokenKind<'storage>,
    line_number: usize,  // 1 based line number
    column_index: usize, // 1 based column index
    character_index: usize,
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

    fn next(&mut self) -> Option<char> {
        match self.iter.next() {
            Some((_, ch)) => Some(ch),
            None => None,
        }
    }

    fn next_with_index(&mut self) -> Option<(usize, char)> {
        self.iter.next()
    }

    fn is_eof(&self) -> bool {
        self.peek().is_none()
    }

    fn peek(&self) -> Option<char> {
        match self.iter.clone().next() {
            Some((_, ch)) => Some(ch),
            None => None,
        }
    }

    fn peek_with_index(&self) -> Option<(usize, char)> {
        self.iter.clone().next()
    }

    fn peek_index(&self) -> Option<usize> {
        match self.iter.clone().next() {
            Some((index, _)) => Some(index),
            None => None,
        }
    }
}

struct Lexer<'storage> {
    cursor: Cursor<'storage>,
    current_line_column: usize,
    current_line_number: usize,
}

impl<'storage> Lexer<'storage> {
    fn new(source_text: &'storage str) -> Self {
        Lexer {
            cursor: Cursor::new(source_text),
            current_line_column: 0,
            current_line_number: 0,
        }
    }

    fn identifier_or_keyword(&mut self) -> Option<Token<'storage>> {
        todo!()
    }

    fn numeric_literal(&mut self) -> Option<Token<'storage>> {
        todo!()
    }

    fn string_literal(&mut self) -> Option<Token<'storage>> {
        todo!()
    }

    fn next_token(&mut self) -> Option<Token<'storage>> {
        if let Some(ch) = self.cursor.peek() {
            if is_whitespace(ch) {
                self.consume_whitespace();
            }
        }
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
                    if let Some(ch) = self.cursor.peek() {
                        if ch == '/' {
                            self.consume_single_line_comment();
                            continue;
                        } else if ch == '*' {
                            self.consume_block_comment();
                            continue;
                        }
                    }
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
                '\'' | '"' => return self.string_literal(),
                '0'..='9' => return self.numeric_literal(),
                'a'..='z' | 'A'..='Z' | '_' => return self.identifier_or_keyword(),
                _ => todo!(),
            }
        }
        None
    }

    /// https://protobuf.com/docs/language-spec#whitespace-and-comments
    fn consume_whitespace(&mut self) {
        debug_assert!(self.cursor.peek().is_some());
        debug_assert!(is_whitespace(self.cursor.peek().unwrap()));
        while let Some((_, ch)) = self.next_char_with_index() {
            if !is_whitespace(ch) {
                // First non-whitespace character
                return;
            }
        }
    }

    fn next_char_with_index(&mut self) -> Option<(usize, char)> {
        match self.cursor.next_with_index() {
            Some((index, ch)) => {
                if ch == '\n' {
                    self.current_line_number += 1;
                    self.current_line_column = 1;
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
        while let Some((_, ch)) = self.next_char_with_index() {
            if ch == '\n' || ch == '\x00' {
                break;
            }
        }
    }
    /// https://protobuf.com/docs/language-spec#whitespace-and-comments
    fn consume_block_comment(&mut self) {
        debug_assert!(self.cursor.peek().is_some());
        debug_assert!(self.cursor.peek().unwrap() == '*');
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
        ' ' => true,
        '\n' => true,
        '\r' => true,
        '\t' => true,
        '\x0c' => true, // Form-feed
        '\x0b' => true, // Vertical-tab
        _ => false,
    }
}

// impl<'a> TokenizedBuffer<'a> {
//     pub fn new(source_buffer: &'a SourceBuffer) -> TokenizedBuffer<'a> {
//         let mut tokenized_buffer = TokenizedBuffer {
//             source_buffer,
//             token_info_vec: vec![],
//             string_literal_storage: vec![],
//             integer_literal_storage: vec![],
//             float_literal_storage: vec![],
//             identifier_reference_storage: vec![],
//             current_line: 1,
//         };
//         tokenized_buffer.lex(&mut source_buffer.text());
//         tokenized_buffer
//     }
//     fn consume_single_line_comment(&mut self, source_text: &mut SliceView) {
//         debug_assert!(source_text.starts_with("//"));
//         drop_first(source_text, 2);
//         let mut remaining_chars = source_text.chars();
//         while let Some(ch) = remaining_chars.next() {
//             if ch == '\n' {
//                 self.current_line += 1;
//                 *source_text = remaining_chars.as_str();
//                 return;
//             }
//             if ch == '\x00' {
//                 *source_text = remaining_chars.as_str();
//                 return;
//             }
//         }
//         *source_text = "";
//     }

//     fn consume_block_comment(&mut self, source_text: &mut SliceView) {
//         debug_assert!(source_text.starts_with("/*"));
//         drop_first(source_text, 2);
//         let mut remaining_chars = source_text.chars();
//         while let Some(ch) = remaining_chars.next() {
//             if ch == '*' {
//                 if let Some(second_ch) = remaining_chars.next() {
//                     if second_ch == '/' {
//                         *source_text = remaining_chars.as_str();
//                         return;
//                     }
//                 }
//             } else if ch == '\n' {
//                 self.current_line += 1;
//             }
//         }
//         *source_text = "";
//     }

//     fn consume_whitespace(&mut self, source_text: &mut SliceView) {
//         let mut chars = source_text.chars();
//         while let Some(ch) = chars.next() {
//             if ch == '\n' {
//                 self.current_line += 1;
//             }
//             if !is_whitespace(ch) {
//                 break;
//             }
//         }
//     }

//     /// https://protobuf.com/docs/language-spec#whitespace-and-comments
//     fn skip_whitespace_and_comments(&mut self, source_text: &mut SliceView) {
//         while !source_text.is_empty() {
//             if source_text.starts_with("//") {
//                 self.consume_single_line_comment(source_text);
//             } else if source_text.starts_with("/*") {
//                 self.consume_block_comment(source_text)
//             } else if is_whitespace(source_text.chars().nth(0).unwrap()) {
//                 self.consume_whitespace(source_text)
//             } else {
//                 break;
//             }
//         }
//     }
//     fn lex(&mut self, source_text: &mut SliceView) {
//         loop {
//             if (source_text.is_empty()) {
//                 self.token_info_vec.push(TokenInfo {
//                     token_kind: TokenKind::Eof,
//                     token_line: todo!(),
//                     token_column_offset: todo!(),
//                 })
//             }
//             self.skip_whitespace_and_comments(source_text);
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
}
