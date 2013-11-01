#[warn(non_uppercase_statics)];
#[warn(non_camel_case_types)];
#[warn(unnecessary_qualification)];
#[link(name = "rustlex", vers = "0.1")];

extern mod extra;
pub use lexer::Lexer;

mod action;
mod automata;
mod dfa;
pub mod lexer;
mod nfa;
mod regex;

fn print_ast(a: &regex::AST, prefix: &str) {
    match *a {
        regex::Or(ref opl, ref opr) => {
            println!("{:s}(Or of: ", prefix);
            print_ast(&**opl, prefix);
            print_ast(&**opr, prefix);
            println!(")");
        }

        regex::Cat(ref opl, ref opr) => {
            println!("{:s}(Concatenation of:", prefix);
            print_ast(&**opl, prefix);
            print_ast(&**opr, prefix);
            println!(")");
        }

        regex::Clos(ref op) => {
            println!("{:s}(Closure of:", prefix);
            print_ast(&**op, prefix);
            println!(")");
        }

        regex::Char((c, p)) => {
            println!("{:s}(Just the char {:c} as pos {:u})", prefix, c, p);
        }
    }
}

