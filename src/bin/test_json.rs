use std::fs;
use serde_json::Value;

fn main() {
    let s = fs::read_to_string("le_char_trees.json").unwrap();
    let d: Value = serde_json::from_str(&s).unwrap();
    let t = d.get("trees").unwrap();
    println!("Is array? {}", t.is_array());
    println!("As array len: {:?}", t.as_array().map(|a| a.len()));
    if let Some(arr) = t.as_array() {
        println!("Class 2: {:?}", arr[2].as_object().map(|o| o.keys().collect::<Vec<_>>()));
        
        let n = arr[2].get("characterTree").and_then(|c| c.get("nodes"));
        println!("Nodes exists? {}", n.is_some());
    }
}
