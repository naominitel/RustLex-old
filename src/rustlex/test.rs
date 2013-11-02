extern mod rustlex;

use rustlex::Lexer;

#[test]
fn main() {
    use std::rt::io;
    let mut regexps = ~[];

    regexps.push((~"a", stringify!(println!("Saw an A")).into_owned(), None));
    regexps.push((~"abb", stringify!(println!("Saw abb")).into_owned(), None));
    regexps.push((~"a*bb*", stringify!(println!("Saw a*b+")).into_owned(), None));
    regexps.push((~"a*bb*", stringify!(println!("Saw a*b+")).into_owned(), Some("Foo".into_owned())));

    let lex = ~Lexer::new(regexps);
    let out = &mut io::stdio::stdout() as &mut io::Writer;
 //   ::automata::to_dot(lex.auto, out);

    lex.write(None, out);
}  
