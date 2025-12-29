use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use serde_json::Value;
use anyhow::Result;

pub struct Resolver {
    translations: HashMap<String, String>,
    affix_map: HashMap<String, AffixData>,
    ability_map: HashMap<String, AbilityData>,
    item_map: HashMap<String, String>,
}

struct AffixData {
    display_name_key: String,
    // tiers: Vec<TierData>, // We might not need detailed tier data if we just want to display "T5"
}

struct AbilityData {
    name: Option<String>,
    name_key: Option<String>,
}

impl Resolver {
    pub fn new() -> Result<Self> {
        let mut resolver = Resolver {
            translations: HashMap::new(),
            affix_map: HashMap::new(),
            ability_map: HashMap::new(),
            item_map: HashMap::new(),
        };

        resolver.load_translations("translations.json")?;
        resolver.load_affixes("item_db.json")?;
        resolver.load_items("item_db.json")?;
        resolver.load_abilities("le_abilities.json")?;

        Ok(resolver)
    }

    fn load_translations(&mut self, path: &str) -> Result<()> {
        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            let json: Value = serde_json::from_reader(reader)?;
            
            if let Some(obj) = json.as_object() {
                // Load 'full' translations
                if let Some(full) = obj.get("full").and_then(|v| v.as_object()) {
                    for (k, v) in full {
                        if let Some(s) = v.as_str() {
                            self.translations.insert(k.clone(), s.to_string());
                        }
                    }
                }
                // Load root translations if any (fallback)
                for (k, v) in obj {
                    if let Some(s) = v.as_str() {
                        self.translations.insert(k.clone(), s.to_string());
                    }
                }
            }
        }
        Ok(())
    }

    fn load_items(&mut self, path: &str) -> Result<()> {
        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            let json: Value = serde_json::from_reader(reader)?;
            
            if let Some(item_list) = json.get("itemList").and_then(|v| v.as_object()) {
                let categories = ["equippable", "nonEquippable"];
                for cat in categories {
                    if let Some(cat_obj) = item_list.get(cat).and_then(|v| v.as_object()) {
                        for (_base_type_id, base_type_val) in cat_obj {
                            if let Some(sub_items) = base_type_val.get("subItems").and_then(|v| v.as_object()) {
                                for (_sub_id, sub_item_val) in sub_items {
                                    if let Some(id) = sub_item_val.get("id").and_then(|v| v.as_str()) {
                                        if let Some(key) = sub_item_val.get("displayNameKey").and_then(|v| v.as_str()) {
                                            self.item_map.insert(id.to_string(), key.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn load_affixes(&mut self, path: &str) -> Result<()> {
        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            let json: Value = serde_json::from_reader(reader)?;
            
            if let Some(affix_list) = json.get("affixList").and_then(|v| v.as_object()) {
                // Helper to process affix objects
                let mut process_affixes = |affixes: &serde_json::Map<String, Value>| {
                    for (_key, value) in affixes {
                        if let Some(obj) = value.as_object() {
                            if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
                                let display_name_key = obj.get("affixDisplayNameKey")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                
                                self.affix_map.insert(id.to_string(), AffixData {
                                    display_name_key,
                                });
                            }
                        }
                    }
                };

                if let Some(single) = affix_list.get("singleAffixes").and_then(|v| v.as_object()) {
                    process_affixes(single);
                }
                if let Some(multi) = affix_list.get("multiAffixes").and_then(|v| v.as_object()) {
                    process_affixes(multi);
                }
            }
        }
        Ok(())
    }

    fn load_abilities(&mut self, path: &str) -> Result<()> {
        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            let json: Value = serde_json::from_reader(reader)?;
            
            if let Some(obj) = json.as_object() {
                for (id, value) in obj {
                    if let Some(ability_obj) = value.as_object() {
                        let name = ability_obj.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
                        let name_key = ability_obj.get("nameKey").and_then(|v| v.as_str()).map(|s| s.to_string());
                        
                        self.ability_map.insert(id.clone(), AbilityData {
                            name,
                            name_key,
                        });
                    }
                }
            }
        }
        Ok(())
    }

    pub fn get_affix_name(&self, id: &str) -> String {
        if let Some(data) = self.affix_map.get(id) {
            if let Some(trans) = self.translations.get(&data.display_name_key) {
                return trans.clone();
            }
            return data.display_name_key.clone();
        }
        id.to_string()
    }

    pub fn get_skill_name(&self, id: &str) -> String {
        // Try direct translation lookup pattern: Skills.Skill_{ID}_0_Name
        let direct_key = format!("Skills.Skill_{}_0_Name", id);
        if let Some(trans) = self.translations.get(&direct_key) {
            return trans.clone();
        }

        if let Some(data) = self.ability_map.get(id) {
            if let Some(name) = &data.name {
                return name.clone();
            }
            if let Some(key) = &data.name_key {
                if let Some(trans) = self.translations.get(key) {
                    return trans.clone();
                }
                return key.clone();
            }
        }
        id.to_string()
    }

    pub fn get_skill_node_name(&self, skill_id: &str, node_id: &str) -> String {
        let key = format!("Skills.Skill_{}_{}_Name", skill_id, node_id);
        if let Some(trans) = self.translations.get(&key) {
            return trans.clone();
        }
        format!("Node {}", node_id)
    }

    pub fn get_passive_name(&self, class_id: u8, node_id: u8) -> String {
        let prefix = match class_id {
            0 => "pr",
            1 => "mg",
            2 => "se",
            3 => "ac",
            4 => "rg",
            _ => return format!("Unknown Class {}", class_id),
        };
        
        let key = format!("Skills.Skill_{}-1_{}_Name", prefix, node_id);
        
        if let Some(trans) = self.translations.get(&key) {
            return trans.clone();
        }
        format!("Passive {}", node_id)
    }

    pub fn get_item_name(&self, id: &str) -> String {
        if let Some(key) = self.item_map.get(id) {
            if let Some(trans) = self.translations.get(key) {
                return trans.clone();
            }
            return key.clone();
        }
        id.to_string()
    }
}
