use tree_sitter::{Language, Node, Parser};
use anyhow::{Context, Result};

use crate::models::ParsedFunction;

extern "C" {
    fn tree_sitter_rust() -> Language;
}

const BRANCH_KINDS: &[&str] = &[
    "if_expression",
    "else_clause",
    "match_expression",
    "match_arm",
    "for_expression",
    "while_expression",
    "loop_expression",
    "binary_expression", 
    "try_expression",   
    "closure_expression",
];
