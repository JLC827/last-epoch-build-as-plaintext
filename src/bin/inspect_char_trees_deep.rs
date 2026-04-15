use anyhow::Result;
use std::fs;
use serde_json::Value;
use std::collections::HashSet;

fn main() -> Result<()> {
    let char_trees_data = fs::read_to_string("debug_data/le_char_trees.json")?;
    let char_trees: Value = serde_json::from_str(&char_trees_data)?;
    
    if let Some(trees) = char_trees.get("trees").and_then(|t| t.as_array()) {
        for (i, tree) in trees.iter().enumerate() {
            let mut skills = HashSet::new();
            if let Some(nodes) = tree.get("characterTree").and_then(|ct| ct.get("nodes")).and_then(|n| n.as_object()) {
                for (_, node) in nodes {
                    if let Some(rel) = node.get("relatedAbilities").and_then(|r| r.as_array()) {
                        for r in rel {
                            skills.insert(r.as_str().unwrap_or("?").to_string());
                        }
                    }
                    if let Some(unl) = node.get("unlockedAbilities").and_then(|u| u.as_array()) {
                        for u in unl {
                            skills.insert(u.as_str().unwrap_or("?").to_string());
                        }
                    }
                }
            }
            println!("Class {} related/unlocked skills: {:?}", i, skills);
        }
    }
    
    // Also check skillList if it exists?
    Ok(())
}
