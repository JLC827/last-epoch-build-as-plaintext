use anyhow::Result;
use std::fs;
use serde_json::Value;

fn main() -> Result<()> {
    let core_data = fs::read_to_string("debug_data/core_db.json")?;
    let core: Value = serde_json::from_str(&core_data)?;
    if let Some(classes) = core.get("classes").and_then(|c| c.as_array()) {
        for (i, class) in classes.iter().enumerate() {
            println!("Class: {:?}", class.as_object().unwrap().keys());
            let name = class.get("name").and_then(|n| n.as_str()).unwrap_or("?");
            println!("Class {}: {}", i, name);
            if let Some(masteries) = class.get("masteries").and_then(|m| m.as_array()) {
                for mastery in masteries {
                    println!("  Mastery keys: {:?}", mastery.as_object().unwrap().keys());
                }
            }
        }
    }
    Ok(())
}
