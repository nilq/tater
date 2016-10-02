use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

use tater::tater::Tater;
use tater::garden::Garden;

use std::collections::HashMap;

mod tater;

fn load_file(tater: &mut Tater, path: &str) {
    let file = File::open(path).unwrap();
    let buffer = BufReader::new(&file);

    let mut line: usize = 0usize;

    for ln in buffer.lines() {
        let l: String = ln.unwrap();

        line += 1usize;

        tater.parse_line(&l, line, Some(path.to_string()));
    }
}

enum Req {
    Yes, Maybe, No,
}

struct ArgType {
    name: String,
    short: Option<String>,
    arg: Req,
}

fn main() {
    let valid_args = vec![
        ArgType {
            name: "help".to_string(),
            short: Some("h".to_string()),
            arg: Req::No,
        },
        ArgType {
            name: "file".to_string(),
            short: Some("f".to_string()),
            arg: Req::Yes,
        },
        ArgType {
            name: "print-stack".to_string(),
            short: Some("s".to_string()),
            arg: Req::Maybe,
        },
        ArgType {
            name: "print-parsed".to_string(),
            short: Some("p".to_string()),
            arg: Req::No,
        },
    ];

    let mut args:HashMap<String, String> = HashMap::new();

    let mut current_arg_name: String = "file".to_string();
    let mut is_first: bool = true;

    for e in env::args().collect::<Vec<String>>() {
        if is_first {
            is_first = false;
            continue;
        }

        if e.len() >= 2 && &e[0 .. 2] == "--" {
            let e = &e[2 ..];
            let contains = valid_args.iter().filter(
                |a| a.name == e).count() != 0;

            if !contains {
                panic!("Invalid argument of name '{}'!", e);
            }

            current_arg_name = e.to_string();
            args.insert(current_arg_name.clone(), "".to_string());

        } else if e.len() >= 1 && &e[0 .. 1] == "-" {
            let e = &e[1 ..];
            let mut arg_name = "".to_string();

            for arg in &valid_args {
                if arg.short == Some(e.to_string()) {
                    arg_name = arg.name.clone();
                    break;
                }
            }

            if arg_name == "" {
                panic!("Invalid argument of name {}!", e);

            } else {
                current_arg_name = arg_name;
            }

            args.insert(current_arg_name.clone(), "".to_string());

        } else {
            let arg = args.get_mut(&current_arg_name).expect("No argument!");
            arg.push_str(e.as_ref());
        }
    }

    let print_parsed = args.contains_key("print_parsed");
    let print_stack  = args.contains_key("print-stack");

    let mut execute = false;

    let mut garden = Garden::new();
    let mut tater  = Tater::new(print_parsed);

    if args.contains_key("help") {
        println!("{}",
            "
            [The Tater Language]

            Usage:
                tater --file <filename>

            Options:
                --print-stack <bits> [prints stack as a sequence of bytes]
                --print-parsed       [prints each line as it's are parsed]
                --help               [display this message]
            "
        );
    } else if args.contains_key("file") {
        load_file(&mut tater, args.get("file").expect("Wtf?"));
        execute = true;
    } else {
        println!("Type 'tater --help' for help on how to use Tater!")
    }

    if execute {
        tater.run(&mut garden);

        if print_stack {
            let bits = usize::from_str(
                match args.get("print-stack").unwrap_or(&"64".to_string()).as_ref() {
                    "" => "64",
                    other => other,
                }
            ).expect("'print-stack' argument is invalid!");

            garden.print_bytes(bits);
        }
    }
}
