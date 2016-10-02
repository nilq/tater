use super::garden::Garden;
use super::tater::Tater;
use super::error::*;

mod mem;
mod sys;

pub trait Instruction {
    fn new(name: &str, arguments: &[&str], err: &Error) -> Box<Instruction> where Self:Sized;
    fn exec(&self, env: &mut Garden, tater: &Tater);
}

pub fn create_instruction(name: &str, arguments: &[&str], tater: &Tater, err: &Error) -> Box<Instruction> {
    if tater.print_parsed {
        println!("{}: {}", name, arguments.join(", "));
    }

    match name {
        "put" => mem::Put::new(name, arguments, err),
        "pop" => mem::Pop::new(name, arguments, err),
        "move" => mem::Move::new(name, arguments, err),

        "call" => sys::Call::new(name, arguments, err),
        "return" => sys::Return::new(name, arguments, err),
        "extern" => sys::Extern::new(name, arguments, err),

        /*
        "and" => logic::And::new(name, arguments, err),
        "or"  => logic::Or::new(name, arguments, err),
        "xor" => logic::Xor::new(name, arguments, err),
        "nor" => logic::Not::new(name, arguments, err),
        "left"  => logic::Left::new(name, arguments, err),
        "right" => logic::Right::new(name, arguments, err),

        "add" => math::Add::new(name, arguments, err),
        "sub" => math::Sub::new(name, arguments, err),
        "mul" => math::Mul::new(name, arguments, err),
        "div" => math::Div::new(name, arguments, err),
        "mod" => math::Mod::new(name, arguments, err),*/

        n => err.throw(ErrorType::NonExistent {
            type_name: "instruction".to_string(),
            value: n.to_string(),
        })
    }
}
