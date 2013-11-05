use action::Action;
use automata::Automata;
use automata::AutomataState;
use std::hashmap::HashMap;
use std::hashmap::HashMapIterator;
use std::hashmap::HashSet;
use std::hashmap::HashSetIterator;

// a deterministic finite automata
struct DFA {
    priv states: ~HashMap<uint, ~State>,
    priv finals: ~HashSet<uint>,
    priv initial: uint
}

// each state corresponds to a set of states of the 
// non-deterministic automata that we built this DFA
// from. we keep this set for the algorithm below needs
// to know if we already have a state for this set
struct State {
    priv nfa_states: ~HashSet<uint>,
    priv trans: ~HashMap<u8, uint>,

    // always empty but needed for 
    // implementing AutomataState
    priv etrans: ~HashSet<uint>,
    
    // index in action table of action to
    // execute when the DFA is left in this
    // state. Also tells if this state is 
    // final. If this is None, it's not final
    priv action: Option<uint>
}

impl AutomataState for State {
    fn transitions(&self) -> ~[(u8, uint)] {
        let mut transitions = ~[];

        for (ch, dst) in self.trans.iter() {
            transitions.push((*ch, *dst));
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

impl Automata<State> for DFA {
    fn label(&self, st: uint) -> Option<~str> {
        match self.states.find(&st).unwrap().action {
            Some(i) => Some(format!("{:u} (regex={:u})", st, i)),
            None => Some(format!("{:u}", st))
        }
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

impl DFA {
    // "determinization" of a NFA
    pub fn new_from_nfa(nfa: &::nfa::NFA, atb: &mut HashMap<uint, ~Action>) -> ~DFA {
        let current_id = &mut 0;

        // we associate a unique number to each state we create to index them
        // this utility function returns the next number
        fn gen_state_num(current_id: &mut uint) -> uint {
            *current_id += 1;
            *current_id
        }

        // creates a new state for the automata that corresponds to the set
        // of positions `st` in the non-deterministic automata
        fn new_state(st: ~HashSet<uint>, current_id: &mut uint) -> (uint, ~State) {
            let id = gen_state_num(current_id);
            let ret = ~State { 
                nfa_states: st, 
                trans: ~HashMap::new(),
                etrans: ~HashSet::new(),
                action: None
            };

            (id, ret)
        }

        let mut ret = ~DFA {
            states: ~HashMap::new(),
            finals: ~HashSet::new(),
            initial: 1
        };

        // the first state to treat is the entry of the DFA. it corresponds
        // to e-closure(e0), where e0 is the entry of the NFA
        let mut st = ~HashSet::new();
        st.insert(nfa.initial());

        // create the entry state of the DFA
        let (f_id, fstate) = new_state(nfa.eclosure(st), current_id);
        ret.states.insert(f_id, fstate);

        // stack of untreated states
        let mut unmarked = ~[];
        unmarked.push(f_id);

        while !unmarked.is_empty() {
            let next_id = unmarked.pop();

            // FIXME: hardcoded value
            for i in range(0, 255 as u8) {
                let mut trans = ~HashMap::new();
                let mut newstates = ~HashMap::new();

                match ret.states.find(&next_id) {
                    Some(st) => {
                        let mut tr = ~HashSet::new();

                        // this state is final if it corresponds to
                        // at least one nfa final state. its action will
                        // be the action of the first final state built
                        let mut action = None;


                        // list all the states in which we can be after
                        // transiting from this state by the current char
                        for st in st.nfa_states.iter() {
                            let st = nfa.find_state(*st).unwrap();

                            let mut trans = match st.trans(i) {
                                Some(t) => t,
                                None => continue
                            };

                            for t in trans {
                                // if we don't already have this state,
                                // add it, as well as all states we can
                                // transition from it by epsilon
                                if !tr.contains(t) {
                                    tr.insert(*t);

                                    let eclos = nfa.eclosure_(*t);
                                    for st in eclos.iter() {
                                        tr.insert(*st);
                                        let st = nfa.find_state(*st).unwrap();

                                        match (st.action(), action) {
                                            (Some(act), None) => action = Some(act),
                                            (Some(act), Some(ref a)) => {
                                                let act = {
                                                    let r = atb.get(&act);
                                                    r.clone()
                                                };
                                                atb.find_mut(a).unwrap().merge(&*act);
                                            }

                                            // non final state
                                            (None, _) => ()
                                        }
                                    }
                                }
                            }
                        }

                        if tr.is_empty() {
                            // this state has no i-transitions
                            // FIXME: add a "dead state"
                            continue;
                        }

                        // find if we already have a DFA state that
                        // corresponds to this state set of the NFA
                        let mut id = None;
                        for (s, st) in ret.states.iter() {
                            if st.nfa_states == tr {
                                id = Some(s);
                                break;
                            }
                        }

                        let _id = match id {
                            Some(i) => *i,
                            None => {
                                // we don't have this sate, create it
                                let (nid, nst) = new_state(tr, current_id);
                                let mut nst = nst;
                                nst.action = action;

                                // ret is already borrowed, keep this state
                                // to add it to ret after
                                newstates.insert(nid, nst);
                                nid
                            }
                        };

                        // in any case, add a transition
                        trans.insert(i, _id);
                    }

                    None => fail!()
                }

                // now ret isn't borrowed anymore, apply modifications
                
                for (i, p) in newstates.move_iter() {
                    if p.is_final() {
                        ret.finals.insert(i);
                    }

                    ret.states.insert(i, p);

                    // add this state to the list of untreated
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

        ret
    }

    pub fn minimize(&mut self) -> ~DFA {
        let mut groups: ~[~HashSet<uint>] = ~[];

        // In a normal minimization algoritm, the initial groups are the set
        // of the finals and the non finals. Here, we need to ensure that two
        // final states that correspond to different regexes won't be merged
        // so he have 1 subgroup for each regex pattern containing final states
        // corresponding to this regex, plus one for non-final states

        for i in self.finals.iter() {
            let act = self.states.find(i).unwrap().action.unwrap();
            let mut inserted = false;

            for group in groups.mut_iter() {
                let e = {
                    match group.iter().next() {
                        Some(u) => Some(*u),
                        None => None
                    }
                };

                match e {
                    Some(s) => {
                        let a2 = self.find_state(s).unwrap().action.unwrap();
                        
                        if a2 == act {
                            group.insert(*i);
                            inserted = true;
                            break;
                        }
                    }

                    None => continue
                }
            }

            if !inserted {
                // we don't have a group yet for states corresponding to this
                // one's regex, create a new one:
                let mut group = ~HashSet::new();
                group.insert(*i);
                groups.push(group);
            }
        }

        // create a group for non-final states
        let mut nonfinal = ~HashSet::new();

        for (i, st) in self.states.iter() {
            if !st.is_final() {
                nonfinal.insert(*i);
            }
        }
        
        groups.push(nonfinal);

        let new_part = |part: &[~HashSet<uint>]| -> ~[~HashSet<uint>] {
            let mut ret = ~[];

            for group in part.iter() {
                let mut subgroups: ~[~HashSet<uint>] = ~[];

                for s in group.iter() {
                    let mut inserted = false;
                    for g in subgroups.mut_iter() {
                        // s goes to g if it has transitions to states in
                        // the same group in part as the elements of g
                        if g.is_empty() {
                            continue;
                        }

                        let e = match g.iter().next() {
                            Some(e) => *e,
                            None => fail!()
                        };

                        match (self.states.find(&e), self.states.find(s)) {
                            (Some(st1), Some(st2)) => {
                                let mut similar = true;
                                for (c, s1) in st1.trans.iter() {
                                    match st2.trans.find(c) {
                                        Some(s2) => {
                                            for s in part.iter() {
                                                if (s.contains(s2) && !s.contains(s1)) || (s.contains(s1) && !s.contains(s2)) {
                                                    similar = false;
                                                    break;
                                                }
                                            }
                                        }

                                        None => similar = false
                                    }
                                }

                                for (c, s1) in st2.trans.iter() {
                                    match st1.trans.find(c) {
                                        Some(s2) => {
                                            for s in part.iter() {
                                                if (s.contains(s2) && !s.contains(s1)) || (s.contains(s1) && !s.contains(s2)) {
                                                    similar = false;
                                                    break;
                                                }
                                            }
                                        }

                                        None => similar = false
                                    }
                                }


                                if similar {
                                    g.insert(*s);
                                    inserted = true;
                                    break;
                                }
                            }
                            _ => fail!()
                        }
                    }

                    if !inserted {
                        // no group matches s, create a new one
                        let mut ngroup = ~HashSet::new();
                        ngroup.insert(*s);
                        subgroups.push(ngroup);
                    }
                }

                for g in subgroups.move_iter() {
                    ret.push(g);
                }
            }

            ret
        };

        fn print_group(group: &HashSet<uint>) {
            print("group: { ");

            for i in group.iter() {
                print!("{:u}, ", *i);
            }

            println("}");
        }

        fn print_parts(part: &[~HashSet<uint>]) {
            println("[");

            for set in part.iter() {
                print_group(&**set);
            }

            println("]");
        }

        let mut part = groups;
        
        loop {
            let new_part = new_part(part);

            if new_part == part {
                break;
            }

            part = new_part;
        }

        let mut ret = ~DFA {
            states: ~HashMap::new(),
            finals: ~HashSet::new(),
            initial: 0,
        };

        let mut reps = ~[];

        for group in part.move_iter() {
            /* final ? */
            let mut final = false;
            for i in group.iter() {
                if self.finals.contains(i) {
                    final = true;
                }
            }

            let initial = group.contains(&self.initial);

            // take a representing state
            let rep = group.iter().next().unwrap();
            reps.push(*rep);

            // adjust transitions
            // FIXME: very inefficient
            for (_, st) in self.states.mut_iter() {
                for (_, dst) in st.trans.mut_iter() {
                    if group.contains(dst) {
                        *dst = *rep;
                    }
                }
            }

            if initial {
                ret.initial = *rep;
            }

            if final {
                ret.finals.insert(*rep);
            }
        }

        for r in reps.iter() {
            let st = self.states.pop(r).unwrap();
            ret.states.insert(*r, st);
        }

        ret
    } 

    pub fn transition_table(&self) -> (~[~[uint]], ~[uint], uint) {
        let mut new_indexes = ~HashMap::new();
        let mut old_indexes = ~HashMap::new();
        let mut current_index = 1u;

        for (i, _) in self.states.iter() {
            new_indexes.insert(*i, current_index); 
            old_indexes.insert(current_index, *i);
            current_index += 1;
        }

        let trans_tb = ::std::vec::from_fn(current_index, |i: uint| {
            let mut trans_table = ~[0, ..256];

            if i != 0 {
                let old_idx = old_indexes.find(&i).unwrap();
                let st = self.states.find(old_idx).unwrap();

                for (ch, dst) in st.trans.iter() {
                    trans_table[*ch] = *new_indexes.find(dst).unwrap();
                }
            }

            trans_table
        });

        let final_tb = ::std::vec::from_fn(current_index, |i: uint| {
            if i == 0 { 0 }
            else {
                let old_idx = old_indexes.find(&i).unwrap();
                let st = self.states.find(old_idx).unwrap();

                match st.action() { 
                    Some(a) => a,
                    None => 0
                }
            }                
        });

        let init_st = *new_indexes.find(&self.initial).unwrap();
        (trans_tb, final_tb, init_st)
    }
}

