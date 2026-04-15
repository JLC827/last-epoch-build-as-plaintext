use anyhow::Result;
use std::fs;
use serde_json::Value;

fn main() -> Result<()> {
    let char_trees_data = fs::read_to_string("debug_data/le_char_trees.json")?;
    let char_trees: Value = serde_json::from_str(&char_trees_data)?;
    
    if let Some(trees) = char_trees.get("trees").and_then(|t| t.as_array()) {
        for (i, tree) in trees.iter().enumerate() {
            let mut skills = Vec::new();
            if let Some(nodes) = tree.get("characterTree").and_then(|ct| ct.get("nodes")).and_then(|n| n.as_object()) {
                for (_, node) in nodes {
                    if let Some(unlocks) = node.get("relatedAbilities").and_then(|ua| ua.as_array()) {
                        for unlock in unlocks {
                            if let Some(ab) = unlock.get("ability").and_then(|a| a.as_str()) {
                                skills.push(ab.to_string());
                            }
                        }
                    }
                }
            }
            println!("Class {} unlock nodes count: {}", i, skills.len());
        }
    }
    Ok(())
}
