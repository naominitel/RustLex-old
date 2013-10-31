#[warn(non_uppercase_statics)];
#[warn(non_camel_case_types)];
#[warn(unnecessary_qualification)];

extern mod extra;

mod action;
mod automata;
mod dfa;
mod lexer;
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

fn main() {
    use std::rt::io;
    let mut regexps = ~[];

    regexps.push((~"a", stringify!(println!("Saw an A")).into_owned()));
    regexps.push((~"abb", stringify!(println!("Saw abb")).into_owned()));
    regexps.push((~"a*bb*", stringify!(println!("Saw a*b+")).into_owned()));

    let lex = ~::lexer::Lexer::new(regexps);
    let out = &mut io::stdio::stdout() as &mut io::Writer;
 //   ::automata::to_dot(lex.auto, out);

    lex.write(None, out);
}  
