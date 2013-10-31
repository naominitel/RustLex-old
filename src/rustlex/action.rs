struct Action {
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
}
