use symbol;
use val;

use unicode_xid::UnicodeXID;
use std::sync::{Arc};

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Identifier(symbol::Symbol),
    Number(val::Number),
    String(Arc<String>),
    BrokenString(String),
    Error(String),
    Whitespace(symbol::Symbol),
    Operator(symbol::Symbol)
}

pub struct Lexer<'a> {
    source: &'a str,
    chars: ::std::str::CharIndices<'a>,
    reversed: Vec<(usize, char)>,
    table: &'a mut symbol::Table
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str, table: &'a mut symbol::Table) -> Self {
        Lexer {
            source: source,
            chars: source.char_indices(),
            reversed: Vec::new(),
            table: table
        }
    }
}

impl<'a> Lexer<'a> {
    fn tick(&mut self) -> (usize, char) {
        if let Some(res) = self.reversed.pop() {
            res
        } else {
            match self.chars.next() {
                Some(res) => res,
                None => (self.source.len(), '\0')
            }
        }
    }

    fn untick(&mut self, index: usize, c: char) {
        self.reversed.push((index, c));
    }

    fn slice(&mut self, start_index: usize, end_index: usize) -> &'a str {
        &self.source[start_index..end_index]
    }

    fn intern(&mut self, source: &str) -> symbol::Symbol {
        self.table.intern(source)
    }

    fn slice_intern(&mut self, start_index: usize, end_index: usize) -> symbol::Symbol {
        let s = self.slice(start_index, end_index);
        self.intern(s)
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;
    fn next(&mut self) -> Option<Token> {
        let (first_index, first_char) =  self.tick();
        if UnicodeXID::is_xid_start(first_char) {
            // We have an identifier.
            loop {
                let (index, char) = self.tick();
                if !UnicodeXID::is_xid_continue(char) {
                    return Some(Token::Identifier(self.slice_intern(first_index, index)));
                }
            }
        } else if first_char.is_whitespace() {
            // We have some whitespace.
            loop {
                let (index, char) = self.tick();
                if !char.is_whitespace() {
                    return Some(Token::Whitespace(self.slice_intern(first_index, index)));
                }
            }
        } else if let Some(digit) = first_char.to_digit(10) {
            // We have a number.
            let (second_index, second_char) = self.tick();
            if second_char.is_digit(10) ||
               second_char == '_' ||
               second_char == '.' ||
               second_char == 'e' ||
               second_char == 'E' {
                // Decimal (integer or floating point).
                self.untick(second_index, second_char);
                let mut last_index;
                loop {
                    let (index, char) = self.tick();
                    if !char.is_digit(10) && char != '_' {
                        self.untick(index, char);
                        last_index = index;
                        break;
                    }
                }

                let mut saw_decimal = false;

                let (index, char) = self.tick();
                if char == '.' {
                    saw_decimal = true;
                    loop {
                        let (index, char) = self.tick();
                        if !char.is_digit(10) && char != '_' {
                            self.untick(index, char);
                            last_index = index;
                            break;
                        }
                    }
                } else {
                    self.untick(index, char);
                }

                let mut saw_exponent = false;

                let (index, char) = self.tick();
                if char == 'e' || char == 'E' {
                    let (index, char) = self.tick();
                    if char.is_digit(10) {
                        saw_exponent = true;
                        loop {
                            let (index, char) = self.tick();
                            if !char.is_digit(10) && char != '_' {
                                self.untick(index, char);
                                last_index = index;
                                break;
                            }
                        }
                    } else {
                        self.untick(index, char);
                    }
                }
                if !saw_exponent {
                    self.untick(index, char);
                }

                let s: String = self.slice(first_index, last_index).chars().filter(|&c| c != '_').collect();
                // TODO(w338): Implement number tags.
                if saw_exponent || saw_decimal {
                    return Some(Token::Number(val::Number::F64(s.parse().unwrap())));
                } else {
                    return Some(Token::Number(val::Number::I64(s.parse().unwrap())));
                }
            }
            if digit == 0 {
                if let Some(radix) = match second_char {
                        'x' => Some(16),
                        'o' => Some(8),
                        'b' => Some(2),
                        _ => None
                    } {
                    let (third_index, third_char) = self.tick();
                    if third_char.is_digit(radix) {
                        loop {
                            let (index, char) = self.tick();
                            if !char.is_digit(radix) && char != '_' {
                                let s: String = self.slice(third_index, index).chars().filter(|&c| c != '_').collect();
                                return Some(Token::Number(val::Number::U64(u64::from_str_radix(&s, radix).unwrap())));
                            }
                        }
                    }
                    self.untick(third_index, third_char);
                    self.untick(second_index, second_char);
                }
            }
            return Some(Token::Number(val::Number::I64(digit as i64)));
        } else if first_char == '"' {
            // We have a string.
            let second_index = first_index + 1;
            let mut output;
            if second_index < self.source.len() {
                if let Some(len_guess) = self.source[(second_index)..].find('"') {
                    output = String::with_capacity(len_guess);
                } else {
                    output = String::new();
                }
            } else {
                return Some(Token::BrokenString("".to_owned()));
            }
            loop {
                let (index, char) = self.tick();
                if char == '\0' {
                    return Some(Token::BrokenString(self.slice(second_index, index).to_owned()));
                }
                if char == '\\' {
                    let (index, char) = self.tick();
                    if char == '\0' {
                        return Some(Token::BrokenString(self.slice(second_index, index).to_owned()));
                    } else if char == 'n' {
                        output.push('\n');
                    } else if char == 't' {
                        output.push('\t');
                    } else if char == '\\' {
                        output.push('\\');
                    } else if char == '"' {
                        output.push('"');
                    } else if char == 'x' {
                        let (_, first_hex) = self.tick();
                        let (_, second_hex) = self.tick();
                        match (first_hex.to_digit(16), second_hex.to_digit(16)) {
                            (Some(first), Some(second)) => {
                                if let Some(c) = ::std::char::from_u32(first * 16 + second) {
                                    output.push(c);
                                } else {
                                    return Some(Token::Error("this form of character escape may only be used with characters in the range [\\x00-\\x7f]".to_owned()));
                                }
                            },
                            _ => {
                                return Some(Token::Error("numeric character escape is too short".to_owned()));
                            }
                        }
                    } else if char == 'u' {
                        let (open_brace_index, open_brace) = self.tick();
                        if open_brace != '{' {
                            return Some(Token::Error("incorrect unicode escape sequence".to_owned()));
                        }
                        let mut end_hex_index = open_brace_index;
                        for i in 0..8 {
                            let (index, char) = self.tick();
                            if char == '}' {
                                end_hex_index = index;
                                break;
                            } else if i == 6 {
                                return Some(Token::Error(
                                        "overlong unicode escape (can have at most 6 hex digits)".to_owned()));
                            } else if !char.is_digit(16) {
                                return Some(Token::Error(format!("invalid character in unicode escape: {}", char)));
                            }
                        }
                        let hex = self.slice(open_brace_index + 1, end_hex_index);
                        let code_point = u32::from_str_radix(hex, 16).unwrap();
                        if let Some(c) = ::std::char::from_u32(code_point) {
                            output.push(c);
                        } else {
                            return Some(Token::Error("invalid unicode character escape".to_owned()));
                        }
                    }
                } else if char == '"' {
                    return Some(Token::String(Arc::new(output)));
                } else {
                    output.push(char);
                }
            }
        } else {
            return None;
        }
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
    assert_eq!(Lexer::new("1_000", &mut tab).next(), Some(Token::Number(val::Number::I64(1000))));
}

#[test]
fn it_lexes_floats() {
    let mut tab = symbol::Table::new();
    assert_eq!(Lexer::new("1.0", &mut tab).next(), Some(Token::Number(val::Number::F64(1.0))));
    assert_eq!(Lexer::new("1e0", &mut tab).next(), Some(Token::Number(val::Number::F64(1.0))));
    assert_eq!(Lexer::new("1.e0", &mut tab).next(), Some(Token::Number(val::Number::F64(1.0))));
    assert_eq!(Lexer::new("1_e0", &mut tab).next(), Some(Token::Number(val::Number::F64(1.0))));
    // This one is not lexed by Rust. Should we allow it?
    assert_eq!(Lexer::new("0_e0", &mut tab).next(), Some(Token::Number(val::Number::F64(0.0))));
}

#[test]
fn it_lexes_hexadecimals() {
    let mut tab = symbol::Table::new();
    assert_eq!(Lexer::new("0x1", &mut tab).next(), Some(Token::Number(val::Number::U64(1))));
    assert_eq!(Lexer::new("0x1 ", &mut tab).next(), Some(Token::Number(val::Number::U64(1))));
}

#[test]
fn it_lexes_octals() {
    let mut tab = symbol::Table::new();
    assert_eq!(Lexer::new("0o1", &mut tab).next(), Some(Token::Number(val::Number::U64(1))));
    assert_eq!(Lexer::new("0o1 ", &mut tab).next(), Some(Token::Number(val::Number::U64(1))));
}

#[test]
fn it_lexes_strings() {
    let mut tab = symbol::Table::new();
    assert_eq!(Lexer::new("\"test\"", &mut tab).next(), Some(Token::String(Arc::new("test".to_owned()))));
    {
        let mut lexer = Lexer::new("\"", &mut tab);
        assert_eq!(lexer.next(), Some(Token::BrokenString("".to_owned())));
    }
    {
        let mut lexer = Lexer::new("\"a", &mut tab);
        assert_eq!(lexer.next(), Some(Token::BrokenString("a".to_owned())));
    }
    {
        let mut lexer = Lexer::new("\"\n\"", &mut tab);
        assert_eq!(lexer.next(), Some(Token::String(Arc::new("\n".to_owned()))));
    }
    {
        let mut lexer = Lexer::new("\"\t\"", &mut tab);
        assert_eq!(lexer.next(), Some(Token::String(Arc::new("\t".to_owned()))));
    }
    {
        let mut lexer = Lexer::new("\"\\u{0}\"", &mut tab);
        assert_eq!(lexer.next(), Some(Token::String(Arc::new("\u{0}".to_owned()))));
    }
    {
        let mut lexer = Lexer::new("\"\\x00\"", &mut tab);
        assert_eq!(lexer.next(), Some(Token::String(Arc::new("\x00".to_owned()))));
    }
    {
        let mut lexer = Lexer::new("\"\\u{1234}\"", &mut tab);
        assert_eq!(lexer.next(), Some(Token::String(Arc::new("\u{1234}".to_owned()))));
    }
    {
        let mut lexer = Lexer::new("\"\\u{000000}\"", &mut tab);
        assert_eq!(lexer.next(), Some(Token::String(Arc::new("\x00".to_owned()))));
    }
    {
        let mut lexer = Lexer::new("\"\\u{0000000}\"", &mut tab);
        assert_eq!(lexer.next(),
                   Some(Token::Error("overlong unicode escape (can have at most 6 hex digits)".to_owned())));
    }
    {
        let mut lexer = Lexer::new("\"\\u{00000000}\"", &mut tab);
        assert_eq!(lexer.next(),
                   Some(Token::Error("overlong unicode escape (can have at most 6 hex digits)".to_owned())));
    }
}

#[test]
fn it_lexes_binary() {
    let mut tab = symbol::Table::new();
    assert_eq!(Lexer::new("0b1", &mut tab).next(), Some(Token::Number(val::Number::U64(1))));
    assert_eq!(Lexer::new("0b1 ", &mut tab).next(), Some(Token::Number(val::Number::U64(1))));
}

#[test]
fn it_lexes_weird_combinations() {
    // All of the ones starting with 0 here are not lexed by Rust. Should we allow them?
    let mut tab = symbol::Table::new();
    let e = tab.intern("e");
    let tokene_ver = tab.intern("e_ver");
    let ever = tab.intern("ever");
    let x = tab.intern("x");
    let x_x = tab.intern("x_x");
    let o = tab.intern("o");
    let o_o = tab.intern("o_o");
    let b = tab.intern("b");
    let b_b = tab.intern("b_b");
    let big = tab.intern("big");
    {
        let mut lexer = Lexer::new("1e", &mut tab);
        assert_eq!(lexer.next(), Some(Token::Number(val::Number::I64(1))));
        assert_eq!(lexer.next(), Some(Token::Identifier(e)));
    }
    {
        let mut lexer = Lexer::new("1ever", &mut tab);
        assert_eq!(lexer.next(), Some(Token::Number(val::Number::I64(1))));
        assert_eq!(lexer.next(), Some(Token::Identifier(ever)));
    }
    {
        let mut lexer = Lexer::new("1e_ver", &mut tab);
        assert_eq!(lexer.next(), Some(Token::Number(val::Number::I64(1))));
        assert_eq!(lexer.next(), Some(Token::Identifier(tokene_ver)));
    }
    {
        let mut lexer = Lexer::new("0x", &mut tab);
        assert_eq!(lexer.next(), Some(Token::Number(val::Number::I64(0))));
        assert_eq!(lexer.next(), Some(Token::Identifier(x)));
    }
    {
        let mut lexer = Lexer::new("0x_x", &mut tab);
        assert_eq!(lexer.next(), Some(Token::Number(val::Number::I64(0))));
        assert_eq!(lexer.next(), Some(Token::Identifier(x_x)));
    }
    {
        let mut lexer = Lexer::new("0o", &mut tab);
        assert_eq!(lexer.next(), Some(Token::Number(val::Number::I64(0))));
        assert_eq!(lexer.next(), Some(Token::Identifier(o)));
    }
    {
        let mut lexer = Lexer::new("0o_o", &mut tab);
        assert_eq!(lexer.next(), Some(Token::Number(val::Number::I64(0))));
        assert_eq!(lexer.next(), Some(Token::Identifier(o_o)));
    }
    {
        let mut lexer = Lexer::new("0b", &mut tab);
        assert_eq!(lexer.next(), Some(Token::Number(val::Number::I64(0))));
        assert_eq!(lexer.next(), Some(Token::Identifier(b)));
    }
    {
        let mut lexer = Lexer::new("0b_b", &mut tab);
        assert_eq!(lexer.next(), Some(Token::Number(val::Number::I64(0))));
        assert_eq!(lexer.next(), Some(Token::Identifier(b_b)));
    }
    {
        let mut lexer = Lexer::new("0big", &mut tab);
        assert_eq!(lexer.next(), Some(Token::Number(val::Number::I64(0))));
        assert_eq!(lexer.next(), Some(Token::Identifier(big)));
    }
}
