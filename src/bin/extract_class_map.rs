use anyhow::Result;
use std::fs;
use regex::Regex;

fn main() -> Result<()> {
    let content = fs::read_to_string("hashed_js/planner_d0feed.js")?;
    
    let class_re = Regex::new(r"characterClass:\{.*?classID:(\d+).*?masteries:\[(.*?)\]\}\]\}")?;
    let ability_re = Regex::new(r#"ability:"([^"]+)""#)?;
    
    for cap in class_re.captures_iter(&content) {
        let class_id = &cap[1];
        let masteries_str = &cap[2];
        
        let mut abilities = Vec::new();
        for ab_cap in ability_re.captures_iter(masteries_str) {
            abilities.push(ab_cap[1].to_string());
        }
        println!("Class ID: {}, Abilities: {:?}", class_id, abilities.len());
    }
    
    Ok(())
}
