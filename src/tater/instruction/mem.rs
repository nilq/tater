use super::super::value::Value;
use super::super::garden::Garden;
use super::super::tater::Tater;
use super::super::error::*;

use super::Instruction;

pub struct Put(Value, Option<Value>);
pub struct Pop(Value);

pub struct Move {
    to: Value,
    from: Value,
}

impl Instruction for Put {
    fn new(name: &str, args: &[&str], err: &Error) -> Box<Instruction> {
        err.check_args("instruction", name, args.len(), ArgumentType::Range(1, 2));

        let val = match args.len() >= 2 {
            true  => Some(Value::new(args[1], err, false)),
            false => None,
        };

        Box::new(Put(Value::new(args[0], err, false), val))
    }

    fn exec(&self, garden: &mut Garden, _: &Tater) {
        let size = self.0.get_usize(garden);
        let pos = garden.stack_len();

        garden.push(size, false);

        match self.1 {
            Some(ref val) => {
                let num = val.get_bignum(garden);
                garden.set_bits_bignum(&num, pos, size);
            },
            None => {}
        }
    }
}

impl Instruction for Pop {
    fn new(name: &str, args: &[&str], err: &Error) -> Box<Instruction> {
        err.check_args("instruction", name, args.len(), ArgumentType::Exact(1));

        Box::new(Pop(Value::new(args[0], err, false)))
    }

    fn exec(&self, garden: &mut Garden, _: &Tater) {
        let size = self.0.get_usize(garden);
        garden.pop(size);
    }
}

impl Instruction for Move {
    fn new(name: &str, args: &[&str], err: &Error) -> Box<Instruction> {
        err.check_args("instruction", name, args.len(), ArgumentType::Exact(2));

        Box::new(Move {
            to: Value::new(args[0], err, true),
            from: Value::new(args[1], err, false),
        })
    }

    fn exec(&self, garden: &mut Garden, _: &Tater) {
        if !self.from.can_coerce(self.to.get_size(garden), garden) {
            panic!("Argument bigger than assignment!");
        }

        let pos = self.to.get_ptr_position(&garden);
        let val = self.from.get_bignum(&garden);
        let size = self.to.get_size(garden);

        garden.set_bits_bignum(&val, pos, size);
    }
}
