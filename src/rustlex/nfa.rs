use automata::Automata;
use automata::AutomataState;
use std::hashmap::HashMap;
use std::hashmap::HashMapIterator;
use std::hashmap::HashSet;
use std::hashmap::HashSetIterator;
use regex;

// a non-deterministic finite automata
struct NFA {
    priv states: ~HashMap<uint, ~State>,
    priv finals: ~HashSet<uint>,
    priv initial: uint
}

struct State {
    priv trans: ~HashMap<u8, ~HashSet<uint>>,
    priv etrans: ~HashSet<uint>,
    priv action: Option<uint>
}

impl AutomataState for State {
    fn transitions(&self) -> ~[(u8, uint)] {
        let mut transitions = ~[];

        for (ch, dsts) in self.trans.iter() {
            for st in dsts.iter() {
                transitions.push((*ch, *st));
            }
        }

        transitions
    }

    fn etransitions<'a>(&'a self) -> HashSetIterator<'a, uint> {
        self.etrans.iter()
    }

    fn is_final(&self) -> bool {
        self.action != None
    }

    fn action(&self) -> Option<uint> {
        self.action
    }
}

impl State {
    pub fn trans<'a>(&'a self, c: u8) -> Option<HashSetIterator<'a, uint>> {
        do self.trans.find(&c).map |s| { s.iter() }
    }
}

impl Automata<State> for NFA {
    fn label(&self, st: uint) -> Option<~str> {
        Some(format!("{:u}", st))
    }

    fn states_iter<'a>(&'a self) -> HashMapIterator<'a, uint, ~State> {
        self.states.iter()
    }

    fn finals_iter<'a>(&'a self) -> HashSetIterator<'a, uint> {
        self.finals.iter()
    }
    
    fn find_state<'a>(&'a self, st: uint) -> Option<&'a ~State> {
        self.states.find(&st)
    }
    
    fn initial(&self) -> uint {
        self.initial
    }
}

fn gen_state_num(current_id: &mut uint) -> uint {
    *current_id += 1;
    *current_id
}

fn new_state(current_id: &mut uint) -> (uint, ~State) {
    let id = gen_state_num(current_id);
    (id, ~State { 
        trans: ~HashMap::new(),
        etrans: ~HashSet::new(),
        action: None
    })
}

fn final_state(st: &NFA) -> uint {
    match st.finals.iter().next() {
        Some(f) => *f,
            None => fail!("")
    }
}

impl NFA {
    pub fn new(r: &regex::AST, current_id: &mut uint) -> ~NFA {
        match *r {
            regex::Or(ref opl, ref opr) => {
                let mut ret = ~NFA { 
                    states: ~HashMap::new(),
                    finals: ~HashSet::new(),
                    initial: 0
                };

                let nfa_l = NFA::new(&**opl, current_id); 
                let nfa_r = NFA::new(&**opr, current_id);

                // consumes sub-NFAs by moving out their contents
                let ~NFA { initial: init_l, states: st_l, finals: finals_l } = nfa_l;
                let ~NFA { initial: init_r, states: st_r, finals: finals_r } = nfa_r;

                // create new final and initial states
                let (first_id, nfirst) = new_state(current_id);
                let (final_id, nfinal) = new_state(current_id);
                let mut nfirst = nfirst;

                // transfer ownership of states from the build sub-NFA
                // to the new NFA, but keeping state numbers 

                for (i, s) in st_l.move_iter() {
                    ret.states.swap(i, s);
                }

                for (i, s) in st_r.move_iter() {
                    ret.states.swap(i, s);
                }
                     
                // set up the new initial state
                nfirst.etrans.insert(init_l);
                nfirst.etrans.insert(init_r);
                ret.initial = first_id;

                // set up the new final state
                for i in finals_l.iter() {
                    match ret.states.find_mut(i) {
                        Some(state) => {
                            state.action = None;
                            state.etrans.insert(final_id);
                        }

                        // maybe should this fail!() ?
                        None => ()
                    }
                }

                for i in finals_r.iter() {
                    match ret.states.find_mut(i) {
                        Some(state) => {
                            state.action = None;
                            state.etrans.insert(final_id);
                        }

                        // maybe should this fail!() ?
                        None => ()
                    }
                }

                ret.finals.insert(final_id);
                ret.states.insert(first_id, nfirst);
                ret.states.insert(final_id, nfinal);

                ret
            }

            regex::Cat(ref opl, ref opr) => {
                let mut ret = NFA::new(&**opl, current_id);
                let ~NFA {
                    initial: init,
                    states: states,
                    finals: finals
                } = NFA::new(&**opr, current_id);
                let mut states = states;

                // merge the final state of the left part with
                // the first state of the right part

                // the left part is supposed to have a single final state
                let final = final_state(ret);

                // remove the old first state
                let ~State { trans: ftrans, etrans: fetrans, action: _ } = 
                    match states.pop(&init) {
                        Some(st) => st,
                        None => fail!("")
                    };

                // update the old final to have the transitions of the
                // old initial
                match ret.states.find_mut(&final) {
                    Some(fstate) => {
                        fstate.trans = ftrans;
                        fstate.etrans = fetrans;
                    }

                    None => fail!("")
                }

                // move all other states into new NFA
                for (i, st) in states.move_iter() {
                    ret.states.insert(i, st);
                }

                // mark the old final of the right part as final
                // and unmark the final of the left part
                let oldfinal = match finals.iter().next() {
                    Some(f) => *f,
                        None => fail!("")
                };

                ret.finals.clear();
                ret.finals.insert(oldfinal);
                ret
            }

            regex::Char((ch, _)) => {
                let mut ret = ~NFA {
                    states: ~HashMap::new(),
                    finals: ~HashSet::new(),
                    initial: 0
                };

                let (init_id, init) = new_state(current_id);
                let (final_id, final) = new_state(current_id);
                let mut init = init;

                let mut dst = ~HashSet::new();
                dst.insert(final_id);
                init.trans.insert(ch as u8, dst);

                ret.states.insert(init_id, init);
                ret.states.insert(final_id, final);

                ret.initial = init_id;
                ret.finals.insert(final_id);

                ret
            }

            regex::Clos(ref op) => {
                let mut ret = NFA::new(&**op, current_id);

                let (first_id, nfirst) = new_state(current_id);
                let (final_id, nfinal) = new_state(current_id);
                let mut nfirst = nfirst;

                let final = final_state(ret);
                match ret.states.find_mut(&final) {
                    Some(st) => {
                        st.etrans.insert(final_id);
                        st.etrans.insert(ret.initial);
                    }
                    None => fail!("")
                }

                nfirst.etrans.insert(ret.initial);
                nfirst.etrans.insert(final_id);
                ret.states.insert(first_id, nfirst);
                ret.states.insert(final_id, nfinal);
                ret.initial = first_id;
                ret.finals.clear();
                ret.finals.insert(final_id);

                ret
            }
        }
    }

    pub fn build_nfa(regexs: ~[(~regex::AST, uint)]) -> ~NFA {
        let mut id = 0;
        let mut ret = ~NFA {
            states: ~HashMap::new(),
            finals: ~HashSet::new(),
            initial: 0
        };

        let mut first = ~State {
            trans: ~HashMap::new(),
            etrans: ~HashSet::new(),
            action: None
        };

        for (reg, act) in regexs.move_iter() {
            let ~NFA {
                states: nstates,
                finals: nfinals,
                initial: ninit
            } = NFA::new(reg, &mut id);

            for (i, p) in nstates.move_iter() {
                ret.states.insert(i, p);
            }

            for p in nfinals.move_iter() {
                ret.finals.insert(p);

                let st = ret.states.find_mut(&p).unwrap();
                st.action = Some(act);
            }

            first.etrans.insert(ninit);
        }

        let n_id = gen_state_num(&mut id);
        ret.initial = n_id;
        ret.states.insert(n_id, first);

        ret
    }

    pub fn to_dot(&self) {
        println("digraph automata {");
        println!("\trankdir = LR;");
        println!("\tsize = \"4,4\";");
        println!("\tnode [shape = box]; {:u};", self.initial);
        print!("\tnode [shape = doublecircle];");

        for i in self.finals.iter() {
            print!(" {:u}", *i);
        }

        println!(";\n\tnode [shape = circle]");

        for (i, st) in self.states.iter() {
            for (c, dst) in st.trans.iter() {
                for d in dst.iter() {
                    println!("\t{:u} -> {:u} [label=\"{:c}\"];", 
                        *i, *d, *c as char);
                }
            }

            for dst in st.etrans.iter() {
                println!("\t{:u} -> {:u} [label=\"e\"];", *i, *dst);
            }
        }
        println("}");
    }
    
    pub fn eclosure(&self, st: &HashSet<uint>) -> ~HashSet<uint> {
        let mut ret = ~st.clone();
        let mut stack = ~[];
        
        for s in st.iter() {
            stack.push(*s);
        }

        while !stack.is_empty() {
            let st = stack.pop();

            match self.states.find(&st) {
                Some(state) => {
                    for i in state.etrans.iter() {
                        if !ret.contains(i) {
                            ret.insert(*i);
                            stack.push(*i);
                        }
                    }
                }
                None => ()
            }
        }

        ret
    }

    pub fn eclosure_(&self, st: uint) -> ~HashSet<uint> {
        let mut hs = ~HashSet::new();
        hs.insert(st);
        self.eclosure(hs)
    }

    pub fn is_final(&self, st: &HashSet<uint>) -> bool {
        for i in st.iter() {
            if self.finals.contains(i) {
                return true;
            }
        }

        false
    }
}

