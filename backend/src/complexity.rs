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

pub fn compute_complexity(func: &ParsedFunction) -> Result<usize> {
    let mut parser = Parser::new();
    let lang = unsafe { tree_sitter_rust() };
    parser
        .set_language(lang)
        .context("Failed to set language for complexity parser")?;

    let tree = parser
        .parse(&func.body_source, None)
        .context("Failed to parse function body")?;

    let mut count = 0usize;
    count_branches(&tree.root_node(), &func.body_source, &mut count);

    Ok(count + 1) // baseline complexity = 1
}
