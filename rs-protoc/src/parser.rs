use crate::{
    error::{Result, RsProtocError},
    lexer::{self, TokenKind},
};
use std::collections::HashMap;

// Package hierarchy

// ─ Package
//    ├─ Messages
//    │   ├─ Fields
//    │   ├─ Oneofs
//    │   │   └─ Fields
//    │   ├─ Messages
//    │   │   └─ (...more...)
//    │   ├─ Enums
//    │   │   └─ Enum Values
//    │   └─ Extensions
//    │
//    ├─ Enums
//    │   └─ Enum Values
//    │
//    ├─ Extensions
//    │
//    └─ Services
//        └─ Methods

enum ElementType {
    Message(Vec<NamedElement>),
    Field(FieldPayload),
    OneOf(Vec<NamedElement>),
    Enum,
    EnumValue,
    Extension,
    Service(Vec<Method>),
    Method,
}

struct NamedElement {
    name: String,
    type_t: ElementType,
}

pub struct Package {
    named_elements: Vec<NamedElement>,
}

struct FieldPayload {}

struct Method {}

pub type PackageMap = HashMap<String, Package>;

pub struct Parser<'a> {
    token_iterator: std::iter::Peekable<lexer::Lexer<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(source_text: &str) -> Parser {
        Parser {
            token_iterator: lexer::Lexer::new(source_text).peekable(),
        }
    }

    fn consume(&mut self, expected_token_kind: &TokenKind) -> bool {
        if let Some(token) = self.token_iterator.peek() {
            if token.kind == *expected_token_kind {
                _ = self.token_iterator.next();
                return true;
            }
        }
        return false;
    }

    fn consume_multiple(&mut self, expected_tokens: &[TokenKind]) -> bool {
        for token in expected_tokens {
            if !self.consume(token) {
                return false;
            }
        }
        return true;
    }

    fn consume_syntax_declaration(&mut self) -> Result<()> {
        // "Should be: "syntax = "proto3""
        if !self.consume_multiple(&[TokenKind::Syntax, TokenKind::Equals]) {
            return Err(crate::error::RsProtocError::ParseError(
                "Expected syntax declaration of the form: \"syntax = proto3\"".to_string(),
            ));
        }
        if let Some(token) = self.token_iterator.peek() {
            if let TokenKind::StringLiteral(string_literal) = &token.kind {
                if string_literal != "proto3" {
                    return Err(crate::error::RsProtocError::ParseError(
                        "Expected syntax declaration of the form: \"syntax = proto3\"".to_string(),
                    ));
                }
            }
            _ = self.token_iterator.next();
        } else {
            return Err(crate::error::RsProtocError::ParseError(
                "Expected syntax declaration of the form: \"syntax = proto3\"".to_string(),
            ));
        }
        if !self.consume(&TokenKind::Semicolon) {
            return Err(crate::error::RsProtocError::ParseError(
                "Expected syntax declaration of the form: \"syntax = proto3\"".to_string(),
            ));
        }
        return Ok(());
    }

    fn parse(&mut self) -> Result<PackageMap> {
        self.consume_syntax_declaration()?;
        /// TODO: Handle import statements
        Ok(PackageMap::new())
    }
}

#[cfg(test)]
mod tests {
    use super::Parser;
    fn add_header(source: &str) -> String {
        let header = "syntax = \"proto3\";\
        package tests.test_package;";
        let mut output = String::from(header);
        output.push_str(source);
        output
    }

    #[test]
    fn parser_syntax_declaration_test() {
        {
            let source = add_header(
                r#"
            message SimpleMessage{
                 int32 field_one = 1;
            }
            "#,
            );
            let mut parser = Parser::new(&source);
            assert_eq!(true, parser.parse().is_ok());
        }

        {
            let source = "syntax = \"proto2\";";
            let mut parser = Parser::new(&source);
            assert_eq!(true, parser.parse().is_err());
        }

        {
            let source = "syntax = \"proto3\"";
            let mut parser = Parser::new(&source);
            assert_eq!(true, parser.parse().is_err());
        }
    }
}
