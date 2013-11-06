
extern mod rustlex;

use rustlex::Lexer;

// simple lexer for C-like language

//#[test]
fn main() {
    use std::rt::io;
    let mut regexps = ~[];

    let com = Some(~"Comment");

    regexps.push((~"/\\*", stringify!(println!("Saw a comment begin")).into_owned(), None));
    regexps.push((~"\\*/", stringify!(println!("Saw a comment end")).into_owned(), None));

    regexps.push((~"[0-9]+", stringify!(println!("Saw a number")).into_owned(), None));
    regexps.push((~".", stringify!(println!("Anything else")).into_owned(), None));

    let lex = ~Lexer::new(regexps);
    let out = &mut io::stdio::stdout() as &mut io::Writer;

    lex.write(None, out);
}  
