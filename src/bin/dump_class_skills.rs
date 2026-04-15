use anyhow::Result;
use headless_chrome::{Browser, LaunchOptions};
use std::time::Duration;

fn main() -> Result<()> {
    let mut options = LaunchOptions::default();
    options.headless = true;
    options.args.push(std::ffi::OsStr::new("--disable-web-security"));
    let browser = Browser::new(options)?;
    let tab = browser.new_tab()?;
    
    println!("Navigating...");
    tab.navigate_to("https://www.lastepochtools.com/planner/BZ3ZagqY")?;
    tab.wait_until_navigated()?;
    
    std::thread::sleep(Duration::from_secs(5));
    
    println!("Evaluating JS...");
    let eval = tab.evaluate(r#"
        (function() {
            let res = {};
            if (window.coreDB && window.coreDB.classes) {
                res.coreDB_classes = window.coreDB.classes.map(c => ({
                    id: c.classID,
                    masteries: c.masteries.map(m => m.skills ? m.skills.map(s => s.ability) : [])
                }));
            }
            if (window.LEAbilities && window.LEAbilities.abilitiesByClass) {
                res.abilitiesByClass = window.LEAbilities.abilitiesByClass;
            }
            
            // Search le_char_trees for skills related to classes
            res.charTrees = [];
            if (window.LECharTrees && window.LECharTrees.trees) {
                res.charTrees = window.LECharTrees.trees.map((t, idx) => {
                    let skills = [];
                    if (t.characterTree && t.characterTree.nodes) {
                        for (let k in t.characterTree.nodes) {
                            let n = t.characterTree.nodes[k];
                            // what unlocks skills?
                            if (n.unlockedAbilities) n.unlockedAbilities.map(u => skills.push(u.ability));
                            if (n.relatedAbilities) n.relatedAbilities.map(r => skills.push(r.ability));
                        }
                    }
                    return { id: idx, skills: skills };
                });
            }
            
            return JSON.stringify(res);
        })()
    "#, false)?;
    
    if let Some(val) = eval.value {
        if let Some(s) = val.as_str() {
            println!("Result: {}", s);
            std::fs::write("class_skills_dump.json", s)?;
        }
    }
    
    Ok(())
}
