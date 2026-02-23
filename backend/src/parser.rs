use std::path::Path;
use anyhow::{Context, Result};
use walkdir::WalkDir;
use tree_sitter::{Language, Node, Parser};
use crate::models::{ParsedFile, ParsedFunction};

extern "C" {
    fn tree_sitter_rust() -> Language;
}
pub fn parse_directory(root_path: &str) -> Result<Vec<ParsedFile>> {
    let mut parser = Parser::new();
    let lang = unsafe { tree_sitter_rust() };
    parser
        .set_language(lang)
        .context("Failed to set tree-sitter Rust language")?;

    let mut results = Vec::new();
    for entry in WalkDir::new(root_path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().extension().map(|x| x == "rs").unwrap_or(false))
    {
        let path = entry.path();
        match parse_file(&mut parser, path) {
            Ok(pf) => results.push(pf),
            Err(e) => {
                tracing::warn!("Skipping {:?}: {e}", path);
            }
        }
    }

    Ok(results)
}
fn parse_file(parser: &mut Parser, path: &Path) -> Result<ParsedFile> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("Cannot read {}", path.display()))?;

    let tree = parser
        .parse(&source, None)
        .context("tree-sitter parse returned None")?;

    let root = tree.root_node();
    let line_count = source.lines().count();
    let module_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(str::to_owned);

    let mut functions = Vec::new();
    let mut imports = Vec::new();
    let mut structs = Vec::new();

    visit_node(&root, &source, &mut functions, &mut imports, &mut structs);

    let path_str = path
        .to_str()
        .unwrap_or_default()
        .replace('\\', "/")
        .to_owned();

    Ok(ParsedFile {
        path: path_str,
        module_name,
        line_count,
        functions,
        imports,
        structs,
    })
}

/// 
fn visit_node(
    node: &Node,
    source: &str,
    functions: &mut Vec<ParsedFunction>,
    imports: &mut Vec<String>,
    structs: &mut Vec<String>,
) {
    match node.kind() {
        "function_item" => {
            if let Some(func) = extract_function(node, source) {
                functions.push(func);
            }
        }
        "use_declaration" => {
            if let Some(import) = extract_use(node, source) {
                imports.push(import);
            }
        }
        "struct_item" => {
            if let Some(name) = extract_name(node, source) {
                structs.push(name);
            }
        }
        _ => {}
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        visit_node(&child, source, functions, imports, structs);
    }
}

fn extract_function(node: &Node, source: &str) -> Option<ParsedFunction> {
    let name = node
        .child_by_field_name("name")
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        .map(str::to_owned)?;

    let line_start = node.start_position().row + 1;
    let line_end = node.end_position().row + 1;

    // Check for `pub` visibility
    let is_public = node
        .child_by_field_name("visibility_modifier")
        .map(|n| {
            n.utf8_text(source.as_bytes())
                .unwrap_or("")
                .starts_with("pub")
        })
        .unwrap_or(false);

    // Check for async keyword
    let is_async = {
        let mut cursor = node.walk();
        node.children(&mut cursor)
            .any(|c| c.kind() == "async")
    };

    let body_source = node
        .utf8_text(source.as_bytes())
        .unwrap_or("")
        .to_owned();

    Some(ParsedFunction {
        name,
        line_start,
        line_end,
        is_public,
        is_async,
        body_source,
    })
}

fn extract_use(node: &Node, source: &str) -> Option<String> {
    node.utf8_text(source.as_bytes())
        .ok()
        .map(|s| s.trim().trim_end_matches(';').to_owned())
}

fn extract_name(node: &Node, source: &str) -> Option<String> {
    node.child_by_field_name("name")
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        .map(str::to_owned)
}
