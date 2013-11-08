use automata::Automata;
use automata::AutomataState;
use dfa::DFA;
use std::hashmap::HashMap;

/*
 * Converts a set of deterministic automatas into their transition-table
 * representation that will be used to browse the automata. The transition
 * tables are returned as a bidimensional array. 
 * In order for this function to work properly, the given automatas must
 * not have states with the same ID
 * This function assumes the first automata in the array corresponds to the 
 * initial 
 */
pub fn transition_table(a: &[~DFA]) -> (~[~[uint]], ~[uint], ~HashMap<uint, uint>) {

    /* 
     * Due to determinisation and minimization algorithms, the state IDs used
     * by those automata may not be linear. This gives all states a new ID
     * to ensure the transition table will only contain actual states, which
     * allow to reduce its size and improve code safety.
     * The newly generated IDs start at 1 since 0 will be the dead state in
     * the resulting automata.
     */
    let mut new_indexes = ~HashMap::new();
    let mut old_indexes = ~HashMap::new();
    let mut current_index = 1u;
    let mut automata = 0;

    while automata < a.len() {
        let au = &a[automata];

        for (i, _) in au.states_iter() {
            new_indexes.insert(*i, current_index); 

            // also remember which automata this states belongs to
            println!("ID {:u} from automata {:u} is now {:u}", *i, automata, current_index);
            old_indexes.insert(current_index, (*i, automata));
            current_index += 1;
        }
            
        automata += 1;
    }

    let trans_tb = ::std::vec::from_fn(current_index, |i: uint| {
        let mut trans_table = ~[0, ..256];

        if i != 0 {
            println!("attempting to access new index {:u}", i);
            let &(old_idx, au) = old_indexes.find(&i).unwrap();
            let au = &a[au];
            let st = au.find_state(old_idx).unwrap();

            for (ch, dst) in st.trans_iter() {
                trans_table[*ch] = *new_indexes.find(dst).unwrap();
            }
        }

        trans_table
    });

    let final_tb = ::std::vec::from_fn(current_index, |i: uint| {
        if i == 0 { 0 }
        else {
            let &(old_idx, au) = old_indexes.find(&i).unwrap();
            let au = &a[au];
            let st = au.find_state(old_idx).unwrap();

            match st.action() { 
                Some(a) => a,
                None => 0
            }
        }                
    });

    (trans_tb, final_tb, new_indexes)
}
