//! Uses Petgraph to visualise the HIR as a graph

use std::collections::HashMap;

use crate::*;
use petgraph::dot::{Dot, Config};
use petgraph::graph::NodeIndex;
use petgraph::Graph;

/// Visualize the HIR as a graph
pub fn visualize(hir: &Body) {
    let graph = graphify(hir);
    let dot = Dot::with_config(
        &graph, 
        &[Config::GraphContentOnly]
    );
    
    println!("digraph {{");
    println!("    rankdir=TB");
    println!("    node [shape=box style=filled fontsize=8 fontname=Verdana fillcolor=\"#efefef\"]");
    println!("    edge [fontsize=8 fontname=Verdana]");
    println!();
    println!("{}", dot);
    println!("}}");
}

/// Graphify the HIR
pub(crate) fn graphify(hir: &Body) -> Graph<String, String> {
    let mut graph = Graph::<String, String>::new();
    
    // Go through each basic block and add it to the graph (should be 1 to 1)
    let mut bb_map: HashMap<NodeIndex, usize> = HashMap::new();
    
    // Add the basic blocks to the graph
    for (i, bb) in hir.basic_blocks.iter().enumerate() {
        let weight = format!("{}", bb);
        let idx = graph.add_node(weight);
        bb_map.insert(idx, i);
    }
    
    // Connect the basic blocks
    for (i, bb) in hir.basic_blocks.iter().enumerate() {
        let terminator: &Option<Terminator> = &bb.terminator;
        let source: NodeIndex = NodeIndex::new(i);
        match terminator {
            Some(Terminator::Goto {target}) => {
                let target_idx = bb_map
                    .get(&NodeIndex::new(target.0))
                    .unwrap();
                let target: NodeIndex = NodeIndex::new(*target_idx);
                graph.add_edge(source, target, "".to_owned());
            },
            Some(Terminator::SwitchInt { value, targets }) => {
                // Add a comment to the label
                let weight = &mut graph[source];
                weight.push_str(&format!("switch({})", value));
                for (i, v) in targets.values.iter().enumerate() {
                    let bb_idx = targets.blocks[i].0;
                    let target_idx = bb_map
                        .get(&NodeIndex::new(bb_idx))
                        .unwrap();
                    let target: NodeIndex = NodeIndex::new(*target_idx);
                    graph.add_edge(source, target, format!("{}", v));
                }
            },
            Some(Terminator::Return) => {
                // Add a comment to the label
                let weight = &mut graph[source];
                weight.push_str("return");
            }
            None => (),
            _ => unimplemented!()
        }
    }

    graph
}

// std::fmt::Display implementations

impl std::fmt::Display for BasicBlockData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.statements.iter().for_each(|stmt| {
            writeln!(f, "{}", stmt).unwrap();
        });

        Ok(())
    }
}

impl std::fmt::Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Assign(place, rvalue) => {
                write!(f, "{} = {}", place, rvalue)
            }
        }
    }
}

impl std::fmt::Display for Place {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Place::Local(local) => {
                write!(f, "%{}", local)
            }
        }
    }
}

impl std::fmt::Display for Rvalue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Rvalue::Use(operand) => {
                write!(f, "{}", operand)
            },
            Rvalue::BinaryOp(op, lhs, rhs) => {
                write!(f, "{} {} {}", lhs, op, rhs)
            }
            Rvalue::UnaryOp(op, operand) => {
                write!(f, "{}{}", op, operand)
            }
        }
    }
}

impl std::fmt::Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Copy(place) => {
                write!(f, "copy {}", place)
            },
            Operand::Move(place) => {
                write!(f, "move {}", place)
            },
            Operand::Constant(constant) => {
                write!(f, "{}", constant)
            }
        }
    }
}

impl std::fmt::Display for Const {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.value, self.ty)
    }
}

impl std::fmt::Display for UnOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnOp::Neg => write!(f, "-"),
            UnOp::Not => write!(f, "!"),
        }
    }
}

impl std::fmt::Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Sub => write!(f, "-"),
            BinOp::Mul => write!(f, "*"),
            BinOp::Div => write!(f, "/"),
            BinOp::Rem => write!(f, "%"),
            BinOp::BitXor => write!(f, "^"),
            BinOp::BitAnd => write!(f, "&"),
            BinOp::BitOr => write!(f, "|"),
            BinOp::ShiftLeft => write!(f, "<<"),
            BinOp::ShiftRight => write!(f, ">>"),
            BinOp::Eq => write!(f, "=="),
            BinOp::Ne => write!(f, "!="),
            BinOp::Lt => write!(f, "<"),
            BinOp::Gt => write!(f, ">"),
            BinOp::Le => write!(f, "<="),
            BinOp::Ge => write!(f, ">="),
        }
    }
}