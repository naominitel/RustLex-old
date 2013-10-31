use action::Action;
use std::hashmap::HashMap;
use std::rt::io::Writer;

struct Lexer {
    auto: ~::dfa::DFA,
    priv actions: ~HashMap<uint, ~Action>
}

impl Lexer {
    fn print_trans_table(table: &[~[uint]], out: &mut Writer) {
        let mut st = 0u;

        writeln!(out, "static transition_table: [[uint, ..256], ..{:u}] = [",
                table.len());

        for tb in table.iter() {
            if st != 0 {
                writeln!(out, ", ");
            }

            writeln!(out, "/* State {:u} */", st);
            print("[ ");

            let mut it = tb.iter();
            match it.next() {
                Some(d) => write!(out, "{:u}", *d),
                None => {
                    write!(out, "/* empty */ ]");
                    st += 1;
                    continue
                }
            }

            let mut count = 1;
            for dst in it {
                write!(out, ", ");

                if count == 16 {
                    write!(out, "\n  ");
                    count = 0;
                }

                write!(out, "{:u}", *dst);
                count += 1;
            }

            write!(out, " ]");
            st += 1;
        }

        writeln!(out, "\n];");
    }

    pub fn new(regex: ~[(~str, ~str)]) -> Lexer {
        let id = &mut 0u;
        let mut asts = ~[];
        let mut acts = ~HashMap::new();

        // parse regexs and actions 
        for (reg, act) in regex.move_iter() {
            let ast = unsafe { ::regex::parse(reg.to_c_str().unwrap()) };
            let action = Action::new(reg, act);

            *id += 1;
            acts.insert(*id, action);
            asts.push((ast, *id));
        }

        let nfa = ::nfa::NFA::build_nfa(asts);
        let mut dfa = ::dfa::DFA::new_from_nfa(nfa);
        let dfa = dfa.minimize();

        Lexer { auto: dfa, actions: acts }
    }

    pub fn write(&self, templ: Option<~str>, out: &mut Writer) {
        use automata::AutomataState;
        use std::rt::io::file;
        use std::rt::io::Reader;
        use std::rt::io::Seek;
        use std::rt::io;

        let templ_fname = match templ {
            Some(s) => s,
            None => "src/rustlex/simul.rst".into_owned()
        };

        let pth = Path::new(templ_fname);
        let mut inp = match file::open(&pth, io::Open, io::Read) {
            Some(s) => s,
            None => fail!("Unable to open template file")
        };

        // get file size
        inp.seek(0, io::SeekEnd);
        let size = inp.tell();
        inp.seek(0, io::SeekSet);

        let mut buf = ::std::vec::from_elem(size as uint, 0 as u8);
        inp.read(buf);

        let contents = ::std::str::from_utf8(buf);
        let (trans_tb, finals_tb, init_st) = self.auto.transition_table();

        for line in contents.line_iter() {
            if line == "#RUSTLEX_TRANSITION_TABLE" {
                Lexer::print_trans_table(trans_tb, out);
            }

            else if line == "#RUSTLEX_ACCEPTING_TABLE" {
                writeln!(out, "static accepting: [uint, .. {:u}] =\n[ ",
                    finals_tb.len());

                let mut it = finals_tb.iter();
                match it.next() {
                    Some(st) => write!(out, "{:u}", *st),
                    None => {
                        writeln!(out, "/* empty */ ]");
                        continue
                    }
                }

                let mut count = 1;
                for st in it {
                    write!(out, ", ");

                    if count == 16 {
                        writeln!(out, "\n  ");
                        count = 0;
                    }

                    write!(out, "{:u}", *st);
                    count += 1;
                }

                writeln!(out, " ];");
            }

            else if line == "#RUSTLEX_STATE_ACTIONS" {
                for (i, action) in self.actions.iter() { 
                    writeln!(out, "            {:u} => \\{\
                            \n                 {:s}\n            \\}",
                            *i, action.action);
                }
            }

            else if line == "#RUSTLEX_INIT_STATE" {
                writeln!(out, "static INIT_STATE: uint = {:u};", init_st);
            }

            else {
                writeln!(out, "{:s}", line);
            }
        } 
    }
}
