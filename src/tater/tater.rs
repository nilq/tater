extern crate gmp;

use super::value::Value;

const COMMENT_CHAR: char = '?';
const ARGUMENT_CHAR: char = ';';

use super::garden::Garden;
use super::instruction::{Instruction, create_instruction};
use super::error::*;
use super::util::*;

use std::collections::HashMap;
use std::fs::File;

use std::io;
use std::io::{BufRead, BufReader, Write};

use std::str::FromStr;

pub struct Tater {
    code: Vec<Box<Instruction>>,
    defines: Vec<(String, String)>,

    pub labels: HashMap<String, usize>,
    pub ext_calls: HashMap<String, Box<Fn(&Value, &mut Garden, &Tater)>>,
    pub print_parsed: bool,
}

impl Tater {
    fn parse_strings(&self, line: &String) -> String {
        let mut new_line: String = String::new();

        let mut is_literal       = false;
        let mut is_in_string     = false;

        for c in line.chars() {
            if !is_in_string && c == '"' {
                is_in_string = true;
                is_literal = false;

                new_line.push('b');

                continue;
            }

            if is_in_string {
                if c == '\\' && !is_literal {
                    is_literal = true;
                    continue;
                }

                if c == '"' && !is_literal {
                    is_in_string = false;
                    continue;
                }

                let mut char_add: char = c;
                if is_literal {
                    char_add = match c {
                        '\\'  => '\\',
                        'n'   => '\n',
                        't'   => '\t',
                        other => other
                    };

                    is_literal = false;
                }

                for b in char_to_boolvec(char_add) {
                    new_line.push(match b {
                        true  => '1',
                        false => '0'
                    })
                }

            } else {
                new_line.push(c);
            }
        }

        return new_line;
    }

    pub fn add_external_call<F>(&mut self, name: &str, external: F)
            where F: 'static + Fn(&Value, &mut Garden, &Tater) {
        self.ext_calls.insert(name.to_string(), Box::new(external));
    }

    fn add_default_external_calls(tater: &mut Tater) {
        tater.add_external_call("numprint", |v, e, _| {
            print!("{}", boolvec_to_bignum(v.get_boolvec(e).as_slice()));
            io::stdout().flush().ok().expect("Wasn't able to flush std:out!");
        });

        tater.add_external_call("print", |v, e, _| {
            let mut chars: Vec<u8> = vec![];

            let boolvec = v.get_boolvec(e);
            let value = boolvec.as_slice();

            for i in 0 .. (value.len() / 8) {
                let nums = &value[i * 8 .. (i + 1) * 8];
                let c = boolvec_to_u8(nums);

                if c == 0 {
                    break;
                }

                chars.push(c);
            }

            let s = String::from_utf8_lossy(chars.as_slice()).to_string();

            print!("{}", s);
            io::stdout().flush().ok().expect("Wasn't able to flush std:out!");
        });
    }



    fn parse_defines(&self, l: &String) -> String {
        let mut sep_chars:Vec<char> = Vec::new();
		let strings:Vec<&str> = l.split(|c:char| {
			if c.is_whitespace() {
				sep_chars.push(c);
				return true;
			}
			match c {
				',' | '[' | ']' | ':' | '-' | '<' | '>' => {
					sep_chars.push(c);
					true
				},
				_ => false
			}
		}).collect::<Vec<&str>>();

		sep_chars.reverse();

		strings.iter().map(|w| {
			let mut ret = String::new();
			let mut did = false;

			for def in &self.defines {
				if def.0 == *w {
					ret.push_str(&format!("{}", def.1));
					did = true;
					break;
				}
			}

			if !did {
				ret.push_str(&format!("{}", w));
			}

			match sep_chars.pop() {
				Some(c) => ret.push(c),
				None => {}
			};

			ret
        }).collect::<String>()
    }

    pub fn new(print_parsed: bool) -> Tater {
        let mut tater = Tater {
            code: Vec::new(),
            defines: Vec::new(),
            labels: HashMap::new(),
            ext_calls: HashMap::new(),
            print_parsed: print_parsed
        };

        Tater::add_default_external_calls(&mut tater);
        tater
    }

    fn parse_args(&mut self, iname: &String, arguments: &[&str], err: &Error){
		let ins = create_instruction(iname.as_ref(), arguments, self, err);
		self.code.push(ins);
    }

    fn parse_labels(&mut self, line: &String) -> bool {
        if line.chars().next() == Some('@') {
            let name = line[1 ..].to_string();
            self.labels.insert(name, self.code.len());
            return true
        }
        false
    }

    fn parse_macros(&mut self, l: &String, err: &Error) -> bool {
        if l.chars().next() == Some('#') {
            let macro_text = l[1 ..].trim();
            let mut macro_args: Vec<&str> = macro_text.split_whitespace().collect();

            if macro_args.len() < 1 {
                err.throw(ErrorType::Empty("macro".to_string()))
            }

            let macro_name = macro_args[0];
            let macro_total_args: &str = &macro_text[macro_name.len() ..].trim();

            macro_args.remove(0);

            match macro_name {
                "define" => {
                    err.check_args("macro", macro_name, macro_args.len(), ArgumentType::AtLeast(2));

                    let name = macro_args[0].to_string();
                    let args = macro_args[1 ..].join(" ");

                    self.defines.push((name, args));
                },
                "require" => {
                    err.check_args("macro", macro_name, macro_args.len(), ArgumentType::AtLeast(1));

                    let file = File::open(macro_total_args).unwrap();
                    let buffer = BufReader::new(&file);
                    let mut line = 0;
                    for l2 in buffer.lines() {
                        line += 1;
                        let l1: String = l2.unwrap();
                        self.parse_line(&l1, line, Some(macro_total_args.to_string()));
                    }
                },
                name => err.throw(ErrorType::NonExistent {
                    type_name: "macro".to_string(),
                    value: name.to_string()
                })
            }
            return true
        }
        false
    }

    pub fn parse_line(&mut self, line_arg: &String, line: usize, path: Option<String>) {
        let mut l: String = line_arg.to_string();
        remove_comments(&mut l, COMMENT_CHAR);

        l = l.trim().to_string();

        let err = Error::new(l.clone(), line, path);

        l = self.parse_strings(&l);

        if self.parse_macros(&l, &err) {
            return;
        }

        if self.parse_labels(&l) {
            return;
        }

        l = self.parse_defines(&l);

        let mut arg_string: String = String::new();

        let name: String = match l.split_whitespace().next() {
            Some(val) => {val.to_string()},
            None => return,
        };

        match l.find(name.as_str()) {
            Some(p) => {
                arg_string = l[(p + name.len()) ..].trim().to_string();
            }
            None => {},
        }

        let arg_vec: Vec<&str> = arg_string
            .split(ARGUMENT_CHAR)
            .map(|val| val.trim())
            .filter(|val| val.trim() != "")
            .collect();

        self.parse_args(&name, &arg_vec, &err);
    }

    pub fn run(&mut self, garden: &mut Garden) {
        let len = self.code.len();
        while garden.instruction < len {
            let ins = &self.code[garden.instruction];
            garden.instruction += 1;

            ins.exec(garden, self);
        }
    }
}
