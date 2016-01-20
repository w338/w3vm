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
pub enum Instruction<A> {
    Get(A),
    Set(A),
    Push,
    Pop(A),
    Call(A),
    Return,
    Throw,
    Catch,
    Jump(A),
    Branch(A),
    Blank,
    Halt,
    Construct,
    Store(A),
    Load(A),
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulus,
    Is,
    Less,
    LessEqual,
    Equal,
    Greater,
    GreaterEqual,
    Right,
    Left,
    And,
    Or,
    Xor,
    Not,
    Grow(A),
    Shrink(A),
    F2I,
    I2F
}

fn parse_instruction(inst: &str, arg: Option<String>) -> Result<Instruction<String>, String> {
    match (inst, arg) {
        ("get", Some(arg))     => Ok(Instruction::Get(arg)),
        ("set", Some(arg))     => Ok(Instruction::Set(arg)),
        ("push", None)         => Ok(Instruction::Push),
        ("pop", Some(arg))     => Ok(Instruction::Pop(arg)),
        ("call", Some(arg))    => Ok(Instruction::Call(arg)),
        ("return", None)       => Ok(Instruction::Return),
        ("throw", None)        => Ok(Instruction::Throw),
        ("catch", None)        => Ok(Instruction::Catch),
        ("jump", Some(arg))    => Ok(Instruction::Jump(arg)),
        ("branch", Some(arg))  => Ok(Instruction::Branch(arg)),
        ("blank", None)        => Ok(Instruction::Blank),
        ("halt", None)         => Ok(Instruction::Halt),
        ("construct", None)    => Ok(Instruction::Construct),
        ("store", Some(arg))   => Ok(Instruction::Store(arg)),
        ("load", Some(arg))    => Ok(Instruction::Load(arg)),
        ("is", None)           => Ok(Instruction::Is),
        ("add", None)          => Ok(Instruction::Add),
        ("subtract", None)     => Ok(Instruction::Subtract),
        ("multiply", None)     => Ok(Instruction::Multiply),
        ("divide", None)       => Ok(Instruction::Divide),
        ("modulus", None)      => Ok(Instruction::Modulus),
        ("less", None)         => Ok(Instruction::Less),
        ("lessequal", None)    => Ok(Instruction::LessEqual),
        ("equal", None)        => Ok(Instruction::Equal),
        ("greater", None)      => Ok(Instruction::Greater),
        ("greaterequal", None) => Ok(Instruction::GreaterEqual),
        ("right", None)        => Ok(Instruction::Right),
        ("left", None)         => Ok(Instruction::Left),
        ("and", None)          => Ok(Instruction::And),
        ("or", None)           => Ok(Instruction::Or),
        ("xor", None)          => Ok(Instruction::Xor),
        ("not", None)          => Ok(Instruction::Not),
        ("grow", Some(arg))    => Ok(Instruction::Grow(arg)),
        ("shrink", Some(arg))  => Ok(Instruction::Shrink(arg)),
        ("f2i", None)          => Ok(Instruction::F2I),
        ("i2f", None)          => Ok(Instruction::I2F),
        (inst, arg)            => {
          Err(format!("Unknown instruction {:?}({:?})", inst, arg))
        }
    }
}
