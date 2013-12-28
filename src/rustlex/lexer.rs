use action::Action;
use automata::Automata;
use std::hashmap::HashMap;
use std::io::Writer;
use trans_table::transition_table;

pub struct Lexer {
    priv auto: ~[~::dfa::DFA],
    priv actions: ~HashMap<uint, ~Action>,
    priv conditions: ~HashMap<~str, uint>
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
            write!(out, "[ ");

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

    pub fn new(regex: ~[(~str, ~str, Option<~str>)]) -> Lexer {
        let id = &mut 0u;
        let mut asts: ~HashMap<~str, ~[(~::regex::AST, uint)]> = ~HashMap::new();
        let mut acts = ~HashMap::new();

        // parse regexs and actions 
        for (reg, act, cond) in regex.move_iter() {
            let ast = unsafe { ::regex::parse(reg.to_c_str().unwrap()) };
            let cond = match cond {
                Some(c) => c,
                None => "Initial".into_owned()
            };

            *id += 1;
            let action = Action::new(reg, act);
            acts.insert(*id, action);

            match asts.find_mut(&cond) {
                Some(arr) => { arr.push((ast, *id)); continue }
                None => ()
            }

            asts.insert(cond, ~[(ast, *id)]);
        }

        let mut dfas = ~[];
        let mut id = 0;
        let mut conds = ~HashMap::new();

        for (cond, asts) in asts.move_iter() {
            let nfa = ::nfa::NFA::build_nfa(asts);
            let mut dfa = ::dfa::DFA::new_from_nfa(nfa, &mut id);
            let dfa = dfa.minimize();

            println!("Initial ID of automata {:s} is {:u} ({:u})", cond, dfa.initial(), id);
            conds.insert(cond, dfa.initial());
            dfas.push(dfa);
        }

        Lexer { auto: dfas, actions: acts, conditions: conds }
    }

    pub fn write(&self, templ: Option<~str>, out: &mut Writer) {
        use std::io::File;
        use std::io::Reader;
        use std::io::Seek;
        use std::io;

        let templ_fname = match templ {
            Some(s) => s,
            None => "src/rustlex/simul.rst".into_owned()
        };

        let pth = Path::new(templ_fname);
        let mut inp = match File::open(&pth) {
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
        let (trans_tb, finals_tb, new_ids) = transition_table(self.auto);

        for line in contents.lines() {
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
                    action.write(*i, out);
                }
            }

            else if line == "#RUSTLEX_CONDITIONS" {
                for (cond, init_s) in self.conditions.iter() {
                    writeln!(out, "static {:s}: uint = {:u};",
                        *cond, *new_ids.find(init_s).unwrap());
                }
            }

            else {
                writeln!(out, "{:s}", line);
            }
        } 
    }
}
