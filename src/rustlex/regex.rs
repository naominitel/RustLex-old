use std::hashmap::HashMap;
use std::hashmap::HashSet;
use std::libc;

/* definitions for Rust wrapper structs */

pub type Position = (char, uint);

pub enum AST {
    Or(~AST, ~AST),
    Cat(~AST, ~AST),
    Clos(~AST),
    Char(Position)
}

impl AST {
    unsafe fn new_from_c_ast(a: &rustlex_c_ast) -> ~AST {
        if a.ty == RustlexOr as i32 {
            let opl = AST::new_from_c_ast(&*a.op_left);
            let opr = AST::new_from_c_ast(&*a.op_right);
            ~Or(opl, opr)
        }

        else if a.ty == RustlexCat as i32 {
            let opl = AST::new_from_c_ast(&*a.op_left);
            let opr = AST::new_from_c_ast(&*a.op_right);
            ~Cat(opl, opr)
        }

        else if a.ty == RustlexClos as i32 {
            let op = AST::new_from_c_ast(&*a.op_left);
            ~Clos(op)
        }

        else {
            ~Char((a.const_c as u8 as char, a.const_pos as uint))
        }
    }

    pub fn nullable(&self) -> bool {
        match *self {
            Or(ref opl, ref opr) => opl.nullable() || opr.nullable(),
            Cat(ref opl, ref opr) => opl.nullable() && opr.nullable(),
            Clos(_) => true,
            _ => false
        }
    }

    pub fn final_pos(&self) -> ~HashSet<Position> {
        match *self {
            Or(ref opl, ref opr) => {
                let lfinals = opl.final_pos();
                let rfinals = opr.final_pos();
                let mut ret = ~HashSet::new();

                for p in lfinals.union_iter(rfinals) {
                    ret.insert(*p);
                }

                ret
            }

            Cat(ref opl, ref opr) => {
                let mut ret = opr.final_pos();
                
                if opr.nullable() {
                    let lfinals = opl.final_pos();
                    for p in lfinals.iter() {
                        ret.insert(*p);
                    }
                }

                ret
            }

            Clos(ref op) => {
                op.final_pos()
            }
            
            Char(pos) => {
                let mut ret = ~HashSet::new();
                ret.insert(pos);
                ret
            }            
        }
    }

    pub fn first_pos(&self) -> ~HashSet<Position> {
        match *self {
            Or(ref opl, ref opr) => {
                let lfirsts = opl.first_pos();
                let rfirsts = opr.first_pos();
                let mut ret = ~HashSet::new();

                for p in lfirsts.union_iter(rfirsts) {
                    ret.insert(*p);
                }

                ret
            }

            Cat(ref opl, ref opr) => {
                let mut ret = opl.first_pos();

                if opl.nullable() {
                    let rfirsts = opr.first_pos();
                    for p in rfirsts.iter() {
                        ret.insert(*p);
                    }
                }

                ret
            }

            Clos(ref op) => {
                op.first_pos()
            }

            Char(pos) => {
                let mut ret = ~HashSet::new();
                ret.insert(pos);
                ret
            }
        }
    }

    pub fn follow_pos(&self) -> ~HashMap<Position, @mut HashSet<Position>> {
        match *self {
            Cat(ref opl, ref opr) => {
                let mut ret = ~HashMap::new();
        
                let posl = opl.follow_pos();  
                let posr = opr.follow_pos();

                /* merge hashmaps */

                for (k, v) in posl.iter() {
                    ret.swap(*k, *v);
                }

                for (k, v) in posr.iter() {
                    ret.insert_or_update_with(*k, *v, |_, val| {
                        // let union = val.union_iter(*v);
                        // *val = HashSet::<Position>::from_iterator(union);
                        for p in v.iter() {
                            val.insert(*p);
                        }
                    });
                }

                let lpos = @mut *opr.final_pos();
                let fpos = @mut *opl.first_pos();

                for p in lpos.iter() {
                    ret.insert_or_update_with(*p, fpos, |_, val| {
                        // let union = val.union_iter(fpos);
                        // *val = HashSet::from_iterator(union);
                        for p in fpos.iter() {
                            val.insert(*p);
                        }
                    });
                }

                ret
            }

            Clos(ref op) => {
                let mut ret = op.follow_pos();
                let lpos = @mut *op.final_pos();
                let fpos = @mut *op.first_pos();

                for p in lpos.iter() {
                    ret.insert_or_update_with(*p, fpos, |_, val| {
                        // let union = val.union_iter(fpos);@
                        // *val = HashSet::from_iterator(union);
                        for p in fpos.iter() {
                            val.insert(*p);
                        }
                    });
                }

                ret
            }

            _ => { ~HashMap::new() }
        }
    }
}

/* definitions for C part */

enum rustlex_c_type {
    RustlexOr = 0,
    RustlexCat = 1, 
    RustlexClos = 2,
    RustlexConst = 3
}

struct rustlex_c_ast {
    ty: libc::c_int,
    op_left: *rustlex_c_ast,
    op_right: *rustlex_c_ast,
    const_c: libc::c_char,
    const_pos: libc::c_uint
}

extern {
    fn rustlex_parse_regex(input: *libc::c_char) -> *rustlex_c_ast;
}

/* wrappers for C function calls */

#[fixed_stack_segment]
pub unsafe fn parse(input: *libc::c_char) -> ~AST {
    use std::vec::raw;
    let c_struct = rustlex_parse_regex(input);
    AST::new_from_c_ast(&*c_struct)
}

