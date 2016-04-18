use std::u8;
use std::u16;
use std::u32;
use std::u64;
use std::i8;
use std::i16;
use std::i32;
use std::i64;
use std::f32;
use std::f64;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Object
}

pub fn number_tag_to_type(tag: &str) -> Result<Type, String> {
    match tag {
        "u8"  => Ok(Type::U8),
        "u16" => Ok(Type::U16),
        "u32" => Ok(Type::U32),
        "i8"  => Ok(Type::I8),
        "i16" => Ok(Type::I16),
        "i32" => Ok(Type::I32),
        "f32" => Ok(Type::F32),
        "f64" => Ok(Type::F64),
        tag   => Err(format!("Uknown numeric tag {}", tag))
    }
}


pub fn max_integer_value_of_type(tp: &Type) -> u64 {
    match *tp {
        Type::U8  => u8::MAX as u64,
        Type::U16 => u16::MAX as u64,
        Type::U32 => u32::MAX as u64,
        Type::U64 => u64::MAX as u64,
        Type::I8  => i8::MAX as u64,
        Type::I16 => i16::MAX as u64,
        Type::I32 => i32::MAX as u64,
        Type::I64 => i64::MAX as u64,
        Type::F32 => f32::MAX as u64,
        Type::F64 => f64::MAX as u64,
        _         => panic!("Expected numeric type")
    }
}

pub fn min_integer_value_of_type(tp: &Type) -> i64 {
    match *tp {
        Type::U8  => u8::MIN as i64,
        Type::U16 => u16::MIN as i64,
        Type::U32 => u32::MIN as i64,
        Type::U64 => u64::MIN as i64,
        Type::I8  => i8::MIN as i64,
        Type::I16 => i16::MIN as i64,
        Type::I32 => i32::MIN as i64,
        Type::I64 => i64::MIN as i64,
        Type::F32 => f32::MIN as i64,
        Type::F64 => f64::MIN as i64,
        _         => panic!("Expected numeric type")
    }
}

pub fn integer_fits_in_type(number: u64, negative: bool, tp: &Type) -> bool {
    if negative {
        if number > 0 {
            // This handles the one more negative value allowed by twos completement.
            number - 1 <= -(min_integer_value_of_type(tp) + 1) as u64
        } else {
            true
        }
    } else {
        number <= max_integer_value_of_type(tp)
    }
}

fn make_negative_i64(number: u64) -> i64 {
    if number != 0 {
        // This handles the one more negative value allowed by twos completement.
        -((number - 1u64) as i64) - 1
    } else {
        number as i64
    }
}

pub fn shrink_integer(number: u64, negative: bool, target_type: &Type) -> Option<Number> {
    if integer_fits_in_type(number, negative, target_type) {
        if negative {
            match *target_type {
                Type::I8  => Some(Number::I8(-(number as i8))),
                Type::I16 => Some(Number::I16(-(number as i16))),
                Type::I32 => Some(Number::I32(-(number as i32))),
                Type::I64 => Some(Number::I64(make_negative_i64(number))),
                Type::F32 => Some(Number::F32(-(number as f32))),
                Type::F64 => Some(Number::F64(-(number as f64))),
                _         => unreachable!()
            }
        } else {
            match *target_type {
                Type::U8  => Some(Number::U8(number as u8)),
                Type::U16 => Some(Number::U16(number as u16)),
                Type::U32 => Some(Number::U32(number as u32)),
                Type::U64 => Some(Number::U64(number as u64)),
                Type::I8  => Some(Number::I8(number as i8)),
                Type::I16 => Some(Number::I16(number as i16)),
                Type::I32 => Some(Number::I32(number as i32)),
                Type::I64 => Some(Number::I64(number as i64)),
                Type::F32 => Some(Number::F32(number as f32)),
                Type::F64 => Some(Number::F64(number as f64)),
                _         => unreachable!()
            }
        }
    } else {
        None
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Number {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Verb {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulus,
    Is,
    Equal,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    ShiftLeft,
    ShiftRight,
    And,
    Or,
    Xor,
    Not,
    F2I,
    I2F
}

fn parse_instruction(inst: &str, arg: Option<i64>) -> Result<Verb, String> {
    match (inst, arg) {
        ("+", None)           => Ok(Verb::Add),
        ("-", None)           => Ok(Verb::Subtract),
        ("*", None)           => Ok(Verb::Multiply),
        ("/", None)           => Ok(Verb::Divide),
        ("%", None)           => Ok(Verb::Modulus),
        ("is", None)          => Ok(Verb::Is),
        ("==", None)          => Ok(Verb::Equal),
        ("<", None)           => Ok(Verb::Less),
        ("<=", None)          => Ok(Verb::LessEqual),
        (">", None)           => Ok(Verb::Greater),
        (">=", None)          => Ok(Verb::GreaterEqual),
        ("<<", None)          => Ok(Verb::ShiftLeft),
        (">>", None)          => Ok(Verb::ShiftRight),
        ("&", None)           => Ok(Verb::And),
        ("|", None)           => Ok(Verb::Or),
        ("^", None)           => Ok(Verb::Xor),
        ("!", None)           => Ok(Verb::Not),
        ("f2i", None)         => Ok(Verb::F2I),
        ("i2f", None)         => Ok(Verb::I2F),
        (inst, arg)           => {
          Err(format!("Unknown verb {:?}({:?})", inst, arg))
        }
    }
}
