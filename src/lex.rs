use symbol;
use val;

use unicode_xid::UnicodeXID;
use std::sync::{Arc};

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Identifier(symbol::Symbol),
    Number(val::Number),
    String(Arc<String>),
    Whitespace(symbol::Symbol),
    Operator(symbol::Symbol)
}

pub struct Lexer<'a> {
    source: &'a str,
    lead_index: usize,
    lead_char: char,
    chars: ::std::str::CharIndices<'a>,
    table: &'a mut symbol::Table
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str, table: &'a mut symbol::Table) -> Self {
        let mut chars = source.char_indices();
        if let Some((index, char)) = chars.next() {
            Lexer {
                source: source,
                lead_index: index,
                lead_char: char,
                chars: chars,
                table: table
            }
        } else {
            Lexer {
                source: source,
                lead_index: 0,
                lead_char: '\0',
                chars: chars,
                table: table
            }
        }
    }
}

impl<'a> Lexer<'a> {
    fn step(&mut self, start_index: usize, end_index: usize, lead_char: char) -> &'a str {
        self.lead_index = end_index;
        self.lead_char = lead_char;
        &self.source[start_index..end_index]
    }

    fn intern(&mut self, source: &str) -> symbol::Symbol {
        self.table.intern(source)
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;
    fn next(&mut self) -> Option<Token> {
        let start_index = self.lead_index;
        let mut s = &self.source[start_index..];
        if self.lead_char == '\0' {
            return None;
        } else if UnicodeXID::is_xid_start(self.lead_char) {
            // We have an identifier.
            while let Some((index, char)) = self.chars.next() {
                if !UnicodeXID::is_xid_continue(char) {
                    s = self.step(start_index, index, char);
                    break;
                }
            }
            return Some(Token::Identifier(self.intern(s)));
        } else if self.lead_char.is_whitespace() {
            // We have some whitespace.
            while let Some((index, char)) = self.chars.next() {
                if !char.is_whitespace() {
                    s = self.step(start_index, index, char);
                    break;
                }
            }
            return Some(Token::Whitespace(self.intern(s)));
        } else if let Some(digit) = self.lead_char.to_digit(10) {
            // We have a number.
            if let Some((index, char)) = self.chars.next() {
                if char.is_digit(10) || char == '_' || char == '.' || char == 'e' || char == 'E' {
                    // Decimal (integer of floating point).
                    let mut iter = ::std::iter::once((index, char)).chain(s.char_indices());
                    let mut saw_decimal = false;
                    let mut saw_exponent = false;
                    while let Some((index, char)) = iter.next() {
                        if !char.is_digit(10) {
                            if char == '.' &&
                               !saw_decimal &&
                               s[(index + 1)..].starts_with(|c: char| c.is_digit(10) ||
                                                                                c == 'e' || 
                                                                                c == 'E') {
                                saw_decimal = true;
                            } else if char == 'e' &&
                                      !saw_exponent &&
                                      s[(index + 1)..].starts_with(|c: char| c.is_digit(10)) {
                                saw_exponent = true;
                            } else {
                                s = self.step(start_index, index, char);
                                break;
                            }
                        }
                    }
                    // TODO(w338): Implement number tags.
                    if saw_exponent || saw_decimal {
                        return Some(Token::Number(val::Number::F64(s.parse().unwrap())));
                    } else {
                        return Some(Token::Number(val::Number::I64(s.parse().unwrap())));
                    }
                } else {
                    let start_index = index + 1;
                    let mut s = &self.source[start_index..];
                    if digit == 0 && char == 'x' {
                        while let Some((index, char)) = self.chars.next() {
                            if !char.is_digit(16) {
                                s = self.step(start_index, index, char);
                                break;
                            }
                        }
                        println!("s = {:?}", s);
                        return Some(Token::Number(val::Number::U64(u64::from_str_radix(s, 16).unwrap())));
                    } else if digit == 0 && char == 'o' {
                        while let Some((index, char)) = self.chars.next() {
                            if !char.is_digit(8) {
                                s = self.step(start_index, index, char);
                                break;
                            }
                        }
                        return Some(Token::Number(val::Number::U64(u64::from_str_radix(s, 8).unwrap())));
                    } else if digit == 0 && char == 'b' {
                        while let Some((index, char)) = self.chars.next() {
                            if !char.is_digit(2) {
                                s = self.step(start_index, index, char);
                                break;
                            }
                        }
                        return Some(Token::Number(val::Number::U64(u64::from_str_radix(s, 2).unwrap())));
                    } else {
                        self.lead_index = index;
                        self.lead_char = char;
                    }
                }
            }
            return Some(Token::Number(val::Number::I64(digit as i64)));
        } else {
            // We must have an operator.
            //let start_index = self.lead_index;
            panic!("what {:?}", self.lead_char);
        }
        unimplemented!()
    }
}

#[test]
fn it_lexes_identifiers() {
    let mut tab = symbol::Table::new();
    let next = tab.intern("test");
    assert_eq!(Lexer::new("test", &mut tab).next(), Some(Token::Identifier(next.clone())));
    assert_eq!(Lexer::new("test ", &mut tab).next(), Some(Token::Identifier(next)));
}

#[test]
fn it_lexes_whitespace() {
    let mut tab = symbol::Table::new();
    let next = tab.intern("    ");
    assert_eq!(Lexer::new("    ", &mut tab).next(), Some(Token::Whitespace(next.clone())));
    assert_eq!(Lexer::new("    test", &mut tab).next(), Some(Token::Whitespace(next)));
}

#[test]
fn it_lexes_decimals() {
    let mut tab = symbol::Table::new();
    assert_eq!(Lexer::new("0", &mut tab).next(), Some(Token::Number(val::Number::I64(0))));
    assert_eq!(Lexer::new("0 ", &mut tab).next(), Some(Token::Number(val::Number::I64(0))));
    assert_eq!(Lexer::new("99", &mut tab).next(), Some(Token::Number(val::Number::I64(99))));
}

#[test]
fn it_lexes_floats() {
    let mut tab = symbol::Table::new();
    assert_eq!(Lexer::new("1.0", &mut tab).next(), Some(Token::Number(val::Number::F64(1.0))));
    assert_eq!(Lexer::new("1e0", &mut tab).next(), Some(Token::Number(val::Number::F64(1.0))));
    assert_eq!(Lexer::new("1.e0", &mut tab).next(), Some(Token::Number(val::Number::F64(1.0))));
}

#[test]
fn it_lexes_hexadecimals() {
    let mut tab = symbol::Table::new();
    assert_eq!(Lexer::new("0x1", &mut tab).next(), Some(Token::Number(val::Number::U64(1))));
    assert_eq!(Lexer::new("0x1 ", &mut tab).next(), Some(Token::Number(val::Number::U64(1))));
}
