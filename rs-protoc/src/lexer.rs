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
}

struct TokenInfo {
    /// The kind of token
    token_kind: TokenKind,
    /// Line on which the Token starts.
    token_line: u32,
    /// Zero-based byte offset of the token within its line.
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
    source_text: &'a SourceBuffer,
    token_info_vec: Vec<TokenInfo>,
    string_literal_storage: Vec<String>,
    integer_literal_storage: Vec<u64>,
    float_literal_storage: Vec<f64>,
    identifier_reference_storage: Vec<&'a str>,
}

impl<'a> TokenizedBuffer<'a> {
    pub fn new(source_text: &'a SourceBuffer) -> TokenizedBuffer<'a> {
        todo!()
    }

    pub fn size(&self) -> usize {
        self.token_info_vec.len()
    }

    pub fn get_token_iterator(&self) -> TokenIterator {
        TokenIterator {
            index: 0,
            length: self.size() as i32,
        }
    }

    pub fn get_token_kind(&self, token: &Token) -> TokenKind {
        self.token_info_vec[token.index as usize].token_kind
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {}
}
