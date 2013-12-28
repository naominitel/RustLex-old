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
    unsafe fn new_from_c_ast(a: &RustlexCAST) -> ~AST {
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
}

/* definitions for C part */

/* #[repr(C)] <- FIXME: uncomment when upgrading to master */
enum RustlexCType {
    RustlexOr = 0,
    RustlexCat = 1, 
    RustlexClos = 2,
    RustlexConst = 3
}

struct RustlexCAST {
    ty: libc::c_int,
    op_left: *RustlexCAST,
    op_right: *RustlexCAST,
    const_c: libc::c_char,
    const_pos: libc::c_uint
}

#[link(name = "lib/regex_parser.o")]
extern {
    fn rustlex_parse_regex(input: *libc::c_char) -> *RustlexCAST;
}

/* wrappers for C function calls */

pub unsafe fn parse(input: *libc::c_char) -> ~AST {
    let c_struct = rustlex_parse_regex(input);
    AST::new_from_c_ast(&*c_struct)
}

