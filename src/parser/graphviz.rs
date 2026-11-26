use anyhow::anyhow;
use log::debug;

use crate::cli::Args;
use crate::errors::TppCompilerError;
use crate::parser::{CstNode, CstNodeKind};
use std::fmt::Write as _;
use std::{fs::write, process::Command};

impl<'this> CstNode<'this> {
    pub fn to_graphviz(&self) -> String {
        let mut out = String::from("digraph CST {\n");
        out.push_str("    node [fontname=\"Helvetica\", fontsize=10];\n");
        let mut next_id = 0usize;
        self.write_dot(&mut out, &mut next_id);
        out.push_str("}\n");
        out
    }

    fn write_dot(&self, output: &mut String, next_id: &mut usize) -> usize {
        let current_id = *next_id;
        *next_id += 1;

        match &self.kind {
            CstNodeKind::NonTerminal(nt) => {
                let label = escape_dot_label(&format!("{nt:?}"));
                let _ = writeln!(
                    output,
                    "    n{current_id} [label=\"{label}\", shape=box, style=filled, fillcolor=\"#E8F0FE\"];"
                );
            }
            CstNodeKind::Terminal(token) => {
                let label = escape_dot_label(&format!("{:?}\n{}", token.kind(), token.value()));
                let _ = writeln!(
                    output,
                    "    n{current_id} [label=\"{label}\", shape=ellipse, style=filled, fillcolor=\"#E6F4EA\"];"
                );
            }
        }

        for child in &self.children {
            let child_id = child.write_dot(output, next_id);
            let _ = writeln!(output, "    n{current_id} -> n{child_id};");
        }

        current_id
    }
}

fn escape_dot_label(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

pub fn export_cst_graphviz(args: &Args, file: &str, cst: &CstNode<'_>) -> anyhow::Result<()> {
    let dot_path = format!("{file}.cst.dot");
    let png_path = format!("{file}.cst.png");

    write(&dot_path, cst.to_graphviz())?;
    debug!("[export_cst_graphviz::{file}] Arquivo DOT gerado em {dot_path}");

    let status = Command::new("dot")
        .args(["-Tpng", &dot_path, "-o", &png_path])
        .status()
        .map_err(|_err| {
            TppCompilerError::new_parser(
                "WAR-SYN-NOT-GEN-SYNTAX-TREE",
                args.use_errors_key,
                file.to_string(),
                None,
                None,
            )
        })?;

    if !status.success() {
        return Err(anyhow!(TppCompilerError::new_parser(
            "WAR-SYN-NOT-GEN-SYNTAX-TREE",
            args.use_errors_key,
            file.to_string(),
            None,
            None,
        )));
    }

    debug!("[export_cst_graphviz::{file}] Imagem gerada em {png_path}");

    Ok(())
}
