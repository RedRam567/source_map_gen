//! Vmf tools and scriping in [Rhai](https://docs.rs/rhai/latest/rhai) the scripting language.

use std::ops::Deref;
use std::{error::Error, fs};
use vmf_parser_nom::ast::Block;
use rhai::Engine;

// TODO: cylinder
// TODO: (multi) clipping
// TODO: get all brushes and entities, all solids, texts, block, prop
// TODO: low, mid, high level repr (vmf, mid, rooms)

#[derive(Clone, Debug, PartialEq, Eq)]
struct StrictEngine {
    cursor_mats: Vec<String>,
}

// fn str_to_vmf<'a>(s: &'a str) -> Result<Vec<Block<&'a str>>, &'static str> {
//     let x = s;
//     // vmfparser::parse(x)

// }

// fn str_to_vmf<'a>(s: &'a str) -> Result<Vec<Block<&'a str>>, &'static str> {
//     parse(s)
// }

// use synom::space::skip_whitespace;
// use synom::IResult;

// /// Parse a VMF string, returning the list of parsed blocks
// pub fn parse<'a, I, K>(input: &'a I) -> Result<Vec<Block<K>>, &'static str> where I: 'a + Deref<Target=str>, K: From<&'a str> {
//     let x: synom::IResult<&str, Vec<Block<_>>> = file(input);
//     // match file(input) {
//     //     IResult::Done(rem, ast) => if rem != "" {
//     //         Err("failed to parse the entire input")
//     //     } else {
//     //         Ok(ast)
//     //     },
//     //     IResult::Error => Err("parse error"),
//     // }
// }

// /// Parse a VMF string, returning the list of parsed blocks
// pub fn parse<'a, K>(input: &'a str) -> Result<Vec<Block<K>>, &'static str>
// where
//     K: From<&'a str>,
// {
//     match file(input) {
//         IResult::Done(rem, ast) => {
//             if skip_whitespace(rem) != "" {
//                 Err("failed to parse the entire input")
//             } else {
//                 Ok(ast)
//             }
//         }
//         IResult::Error => Err("parse error"),
//     }
// }

#[cfg(test)]
mod tests {
    use rhai::Dynamic;

    use super::*;

    #[test]
    fn test_rhai() {
        let mut engine = Engine::new();
        let ast = engine.compile_file("scripts/example.rhai".into()).unwrap();
        let result = engine.eval_ast::<i32>(&ast).unwrap();
        dbg!(result);
        panic!();
    }
}