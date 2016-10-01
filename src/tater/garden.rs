extern crate gmp;
extern crate time;

use super::util::*;
use super::tater::Tater;

use std::mem::size_of;

pub struct Garden {
    stack: Vec<bool>,
    call_stack: Vec<usize>,

    pub instruction: usize,
    pub validity: bool,
    pub input_string: String,
    pub rand_state: gmp::rand::Randstate,
}

#[allow(dead_code)]
impl Garden {
    pub fn new() -> Garden {
        let mut ret = Garden {
            stack: Vec::new(),
            call_stack: Vec::new(),
            instruction: 0,
            validity: true,
            input_string: "".to_string(),
            rand_state: gmp::rand::RandState::new(),
        };
        ret.rand_state.seed_ui(time::get_time().sec as u64);
        ret
    }

    pub fn stack_len(&self) -> usize {
        self.stack.len()
    }

    pub fn push(&mut self, bits: usize, value: bool) {
        let len = self.stack.len();
        self.stack.resize(len + bits, value);
    }

    pub fn pop(&mut self, bits: usize) {
        let len = self.stack.len();
        self.stack.truncate(len - bits);
    }

    pub fn slice(&self, start: usize, end: usize) -> &[bool] {
        &self.stack[start .. end]
    }

    pub fn print_bytes(&self, bits_per_byte: usize) {
        for i in 0 .. (self.stack_len() / bits_per_byte) {
            let bits = self.slice(i * bits_per_byte, (i + 1) * bits_per_byte);
            let num = boolvec_to_bignum(bits);

            print!("{}, ", num);
        }
        println!("");
    }

    pub fn set_bits_boolvec(&mut self, num: &[bool], pos: usize, len: usize) {
        for i in 0 .. len {
            self.stack[pos + i] = match i < size_of::<usize>() * 8 {
                true => num & (1 << i) != 0,
                false => false,
            }
        }
    }

    pub fn set_bits_bignum(&mut self, num: &gmp::mpz::Mpz, pos: usize, len: usize) {
        self.set_bits_boolvec(&bignum_to_boolvec(num), pos, len);
    }

    pub call(&mut self, tater: &Tater, name: &str) {
        self.call_stack.push(self.instruction);
        self.instruction = *tater.labels.get(name).expect(
            format!("No such label of name {}!", name).as_ref()
        );
    }

    pub fn ret(&mut self) {
        let pos = self.call_stack.pop().expect("Attempt to return on empty 'call_stack'!")
        self.instruction = pos;
    }

    pub fn goto(&mut self, tater: &Tater, name: &str) {
        self.instruction = *tater.labels.get(name).expect(
            format!("No such label of name {}!", name).as_ref()
        );
    }
}
