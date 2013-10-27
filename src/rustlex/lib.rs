extern mod extra;

mod dfa;
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
    let regex = "a";
    let regex2 = "abb";
    let regex3 = "a*bb*";

    unsafe {
        let ast = regex::parse(regex.to_c_str().unwrap());
        let ast2 = regex::parse(regex2.to_c_str().unwrap());
        let ast3 = regex::parse(regex3.to_c_str().unwrap());
   //     print_ast(ast, "");

   // println!("First positions: {:?}", ast.first_pos());
   // println!("Final positions: {:?}", ast.final_pos());
        let asts = [ast, ast2, ast3];

    let nfa = nfa::NFA::build_nfa(asts);
  //  nfa.to_dot();

    let dfa = ~dfa::DFA::new_from_nfa(nfa);
    dfa.to_dot();
    }
}  
