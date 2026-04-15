use anyhow::Result;
use std::fs;
use serde_json::Value;

fn main() -> Result<()> {
    let trees_data = fs::read_to_string("debug_data/le_skill_trees.json")?;
    let trees: Value = serde_json::from_str(&trees_data)?;
    if let Some(t) = trees.as_object() {
        for (k, v) in t.iter().take(5) {
            println!("Tree {}: {:?}", k, v.as_object().unwrap().keys());
            if let Some(ability) = v.get("ability").and_then(|a| a.as_str()) {
                println!("  Ability: {}", ability);
            }
        }
    }
    Ok(())
}
