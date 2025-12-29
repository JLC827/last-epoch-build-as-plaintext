use std::fs::File;
use std::io::BufReader;
use serde_json::Value;
use anyhow::Result;

fn main() -> Result<()> {
    let file = File::open("translations.json")?;
    let reader = BufReader::new(file);
    let json: Value = serde_json::from_reader(reader)?;

    if let Some(obj) = json.as_object() {
        println!("Keys in translations.json:");
        for key in obj.keys() {
            println!("- {}", key);
            if let Some(inner) = obj.get(key).and_then(|v| v.as_object()) {
                println!("  (Count: {})", inner.len());
                if inner.len() < 5 {
                    println!("  Content: {:?}", inner);
                }
            } else if let Some(inner) = obj.get(key).and_then(|v| v.as_array()) {
                println!("  (Array Length: {})", inner.len());
            } else {
                // Print type or a snippet
                println!("  (Type: {})", if obj.get(key).unwrap().is_string() { "String" } else { "Other" });
            }
        }
    } else {
        println!("translations.json is not an object");
    }

    Ok(())
}
