use std::io::Writer;

pub struct Action {
    pattern: ~str,
    action: ~str
}

impl Action {
    pub fn new(pat: ~str, act: ~str) -> ~Action {
        ~Action {
            pattern: pat,
            action: act
        }
    }

    pub fn write(&self, num: uint, out: &mut Writer) {
        writeln!(out, "            {:u} => \\{", num);
        writeln!(out, "                 {:s}", self.action);
        writeln!(out, "            \\}");
    }
}
