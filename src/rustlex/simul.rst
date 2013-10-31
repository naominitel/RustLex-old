#RUSTLEX_TRANSITION_TABLE
#RUSTLEX_ACCEPTING_TABLE
#RUSTLEX_INIT_STATE

static INPUT_BUFSIZE: uint = 256;

struct InputBuffer {
    buf: ~[u8],
    current_pos: uint
}

struct Lexer {
    stream: ~::std::rt::io::Reader,
    inp: ~InputBuffer,
}

impl Lexer {
    fn next_input(&mut self) -> Option<u8> {
        if self.inp.current_pos == self.inp.buf.len() {
            // more input
            println("growing buffer");
            self.inp.buf = ::std::vec::from_elem(INPUT_BUFSIZE, 0 as u8);
            match self.stream.read(self.inp.buf.mut_slice_from(0)) {
                None => { println!("none"); return None } 
                Some(b) => if b < INPUT_BUFSIZE {
                    println!("Read {:u} bytes", b);
                    self.inp.buf.truncate(b); 
                }
            }

            println!("The buffer is now {:u} long", self.inp.buf.len());

            self.inp.current_pos = 0;
        }

        let ret = self.inp.buf[self.inp.current_pos];
        self.inp.current_pos += 1;
        Some(ret)
    }

    fn go_back(&mut self, pos: uint) {
        self.inp.current_pos = pos;
    }

    fn next<'a>(&'a mut self) -> Option<(uint, &'a [u8])> {
        let oldpos = self.inp.current_pos;
        let mut advance = self.inp.current_pos;
        let mut last_matching_action = 0;
        let mut current_st = INIT_STATE;

        while current_st != 0 {
            let i = match self.next_input() {
                Some(i) => i,
                None => return None
            };

            let new_st = transition_table[current_st][i];
            let action = accepting[new_st];

            if action != 0 {
                advance = self.inp.current_pos;

                // final state
                last_matching_action = action;
            }

            current_st = new_st;
        }

        // go back to last matching state in the input
        self.go_back(advance);

        // execute action corresponding to found state
        match last_matching_action {
#RUSTLEX_STATE_ACTIONS
            _ => {
                // default action is printing on stdout
                self.go_back(oldpos + 1);
                let s = self.inp.buf.slice(oldpos, self.inp.current_pos);
                print!("{:s}", ::std::str::from_utf8(s));
            }
        }
    
        // if the user code did not return, continue
        self.next()
    }

    fn new(stream: ~::std::rt::io::Reader) -> ~Lexer {
        let buf = ~InputBuffer { buf: ~[], current_pos: 0 };
        ~Lexer { stream: stream, inp: buf }
    }
}

fn main() {
    let pth = Path::new("input");
    let inp = ~::std::rt::io::file::open(&pth, ::std::rt::io::Open, ::std::rt::io::Read).unwrap() as ~::std::rt::io::Reader;
    let mut lexer = Lexer::new(inp);

    for (_, s) in lexer {
        println!("matched string: {:s}", ::std::str::from_utf8(s));
    }
}
