use std::fs::File;
use std::io::BufReader;
use serde_json::Value;
use anyhow::Result;

fn main() -> Result<()> {
    let file = File::open("item_db.json")?;
    let reader = BufReader::new(file);
    let json: Value = serde_json::from_reader(reader)?;

    if let Some(obj) = json.as_object() {
        if let Some(unique_list) = obj.get("uniqueList") {
            if let Some(inner) = unique_list.get("uniques") {
                 if let Some(unique_map) = inner.as_array() {
                    println!("uniqueList.uniques is an ARRAY with {} items", unique_map.len());
                     // Search in array
                    let target_id = "UAzDsEYDYA4g";
                    for value in unique_map {
                        if let Some(id_str) = value.get("id").and_then(|v| v.as_str()) {
                            if target_id.starts_with(id_str) {
                                println!("Found prefix match for {}: ID={} NameKey={:?}", target_id, id_str, value.get("displayNameKey"));
                            }
                        }
                    }
                 } else if let Some(unique_map) = inner.as_object() {
                    println!("uniqueList.uniques is an OBJECT with {} keys", unique_map.len());
                    
                    let target_id = "UAzDsEYDYA4g";
                    let mut found = false;

                    for (key, value) in unique_map {
                        if let Some(id_str) = value.get("id").and_then(|v| v.as_str()) {
                            if id_str == target_id {
                                println!("Found exact match for {}: Key={}", target_id, key);
                                found = true;
                            }
                            if target_id.starts_with(id_str) {
                                println!("Found prefix match for {}: ID={} Key={} NameKey={:?}", target_id, id_str, key, value.get("displayNameKey"));
                                found = true;
                            }
                        }
                    }
                 } else {
                     println!("uniqueList.uniques is neither array nor object");
                 }
            } else {
                println!("uniqueList.uniques not found");
            }
        } else {
            println!("uniqueList not found");
        }
    }
    Ok(())
}
