use std::fs::File;
use std::io::BufReader;
use serde_json::Value;
use anyhow::Result;

fn main() -> Result<()> {
    let file = File::open("le_abilities.json")?;
    let reader = BufReader::new(file);
    let json: Value = serde_json::from_reader(reader)?;

    if let Some(obj) = json.as_object() {
        println!("Root keys: {:?}", obj.keys().take(10).collect::<Vec<_>>());
        
        // Try to find a skill by ID if I knew one.
        // Let's just print the first entry.
        if let Some((key, value)) = obj.iter().next() {
            println!("First entry key: {}", key);
            println!("First entry value: {:#?}", value);
        }
    }
    Ok(())
}
