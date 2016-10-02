extern crate gmp;

use super::super::value::Value;
use super::super::garden::Garden;
use super::super::tater::Tater;
use super::super::error::*;

use super::Instruction;

pub struct Return;
pub struct Call(String);

pub struct Extern {
    name: String,
    val: Value,
}

impl Instruction for Return {
    fn new(name: &str, args: &[&str], err: &Error) -> Box<Instruction> {
        err.check_args("instruction", name, args.len(), ArgumentType::Exact(0));
        Box::new(Return)
    }

    fn exec(&self, garden: &mut Garden, _: &Tater) {
        garden.ret();
    }
}

impl Instruction for Call {
    fn new(name: &str, args: &[&str], err: &Error) -> Box<Instruction> {
        err.check_args("instruction", name, args.len(), ArgumentType::Exact(1));

        Box::new(Call(args[0].to_string()))
    }

    fn exec(&self, garden: &mut Garden, tater: &Tater) {
        garden.call(tater, self.0.as_ref());
    }
}

impl Instruction for Extern {
    fn new(name: &str, args: &[&str], err: &Error) -> Box<Instruction> {
        err.check_args("instruction", name, args.len(), ArgumentType::Range(1, 2));

        let value = match args.len() == 2 {
            true  => Value::new(args[1], err, false),
            false => Value::Bignum(gmp::mpz::Mpz::one())
        };

        Box::new(Extern {
            name: args[0].to_string(),
            val: value,
        })
    }

    fn exec(&self, garden: &mut Garden, tater: &Tater) {
        let call = tater.ext_calls.get(&self.name);
        match call {
            Some(f) => f(&self.val, garden, tater),
            None => panic!("Invalid external call of name '{}'!", self.name)
        }
    }
}
