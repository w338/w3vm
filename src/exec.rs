use std::result;
use std::usize;

const INVALID_REGISTER: usize = usize::MAX;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Type {
    Imprecise,
    Integer,
    Bool,
    Uncalculated,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Val {
    Integer(i64),
    Imprecise(f64),
    Bool(bool),
    Uncalculated,
}

fn type_of(v: Val) -> Type {
    match v {
        Val::Imprecise(_) => Type::Imprecise,
        Val::Integer(_) => Type::Integer,
        Val::Bool(_) => Type::Bool,
        Val::Uncalculated => Type::Uncalculated,
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ExecError {
    Type(Verb, Type),
    Invalid(String),
    DivisionByZero,
}

type Result<T> = result::Result<T, ExecError>;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
    ShiftRightSigned,
    And,
    Or,
    Xor,
    Not
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Instruction {
    verb: Verb,
    src: usize,
    tgt: usize,
    dst: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Function {
    instructions: Vec<Instruction>
}

fn run_imprecise_verb(verb: Verb, x: f64, y: f64) -> Val {
    match verb {
        Verb::Add => Val::Imprecise(x + y),
        Verb::Subtract => Val::Imprecise(x - y),
        Verb::Multiply => Val::Imprecise(x * y),
        Verb::Divide => Val::Imprecise(x / y),
        Verb::Modulus => Val::Imprecise(x % y),
        Verb::Less => Val::Bool(x < y),
        Verb::LessEqual => Val::Bool(x <= y),
        Verb::Greater => Val::Bool(x > y),
        Verb::GreaterEqual => Val::Bool(x >= y),
        _ => panic!(format!("Not an Imprecise verb: {:?}", verb))
    }
}

fn run_integer_verb(verb: Verb, x: i64, y: i64) -> Result<Val> {
    match verb {
        Verb::Add => Ok(Val::Integer(x + y)),
        Verb::Subtract => Ok(Val::Integer(x - y)),
        Verb::Multiply => Ok(Val::Integer(x * y)),
        Verb::Divide => if y == 0 {
            Err(ExecError::DivisionByZero)
        } else {
            Ok(Val::Integer(x / y))
        },
        Verb::Modulus => Ok(Val::Integer(x % y)),
        Verb::Less => Ok(Val::Bool(x < y)),
        Verb::LessEqual => Ok(Val::Bool(x <= y)),
        Verb::Greater => Ok(Val::Bool(x > y)),
        Verb::GreaterEqual => Ok(Val::Bool(x >= y)),
        Verb::Equal => Ok(Val::Bool(x == y)),
        Verb::And => Ok(Val::Integer(x & y)),
        Verb::Or => Ok(Val::Integer(x | y)),
        Verb::Xor => Ok(Val::Integer(x ^ y)),
        Verb::ShiftLeft => Ok(Val::Integer(x << y)),
        Verb::ShiftRight => Ok(Val::Integer(((x as u64) >> y) as i64)),
        Verb::ShiftRightSigned => Ok(Val::Integer(x >> y)),
        _ => panic!(format!("Not an Integer verb: {:?}", verb))
    }
}

fn run_bool_verb(verb: Verb, x: bool, y: bool) -> Val {
    match verb {
        Verb::And => Val::Bool(x & y),
        Verb::Or => Val::Bool(x | y),
        Verb::Xor => Val::Bool(x ^ y),
        _ => panic!(format!("Not an Bool verb: {:?}", verb))
    }
}

fn run_instructions(instructions: &[Instruction], frame: &mut [Val]) -> Result<()> {
    let mut pc = 0;
    while pc < instructions.len() {
        let inst = instructions[pc];
        match inst.verb {
            Verb::Add | Verb::Subtract | Verb::Multiply | Verb::Divide | Verb::Modulus | Verb::Less
                | Verb::LessEqual | Verb::Greater | Verb::GreaterEqual => match (frame[inst.src],
                                                                                 frame[inst.tgt]) {
                (Val::Imprecise(x), Val::Imprecise(y)) => frame[inst.dst] = run_imprecise_verb(inst.verb, x, y),
                (Val::Imprecise(x), Val::Integer(y)) => frame[inst.dst] = run_imprecise_verb(inst.verb, x, y as f64),
                (Val::Integer(x), Val::Imprecise(y)) => frame[inst.dst] = run_imprecise_verb(inst.verb, x as f64, y),
                (Val::Integer(x), Val::Integer(y)) => match run_integer_verb(inst.verb, x, y) {
                    Ok(res) => frame[inst.dst] = res,
                    Err(e) => return Err(e),
                },
                (Val::Imprecise(_), y) => return Err(ExecError::Type(inst.verb, type_of(y))),
                (x, _) => return Err(ExecError::Type(inst.verb, type_of(x)))
            },
            Verb::Not => match (frame[inst.src], inst.tgt) {
                (Val::Integer(x), INVALID_REGISTER) => frame[inst.dst] = Val::Integer(!x),
                (Val::Bool(x), INVALID_REGISTER) => frame[inst.dst] = Val::Bool(!x),
                (_, INVALID_REGISTER) => return Err(ExecError::Invalid("Not should not have a target".to_owned())),
                (x, _) => return Err(ExecError::Type(inst.verb, type_of(x)))
            },
            Verb::And | Verb::Or | Verb::Xor => match (frame[inst.src], frame[inst.tgt]) {
                (Val::Integer(x), Val::Integer(y)) => frame[inst.dst] = run_integer_verb(inst.verb, x, y).unwrap(),
                (Val::Bool(x), Val::Bool(y)) => frame[inst.dst] = run_bool_verb(inst.verb, x, y),
                (Val::Integer(_), y) => return Err(ExecError::Type(inst.verb, type_of(y))),
                (Val::Bool(_), y) => return Err(ExecError::Type(inst.verb, type_of(y))),
                (x, _) => return Err(ExecError::Type(inst.verb, type_of(x)))
            },
            Verb::ShiftLeft | Verb::ShiftRight | Verb::ShiftRightSigned => match (frame[inst.src], frame[inst.tgt]) {
                (Val::Integer(x), Val::Integer(y)) => frame[inst.dst] = run_integer_verb(inst.verb, x, y).unwrap(),
                (Val::Integer(_), y) => return Err(ExecError::Type(inst.verb, type_of(y))),
                (x, _) => return Err(ExecError::Type(inst.verb, type_of(x)))
            },
            Verb::Equal => match (frame[inst.src], frame[inst.tgt]) {
                (Val::Integer(x), Val::Integer(y)) => frame[inst.dst] = Val::Bool(x == y),
                (Val::Imprecise(x), Val::Imprecise(y)) => frame[inst.dst] = Val::Bool(x == y),
                (Val::Bool(x), Val::Bool(y)) => frame[inst.dst] = Val::Bool(x == y),
                (Val::Integer(x), Val::Imprecise(y)) => frame[inst.dst] = Val::Bool(x as f64 == y),
                (Val::Imprecise(x), Val::Integer(y)) => frame[inst.dst] = Val::Bool(x == y as f64),
                (_, _) => frame[inst.dst] = Val::Bool(false),
            },
            Verb::Is => match (frame[inst.src], frame[inst.tgt]) {
                (Val::Integer(x), Val::Integer(y)) => frame[inst.dst] = Val::Bool(x == y),
                (Val::Imprecise(x), Val::Imprecise(y)) => {
                    if x.is_nan() && y.is_nan() {
                        frame[inst.dst] = Val::Bool(true)
                    } else {
                        frame[inst.dst] = Val::Bool(x == y)
                    }
                },
                (Val::Bool(x), Val::Bool(y)) => frame[inst.dst] = Val::Bool(x == y),
                (_, _) => frame[inst.dst] = Val::Bool(false),
            }
        }
        pc += 1;
    }
    return Ok(());
}

#[test]
fn it_adds() {
    let func = Function{
        instructions: vec![
            Instruction{
                verb: Verb::Add,
                src: 0,
                tgt: 1,
                dst: 2,
            }
        ]
    };
    let mut frame = vec![
        Val::Imprecise(1f64),
        Val::Imprecise(2f64),
        Val::Uncalculated,
    ];
    let res = run_instructions(&func.instructions, frame.as_mut_slice());
    assert!(res.is_ok());
    assert_eq!(frame[2], Val::Imprecise(3f64));
}

#[test]
fn it_subtracts() {
    let func = Function{
        instructions: vec![
            Instruction{
                verb: Verb::Subtract,
                src: 0,
                tgt: 1,
                dst: 2,
            }
        ]
    };
    let mut frame = vec![
        Val::Integer(1i64),
        Val::Integer(2i64),
        Val::Uncalculated,
    ];
    let res = run_instructions(&func.instructions, frame.as_mut_slice());
    assert!(res.is_ok());
    assert_eq!(frame[2], Val::Integer(-1i64));
}

#[test]
fn it_errors_on_integer_div_zero() {
    let func = Function{
        instructions: vec![
            Instruction{
                verb: Verb::Divide,
                src: 0,
                tgt: 1,
                dst: 2,
            }
        ]
    };
    let mut frame = vec![
        Val::Integer(1i64),
        Val::Integer(0i64),
        Val::Uncalculated,
    ];
    let res = run_instructions(&func.instructions, frame.as_mut_slice());
    assert!(!res.is_ok());
    assert_eq!(frame[2], Val::Uncalculated);
}

#[test]
fn it_float_div_produces_infinity() {
    let func = Function{
        instructions: vec![
            Instruction{
                verb: Verb::Divide,
                src: 0,
                tgt: 1,
                dst: 2,
            }
        ]
    };
    let mut frame = vec![
        Val::Imprecise(1f64),
        Val::Imprecise(0f64),
        Val::Uncalculated,
    ];
    let res = run_instructions(&func.instructions, frame.as_mut_slice());
    assert!(res.is_ok());
    if let Val::Imprecise(v) = frame[2] {
        assert!(v.is_infinite())
    } else {
        assert!(false)
    }
}
