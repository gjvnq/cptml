#[macro_use] extern crate lalrpop_util;

pub mod prelude;
pub mod lexer;

lalrpop_mod!(pub calculator1); // synthesized by LALRPOP