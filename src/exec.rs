use std::result;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Type {
    Imprecise,
    Uncalculated,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Val {
    Imprecise(f64),
    Uncalculated,
}

fn type_of(v: Val) -> Type {
    match v {
        Val::Imprecise(_) => Type::Imprecise,
        Val::Uncalculated => Type::Uncalculated,
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ExecError {
    Type(Verb, Type)
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

fn run_instructions(instructions: &[Instruction], frame: &mut [Val]) -> Result<()> {
    let mut pc = 0;
    while pc < instructions.len() {
        let inst = instructions[pc];
        match inst.verb {
            Verb::Add => match (frame[inst.src], frame[inst.tgt]) {
                (Val::Imprecise(x), Val::Imprecise(y)) => frame[inst.dst] = Val::Imprecise(x + y),
                (Val::Imprecise(_), y) => return Err(ExecError::Type(Verb::Add, type_of(y))),
                (x, _) => return Err(ExecError::Type(Verb::Add, type_of(x))),
            },
            _   => unimplemented!()
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
