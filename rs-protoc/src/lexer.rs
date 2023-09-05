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
}

type SliceView<'a> = &'a str;

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

fn skip_whitespace(source_text: &mut SliceView) -> bool {
    let mut chars = source_text.chars();
    if let Some(ch) = chars.next() {
        if is_whitespace(ch) {
            *source_text = chars.as_str();
            return true;
        }
    }
    false
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
        };
        tokenized_buffer.lex(&mut source_buffer.text());
        tokenized_buffer
    }
    fn lex(&mut self, source_text: &mut SliceView) {
        while !skip_whitespace(source_text) {
            todo!()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
