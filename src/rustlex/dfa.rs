use std::to_bytes::Cb;
use std::hashmap::HashMap;
use std::hashmap::HashSet;
use std::sys;
use regex::Position;
use nfa;

struct DFA {
    states: ~HashMap<uint, ~State>,
    finals: ~HashSet<uint>,
    initial: uint
}

struct State {
    nfa_states: ~HashSet<uint>,
    trans: ~HashMap<u8, uint>,
    final: bool
}

fn gen_state_num(current_id: &mut uint) -> uint {
    *current_id += 1;
    *current_id
}

fn new_state(st: &HashSet<uint>, current_id: &mut uint) -> (uint, ~State) {
    let id = gen_state_num(current_id);
    let mut ret = ~State { 
        nfa_states: ~HashSet::new(), 
        trans: ~HashMap::new(),
        final: false
    };

    for i in st.iter() {
        ret.nfa_states.insert(*i);
    }

    (id, ret)
}

fn get_next(st: &HashSet<uint>) -> uint {
    *st.iter().next().unwrap()
}

impl DFA {
   pub fn new_from_nfa(nfa: &nfa::NFA) -> ~DFA {
        let mut ret = ~DFA {
            states: ~HashMap::new(),
            finals: ~HashSet::new(),
            initial: 1
        };
        let mut id = 0;
        let current_id = &mut 0;

        let mut st = ~HashSet::new();
        st.insert(nfa.initial);
        let (f_id, fstate) = new_state(nfa.eclosure(st), current_id);
        ret.states.insert(f_id, fstate);

        let mut unmarked = ~[];
        unmarked.push(f_id);

        while !unmarked.is_empty() {
            let next_id = unmarked.pop();

           // println!("Marking state {:u}", next_id);

            for i in range(0, 255 as u8) {
                let mut trans = ~HashMap::new();
                let mut newstates = ~HashMap::new();

                match ret.states.find(&next_id) {
                    Some(st) => {
                        let tr = nfa.trans(st.nfa_states, i);

                        if tr.is_empty() {
                            // this state has no i-transitions
                            continue;
                        }

                        let eclos = nfa.eclosure(tr);

                        let mut id = None;
                        for (s, st) in ret.states.iter() {
                            if st.nfa_states == eclos {
                                id = Some(s);
                                break;
                            }
                        }

                        let _id = match id {
                            Some(i) => *i,
                            None => {
                                let (nid, nst) = new_state(eclos, current_id);
                                let mut nst = nst;

                                if nfa.is_final(eclos) {
                                    nst.final = true;
                                }

                                newstates.insert(nid, nst);
                                nid
                            }
                        };

                        trans.insert(i, _id);
                    }
                
                    None => fail!("")
                }
                
                for (i, p) in newstates.move_iter() {
                    if p.final {
                        ret.finals.insert(i);
                    }
                    ret.states.insert(i, p);
                    unmarked.push(i);
                }

                match ret.states.find_mut(&next_id) {
                    Some(st) => {
                        for (i, p) in trans.move_iter() {
                            st.trans.insert(i, p);
                        }
                    }
                    None => ()
                }
            }


        }               

        ~*ret
    }

    pub fn to_dot(&self) {
        println("digraph deterministic {");
        println!("\trankdir = LR;");
        println!("\tsize= \"4,4\";");
        println!("\tnode [shape = box]; {:u};", self.initial);
        print!("\tnode [shape = doublecircle];");

        for i in self.finals.iter() {
            print!(" {:u}", *i);
        }

        println!(";\n\tnode [shape = circle]");

        for (i, st) in self.states.iter() {
            for (c, dst) in st.trans.iter() {
                println!("\t{:u} -> {:u} [label=\"{:c}\"];", 
                        *i, *dst, *c as char);
            }
        }

        println("}");
    }

}

