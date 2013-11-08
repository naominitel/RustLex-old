extern mod rustlex;

// simple lexer for C-like language
//#[test]
fn main() {
    use rustlex::Lexer;
    use std::rt::io;

    let mut regexps = ~[];

    regexps.push((~"/\\*", stringify!({
        println!("Saw a comment begin");
        self.condition = Comment;
    }).into_owned(), None));

    regexps.push((~"\\*/", stringify!({
        println!("Saw a comment end");
        self.condition = Initial;
    }).into_owned(), Some(~"Comment")));

    regexps.push((~"[0-9]+", stringify!({
        println!("Saw a number");
    }).into_owned(), None));

    regexps.push((~".", stringify!({
        println!("Anything else");
    }).into_owned(), None));

    regexps.push((~".", stringify!({
        println!("Anything in a comment");
    }).into_owned(), Some(~"Comment")));

    let lex = ~Lexer::new(regexps);
    let out = &mut io::stdio::stdout() as &mut io::Writer;

    lex.write(None, out);
}
