extern crate regex;
extern crate gmp;

use self::regex::Regex;

use std::str::FromStr;
use std::fmt;

use super::util::*;
use super::error::*;
use super::garden::Garden;

pub enum Value {
    Boolvec (Vec<bool>),
    Bignum (gmp::mpz::Mpz),
    Position (Box<Value>, bool),
    Pointer {
        pos: Box<Value>,
        len: Box<Value>,
        rev: bool,
    },
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
		    Value::Boolvec(ref vec) => write!(f, "v{}", boolvec_to_bignum(vec)),
			Value::Bignum(ref vec) => write!(f, "n{}", vec),
            Value::Pointer {ref pos, ref len, ref rev} => {
				let negative_str = match *rev {
                    true => "-",
                    false => "",
                };
				write!(f, "[{}{}:{}]", negative_str, pos, len)
			},
			Value::Position(ref ptr, rev) => {
				write!(f, "{}{}", match rev{
					true => '>',
					false => '<'
				}, format!("{}", ptr))
			}
		}
    }
}


#[allow(dead_code)]
impl Value {
    pub fn is_ptr(&self) -> bool {
        match *self {
            Value::Pointer{..} => true,
            _ => false
        }
    }

    pub fn get_ptr_position(&self, garden: &Garden) -> usize {
        match *self {
            Value::Pointer {
                ref pos, ref rev, ..
            } => match *rev {
                true => garden.stack_len() - pos.get_usize(garden),
                false => pos.get_usize(garden)
            },
            _ => panic!("Value, {}, is not a pointer!", self)
        }
    }

    pub fn get_usize(&self, garden: &Garden) -> usize {
        bignum_to_usize(&self.get_bignum(garden))
    }

    pub fn get_bignum(&self, garden: &Garden) -> gmp::mpz::Mpz {
        match *self {
            Value::Bignum(ref num) => num.clone(),
            Value::Boolvec(ref vec) => boolvec_to_bignum(vec),
            Value::Pointer {..} => {
                let size = self.get_ptr_size(garden);
                let pos  = self.get_ptr_position(garden);
                let bits = garden.slice(pos, pos + size);

                boolvec_to_bignum(bits)
            },
            Value::Position(ref ptr, rev) => {
                let mut pos = ptr.get_ptr_position(garden);
                if rev {
                    pos += ptr.get_ptr_size(garden);
                }
                usize_to_bignum(pos)
            }
        }
    }

    pub fn get_boolvec(&self, garden: &Garden) -> Vec<bool> {
        match *self {
            Value::Bignum(ref num) => bignum_to_boolvec(num),
            Value::Boolvec(ref vec) => vec.clone(),
            Value::Pointer {..} => {
                let size = self.get_ptr_size(garden);
                let pos  = self.get_ptr_position(garden);
                let bits = garden.slice(pos, pos + size);

                bits.to_vec()
            },
            Value::Position(ref ptr, rev) => {
                let mut pos = ptr.get_ptr_position(garden);
                if rev {
                    pos += ptr.get_ptr_size(garden);
                }
                usize_to_boolvec(pos)
            }
        }
    }

    pub fn get_ptr_size(&self, garden: &Garden) -> usize {
        match *self {
            Value::Pointer {ref len, ..} => len.get_usize(garden),
            _ => panic!("Value, {}, is not a pointer!", self)
        }
    }

    pub fn can_coerce(&self, new_size: usize, garden: &Garden) -> bool {
        match *self {
            Value::Pointer {ref len, ..} => len.get_usize(garden) <= new_size,
            Value::Boolvec(ref vec) => vec.len() <= new_size,
            Value::Bignum(ref num) => num.bit_length() <= new_size,
            Value::Position(ref ptr, rev) => {
                let mut pos = ptr.get_ptr_position(garden);
                if rev {
                    pos += ptr.get_ptr_size(garden);
                }
                usize_len(pos) <= new_size
            }
        }
    }

    pub fn get_size(&self, garden: &Garden) -> usize {
        match *self {
            Value::Pointer {ref len, ..} => len.get_usize(garden),
            Value::Boolvec(ref vec) => vec.len(),
            Value::Bignum(ref num) => num.bit_length(),
            Value::Position(ref ptr, rev) => {
                let mut pos = ptr.get_ptr_position(garden) ;
				if rev {
					pos += ptr.get_ptr_size(garden);
				}
                usize_len(pos)
            }
        }
    }

    pub fn new(value: &str, err: &Error, require_pointer: bool) -> Value {
        let ret = match Value::create(value, err) {
            Some(ret) => ret,
            None => err.throw(ErrorType::InvalidValue(value.to_string()))
        };
        if require_pointer && !ret.is_ptr() {
            err.throw(ErrorType::InvalidPointer(ret))
        };
        ret
    }

    fn create(value: &str, err: &Error) -> Option<Value> {
        let value = value.trim();

        println!("Parsing: {}", value);

        let re_ptr = Regex::new(r"^\[(.*?)\]$").unwrap();

        if re_ptr.is_match(value) {
            let mut args = ("".to_string(), "".to_string());
            let mut bracket_n = 0;

            let mut i = 0;
            let mut arg_i = 0;
            let mut from_back = false;

            for c in value.chars() {
                let skip = i < 1 || i >= value.len() - 1 || c.is_whitespace();
                i += 1;

                if skip {
                    continue;
                }

                if c == '[' {
                    bracket_n -= 1;
                } else if c == ']' {
                    bracket_n -= 1;
                }

                if c == ';' && bracket_n == 0 {
                    arg_i = 1;
                } else if c == '-' && args.0.len() == 0 && arg_i == 0 {
                    from_back = true;
                } else {
                    if arg_i == 0 {
                        args.0.push(c);
                    } else if arg_i == 1 {
                        args.1.push(c);
                    }
                }
            }

            if args.1 == "" {
                args.1 = "1".to_string();
            }

            let position = Value::create(&args.0, err);
            let length = Value::create(&args.1, err);

            return match (position, length) {
                (Some(pos_val), Some(len_val)) => {
                    Some(Value::Pointer {
                        pos: Box::new(pos_val),
                        len: Box::new(len_val),
                        rev: from_back,
                    })
                },
                _ => None
            }
        } else if value.chars().next() == Some('b') {
            let mut boolvec: Vec<bool> = vec![];
            boolvec.reserve(value.len() - 1);

            for c in value[1 ..].chars() {
                boolvec.push(match c {
                    '0' => false,
                    '1' => true,
                    other => err.throw(ErrorType::Generic(
                        format!("'{}' is invalid for boolvec!", c)
                    ))
                });
            }

            return Some(Value::Boolvec(boolvec));

        } else if value.chars().next() == Some('<') {
            return match Value::create(&value[1 ..], err) {
                Some(val) => {
                    if !val.is_ptr() {
                        err.throw(ErrorType::InvalidPointer(val));
                    }
                    Some(Value::Position(Box::new(val), false))
                }
                None => None
            }

        } else if value.chars().next() == Some('>') {
            return match Value::create(&value[1 ..], err) {
                Some(val) => {
                    if !val.is_ptr() {
                        err.throw(ErrorType::InvalidPointer(val));
                    }
                    Some(Value::Position(Box::new(val), true))
                }
                None => None
            }

        } else {
            // non-pointer value
            match gmp::mpz::Mpz::from_str(value) {
                Ok(val) => Some(Value::Bignum(val)),
                Err(_) => None,
            }
        }
    }
}
