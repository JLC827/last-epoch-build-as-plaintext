use std::fs::File;
use std::io::BufReader;
use serde_json::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("core_db.json")?;
    let reader = BufReader::new(file);
    let json: Value = serde_json::from_reader(reader)?;

    println!("JSON loaded.");
    
    if let Some(list) = json.get("playerPropertyList").and_then(|v| v.as_array()) {
        if let Some(item) = list.get(62) {
            println!("Found Item at index 62 of playerPropertyList:");
            println!("{}", serde_json::to_string_pretty(item)?);
        } else {
            println!("Index 62 out of bounds.");
        }
    }

    Ok(())
}

