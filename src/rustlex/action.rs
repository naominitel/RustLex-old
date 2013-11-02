use std::hashmap::HashMap;
use std::rt::io::Writer;

#[deriving(Clone)]
struct Action {
    id: uint,
    pattern: ~str,

    // couples actions/conditions
    actions: ~HashMap<~str,~str>
}

impl Action {
    pub fn new(pat: ~str, act: ~str, cond: ~str, id: uint) -> ~Action {
        let mut acts = ~HashMap::new();
        acts.insert(cond, act);

        ~Action {
            id: id,
            pattern: pat,
            actions: acts
        }
    }

    pub fn merge(&mut self, action: &Action) {
        for (cond, a) in action.actions.iter() {
            match self.actions.find_mut(cond) {
                Some(act) => {
                    if action.id < self.id {
                        *act = a.clone();
                    }

                    continue
                }
                None => ()
            }

            self.actions.insert(cond.clone(), a.clone());
        }
    }

    pub fn write(&self, num: uint, out: &mut Writer) {
        writeln!(out, "            {:u} => \\{", num);
        writeln!(out, "                match self.condition \\{");

        for (cond, act) in self.actions.iter() {
            writeln!(out, "                    {:s} => \\{", *cond);
            writeln!(out, "                        {:s}", *act);
            writeln!(out, "                    \\}");
        }

        writeln!(out, "                    _ => ()");
        writeln!(out, "                \\}");
        writeln!(out, "            \\}");
    }
}
