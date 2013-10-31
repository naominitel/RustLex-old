use std::hashmap::HashMapIterator;
use std::hashmap::HashSetIterator;
use std::rt::io::Writer;

pub trait AutomataState {
    fn transitions(&self) -> ~[(u8, uint)];
    fn etransitions<'a>(&'a self) -> HashSetIterator<'a, uint>;
    fn is_final(&self) -> bool;
    fn action(&self) -> Option<uint>;
}

pub trait Automata<State: AutomataState> {
    fn states_iter<'a>(&'a self) -> HashMapIterator<'a, uint, ~State>;
    fn finals_iter<'a>(&'a self) -> HashSetIterator<'a, uint>;
    fn find_state<'a>(&'a self, k: uint) -> Option<&'a ~State>;
    fn initial(&self) -> uint;
    fn label(&self, state: uint) -> Option<~str>;
}

// generate a graphviz dot-representation of the automata
// writes it to the `out` output stream
pub fn to_dot<T: AutomataState, U: Automata<T>>(au: &U, out: &mut Writer) {
    writeln!(out, "digraph automata \\{");
    writeln!(out, "\trankdir = LR;");
    writeln!(out, "\tsize= \"4,4\";");
    writeln!(out, "\tnode [shape = box]; {:u};", au.initial());
    writeln!(out, "\tnode [shape = doublecircle];");
    write!(out, "\t");

    for i in au.finals_iter() {
        write!(out, "{:u} ", *i);
    }

    writeln!(out, ";\n\tnode [shape = circle]");

    for (i, st) in au.states_iter() {
        let trans = st.transitions();
        for &(c, dst) in trans.iter() {
            writeln!(out, "\t{:u} -> {:u} [label=\"{:c}\"];", 
                    *i, dst, c as char);
        }

        for dst in st.etransitions() {
            writeln!(out, "\t{:u} -> {:u} [label=\"e\"];", *i, *dst);
        }
    }

    for i in au.finals_iter() {
        match au.label(*i) {
            Some(st) => writeln!(out, "\t{:u} [label=\"{:s}\"];", *i, st),
            None => ()
        }
    }

    writeln!(out, "\\}");
}

