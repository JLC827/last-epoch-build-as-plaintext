use std::fs::File;
use std::io::BufReader;
use serde_json::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("core_db.json")?;
    let reader = BufReader::new(file);
    let json: Value = serde_json::from_reader(reader)?;

    if let Some(property_list) = json.get("propertyList").and_then(|v| v.as_array()) {
        for prop in property_list {
            if let Some(id) = prop.get("property").and_then(|v| v.as_u64()) {
                if id == 98 {
                    println!("Found Property 98: {:?}", prop);
                }
            }
        }
    } else {
        println!("Could not find propertyList");
    }

    Ok(())
}
