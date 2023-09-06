use std::str::Chars;

use crate::error::Result;
use crate::source_text::SourceBuffer;

pub struct Token {
    index: i32,
}

#[derive(Clone, Copy)]
pub enum TokenKind {
    Identifier,
    IntegerLiteral,
    FloatLiteral,
    StringLiteral,
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

struct TokenInfo {
    /// The kind of token
    token_kind: TokenKind,
    /// Line on which the Token starts.
    token_line: u32,
    /// Zero-based character offset of the token within its line.
    token_column_offset: u32,
}

pub struct TokenIterator {
    index: i32,
    length: i32,
}

impl<'a> Iterator for TokenIterator {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.length {
            let index = self.index;
            self.index += 1;
            Some(Token { index })
        } else {
            None
        }
    }
}

struct TokenizedBuffer<'a> {
    source_buffer: &'a SourceBuffer,
    token_info_vec: Vec<TokenInfo>,
    string_literal_storage: Vec<String>,
    integer_literal_storage: Vec<u64>,
    float_literal_storage: Vec<f64>,
    identifier_reference_storage: Vec<&'a str>,
    current_line: u32,
}

type SliceView<'a> = &'a str;

fn drop_first(source_text: &mut SliceView, number_of_characters_to_drop: usize) {
    if source_text.len() >= number_of_characters_to_drop {
        let original_slice = *source_text;
        *source_text = &original_slice[number_of_characters_to_drop..];
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

impl<'a> TokenizedBuffer<'a> {
    pub fn new(source_buffer: &'a SourceBuffer) -> TokenizedBuffer<'a> {
        let mut tokenized_buffer = TokenizedBuffer {
            source_buffer,
            token_info_vec: vec![],
            string_literal_storage: vec![],
            integer_literal_storage: vec![],
            float_literal_storage: vec![],
            identifier_reference_storage: vec![],
            current_line: 1,
        };
        tokenized_buffer.lex(&mut source_buffer.text());
        tokenized_buffer
    }
    fn consume_single_line_comment(&mut self, source_text: &mut SliceView) {
        debug_assert!(source_text.starts_with("//"));
        drop_first(source_text, 2);
        let mut remaining_chars = source_text.chars();
        while let Some(ch) = remaining_chars.next() {
            if ch == '\n' {
                self.current_line += 1;
                *source_text = remaining_chars.as_str();
                return;
            }
            if ch == '\x00' {
                *source_text = remaining_chars.as_str();
                return;
            }
        }
        *source_text = "";
    }

    fn consume_block_comment(&mut self, source_text: &mut SliceView) {
        debug_assert!(source_text.starts_with("/*"));
        drop_first(source_text, 2);
        let mut remaining_chars = source_text.chars();
        while let Some(ch) = remaining_chars.next() {
            if ch == '*' {
                if let Some(second_ch) = remaining_chars.next() {
                    if second_ch == '/' {
                        *source_text = remaining_chars.as_str();
                        return;
                    }
                }
            } else if ch == '\n' {
                self.current_line += 1;
            }
        }
        *source_text = "";
    }

    fn consume_whitespace(&mut self, source_text: &mut SliceView) {
        let mut chars = source_text.chars();
        while let Some(ch) = chars.next() {
            if ch == '\n' {
                self.current_line += 1;
            }
            if !is_whitespace(ch) {
                break;
            }
        }
    }

    /// https://protobuf.com/docs/language-spec#whitespace-and-comments
    fn skip_whitespace_and_comments(&mut self, source_text: &mut SliceView) {
        while !source_text.is_empty() {
            if source_text.starts_with("//") {
                self.consume_single_line_comment(source_text);
            } else if source_text.starts_with("/*") {
                self.consume_block_comment(source_text)
            } else if is_whitespace(source_text.chars().nth(0).unwrap()) {
                self.consume_whitespace(source_text)
            } else {
                break;
            }
        }
    }
    fn lex(&mut self, source_text: &mut SliceView) {
        loop {
            if (source_text.is_empty()) {
                self.token_info_vec.push(TokenInfo {
                    token_kind: TokenKind::Eof,
                    token_line: todo!(),
                    token_column_offset: todo!(),
                })
            }
            self.skip_whitespace_and_comments(source_text);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
