use std::fs::File;
use std::io::BufReader;
use serde_json::Value;

fn main() {
    let file = File::open("item_db.json").expect("file should open read only");
    let reader = BufReader::new(file);
    let v: Value = serde_json::from_reader(reader).expect("file should be proper JSON");

    if let Some(unique_list) = v.get("uniqueList").and_then(|v| v.get("uniques")).and_then(|v| v.as_object()) {
        for (key, value) in unique_list {
            if let Some(id) = value.get("id").and_then(|v| v.as_str()) {
                if id == "UAzDsEYDYA4g" {
                    println!("Found ID in key: {}", key);
                    println!("{}", serde_json::to_string_pretty(value).unwrap());
                    return;
                }
            }
        }
    }
    println!("Item not found");
}
