use std::fs::File;
use std::io::BufReader;
use serde_json::Value;
use anyhow::Result;

fn main() -> Result<()> {
    let file = File::open("item_db.json")?;
    let reader = BufReader::new(file);
    let json: Value = serde_json::from_reader(reader)?;

    if let Some(obj) = json.as_object() {
        // Check Affix List
        if let Some(affix_list) = obj.get("affixList").and_then(|v| v.as_object()) {
             if let Some(single) = affix_list.get("singleAffixes").and_then(|v| v.as_object()) {
                 if let Some((key, val)) = single.iter().next() {
                     println!("Affix Example: {} -> {:#?}", key, val);
                 }
             }
        }

        // Check Base Item Implicits
        if let Some(item_list) = obj.get("itemList").and_then(|v| v.as_object()) {
            println!("Found itemList with keys: {:?}", item_list.keys().collect::<Vec<_>>());
            if let Some(equippable) = item_list.get("equippable").and_then(|v| v.as_object()) {
                 println!("Found equippable with keys: {:?}", equippable.keys().take(5).collect::<Vec<_>>());
                 if let Some((key, base_type)) = equippable.iter().next() {
                     println!("Checking base type: {}", key);
                     if let Some(sub_items) = base_type.get("subItems").and_then(|v| v.as_array()) {
                         println!("Found subItems array of length: {}", sub_items.len());
                         if let Some(first_item) = sub_items.first() {
                             println!("Base Item Example: {:#?}", first_item);
                         }
                     } else {
                         println!("No subItems found in base_type");
                         println!("Base Type Object: {:#?}", base_type);
                     }
                 }
            } else {
                println!("Could not find equippable object");
            }
        } else {
            println!("Could not find itemList object");
        }
    }
    Ok(())
}
