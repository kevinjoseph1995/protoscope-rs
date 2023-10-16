use crate::{
    error::Result,
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

type ElementName = String;

enum NamedElement {
    MessageType(ElementName, Vec<NamedElement>),
    FieldType(ElementName, Field),
    OneOfType(ElementName, Vec<Field>),
    EnumType(ElementName),
    EnumValueType(ElementName),
    ExtensionType(ElementName),
    ServiceType(ElementName, Vec<Method>),
    MethodType(ElementName),
}

pub struct Package {
    named_elements: Vec<NamedElement>,
}

struct Field {}

struct Method {}

pub type PackageMap = HashMap<String, Package>;

pub struct Parser<'a> {
    source_text: &'a str,
    token_iterator: std::iter::Peekable<lexer::Lexer<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(source_text: &str) -> Parser {
        Parser {
            source_text,
            token_iterator: lexer::Lexer::new(source_text).peekable(),
        }
    }

    fn consume(&mut self, expected_token_kind: TokenKind) -> bool {
        if let Some(token) = self.token_iterator.peek() {
            if token.kind == expected_token_kind {
                _ = self.token_iterator.next();
                return true;
            }
        }
        return false;
    }

    fn consume_syntax_declaration(&mut self) -> Result<()> {
        if self.consume(TokenKind::Syntax) {
            if !self.consume(TokenKind::Equals) {
                return Err(crate::error::RsProtocError::ParseError(
                    "Expected syntax declaration of the form: \"syntax = proto3\"".to_string(),
                ));
            }
            if let Some(token) = self.token_iterator.peek() {
                if let TokenKind::StringLiteral(string_literal) = &token.kind {
                    if string_literal != "proto3" {
                        return Err(crate::error::RsProtocError::ParseError(
                            "Expected syntax declaration of the form: \"syntax = proto3\""
                                .to_string(),
                        ));
                    }
                }
                _ = self.token_iterator.next();
            } else {
                return Err(crate::error::RsProtocError::ParseError(
                    "Expected syntax declaration of the form: \"syntax = proto3\"".to_string(),
                ));
            }
            if !self.consume(TokenKind::Semicolon) {
                return Err(crate::error::RsProtocError::ParseError(
                    "Expected syntax declaration of the form: \"syntax = proto3\"".to_string(),
                ));
            }
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
