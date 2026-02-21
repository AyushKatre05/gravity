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

pub fn compute_all(
    files: &[crate::models::ParsedFile],
) -> Vec<(String, String, usize)> {
    let mut results = Vec::new();

    for pf in files {
        for func in &pf.functions {
            let score = compute_complexity(func).unwrap_or(1);
            results.push((pf.path.clone(), func.name.clone(), score));
        }
    }

    results
}

fn count_branches(node: &Node, source: &str, count: &mut usize) {
    match node.kind() {
        "if_expression" => *count += 1,
        "else_clause" => {
            // Only count `else if`, not bare `else`
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "if_expression" {
                    *count += 1;
                }
            }
        }
        "match_arm" => *count += 1,
        "for_expression" => *count += 1,
        "while_expression" => *count += 1,
        "loop_expression" => *count += 1,
        "try_expression" => *count += 1, // `?` operator
        "closure_expression" => *count += 1,
        "binary_expression" => {
            // Only `&&` and `||` add a branch
            let op = node
                .children(&mut node.walk())
                .find(|c| c.kind() == "&&" || c.kind() == "||");
            if op.is_some() {
                *count += 1;
            }
        }
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        count_branches(&child, source, count);
    }
}
