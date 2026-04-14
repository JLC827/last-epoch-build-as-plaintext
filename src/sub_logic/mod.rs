use anyhow::Result;
use clap::Parser;
use headless_chrome::{Browser, LaunchOptions};
use std::fs::File;
use std::io::Write;
use std::time::Duration;
use serde_json::Value;

pub mod resolver;
pub mod idols_scraper;
use resolver::Resolver;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// URL of the build planner to scrape
    #[arg(required_unless_present = "idols", conflicts_with = "idols")]
    pub url: Option<String>,

    /// Scrape idols instead of build
    #[arg(long, conflicts_with = "url")]
    pub idols: bool,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<String>,
}

pub fn run() -> Result<()> {
    let args = Args::parse();

    if args.idols {
        return idols_scraper::scrape_idols();
    }

    let url = args.url.clone().ok_or_else(|| anyhow::anyhow!("URL is required when not using --idols"))?;

    let output_file_path = args.output.unwrap_or_else(|| {
        let parts = url.trim_end_matches('/').split('/');
        let name = parts.last().unwrap_or("build_data");
        format!("builds/{}.txt", name)
    });

    println!("Scraping URL: {}", url);
    // Ensure output directories exist
    std::fs::create_dir_all("builds").ok();
    std::fs::create_dir_all("debug_data").ok();

    let browser = Browser::new(LaunchOptions {
        headless: true,
        window_size: Some((1920, 1080)),
        args: vec![
            "--disable-blink-features=AutomationControlled",
            "--user-agent=Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
            "--no-sandbox",
            "--disable-setuid-sandbox",
        ].iter().map(|s| std::ffi::OsStr::new(s)).collect(),
        ..Default::default()
    })?;

    let tab = browser.new_tab()?;
    
    // Inject hook to intercept the planner data API fetch
    let inject_json_hook = r#"
        window.__letools_fetches = window.__letools_fetches || [];
        if (!window.__origFetch__installed) {
            window.__origFetch__installed = true;
            const originalFetch = window.fetch;
            window.fetch = async function() {
                let parsedArgs = Array.from(arguments);
                let url = parsedArgs[0];
                let res = await originalFetch.apply(this, arguments);
                
                if (typeof url === 'string' && url.includes('/api/internal/planner_data/')) {
                    let clone = res.clone();
                    try {
                        let text = await clone.text();
                        window.__letools_fetches.push({url: url, text: text});
                    } catch(e) {}
                }
                return res;
            };
        }
    "#;
    
    tab.call_method(headless_chrome::protocol::cdp::Page::AddScriptToEvaluateOnNewDocument {
        source: inject_json_hook.into(),
        world_name: None,
        include_command_line_api: Some(false),
        run_immediately: None,
    })?;

    // Navigate to the URL
    tab.navigate_to(&url)?;
    
    // Wait for the page to load
    println!("Waiting for page to load...");
    tab.wait_for_element("body")?;
    
    // Give React some time to render the dynamic content
    std::thread::sleep(Duration::from_secs(5));

    println!("Page loaded. Starting extraction...");

    // Dump LESkillTrees
    let skill_trees_res = tab.evaluate("JSON.stringify(window.LESkillTrees || {})", false)?;
    if let Some(val) = skill_trees_res.value {
        if let Some(s) = val.as_str() {
            std::fs::write("debug_data/le_skill_trees.json", s)?;
            println!("Saved le_skill_trees.json");
        }
    }

    // Dump LECharTrees
    let char_trees_res = tab.evaluate("JSON.stringify(window.LECharTrees || {})", false)?;
    if let Some(val) = char_trees_res.value {
        if let Some(s) = val.as_str() {
            std::fs::write("debug_data/le_char_trees.json", s)?;
            println!("Saved le_char_trees.json");
        }
    }

    // 1. Extract Translations
    println!("Extracting translations...");
    let translation_script = r#"
        (async () => {
            let version = 'version140';
            const match = document.documentElement.innerHTML.match(/\/data\/(version[^/]+)\//);
            if (match) {
                version = match[1];
            }
            const results = {};
            
            try {
                const resp = await fetch(`/data/${version}/i18n/full/en.json`);
                if (resp.ok) {
                    results['full'] = await resp.json();
                } else {
                    results['full'] = { error: `HTTP ${resp.status}` };
                }
            } catch (e) {
                results['full'] = { error: e.toString() };
            }
            
            // Static translations
            try {
                const resp = await fetch('/static_data/i18n/en.json');
                if (resp.ok) {
                    results['static'] = await resp.json();
                }
            } catch (e) {}

            return JSON.stringify(results);
        })()
    "#;
    
    let trans_res = tab.evaluate(translation_script, true)?;
    if let Some(val) = trans_res.value {
        if let Some(s) = val.as_str() {
            let mut file = File::create("debug_data/translations.json")?;
            file.write_all(s.as_bytes())?;
            println!("Saved translations.json");
        }
    }

    // 2. Extract LEAbilities
    println!("Extracting LEAbilities...");
    let abilities_res = tab.evaluate("JSON.stringify(window.LEAbilities || {})", false)?;
    if let Some(val) = abilities_res.value {
        if let Some(s) = val.as_str() {
            let mut file = File::create("debug_data/le_abilities.json")?;
            file.write_all(s.as_bytes())?;
            println!("Saved le_abilities.json");
        }
    }

    // 3. Extract coreDB
    println!("Extracting coreDB...");
    let coredb_res = tab.evaluate("JSON.stringify(window.coreDB || {})", false)?;
    if let Some(val) = coredb_res.value {
        if let Some(s) = val.as_str() {
            let mut file = File::create("debug_data/core_db.json")?;
            file.write_all(s.as_bytes())?;
            println!("Saved core_db.json");
        }
    }

    // 4. Extract itemDB
    println!("Extracting itemDB...");
    let itemdb_res = tab.evaluate("JSON.stringify(window.itemDB || {})", false)?;
    if let Some(val) = itemdb_res.value {
        if let Some(s) = val.as_str() {
            let mut file = File::create("debug_data/item_db.json")?;
            file.write_all(s.as_bytes())?;
            println!("Saved item_db.json");
        }
    }

    let build_info = extract_build_info(&tab)?;
    
    // Save build_info.json for debugging
    let mut file = File::create("debug_data/build_info.json")?;
    file.write_all(build_info.as_bytes())?;
    println!("Saved build_info.json");

    let build_json: Value = serde_json::from_str(&build_info)?;

    // Initialize Resolver
    println!("Initializing data resolver...");
    let resolver = Resolver::new()?;

    let mut file = File::create(&output_file_path)?;
    writeln!(file, "Build Data for {}\n", url)?;
    writeln!(file, "Note: Last Epoch tools does not report forging potential. Any non-corrupted items may have forging potential left over (but may be limited by forging level limits), or are due for corrupting.\n")?;
    
    write_character_info(&mut file, &build_json)?;

    writeln!(file, "--- Character Stats ---")?;
    write_stats(&mut file, &build_json)?;
    
    writeln!(file, "\n--- Skills & Passives ---")?;
    write_skills(&mut file, &build_json, &resolver)?;
    write_passives(&mut file, &build_json, &resolver)?;
    
    writeln!(file, "\n--- Equipment ---")?;
    write_equipment(&mut file, &build_json, &resolver)?;

    writeln!(file, "\n--- Idols ---")?;
    write_idols(&mut file, &build_json, &resolver)?;
    
    writeln!(file, "\n--- Blessings ---")?;
    write_blessings(&mut file, &build_json, &resolver)?;

    println!("Data saved to {}", output_file_path);

    // Copy to clipboard
    if let Ok(content) = std::fs::read_to_string(&output_file_path) {
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            if clipboard.set_text(content).is_ok() {
                println!("Data copied to clipboard.");
            } else {
                eprintln!("Failed to copy data to clipboard.");
            }
        } else {
            eprintln!("Failed to access clipboard.");
        }
    }

    Ok(())
}

fn extract_build_info(tab: &headless_chrome::Tab) -> anyhow::Result<String> {
    let script = r#"
        (function() {
            let fetches = window.__letools_fetches;
            if (!fetches || fetches.length === 0) return null;
            
            // Grab the last matching fetch payload
            let lastFetch = fetches[fetches.length - 1].text;
            try {
                let parsed = JSON.parse(lastFetch);
                const buildInfo = parsed;
                if (!buildInfo.data) return null;
                
                // Parse precalc_data if it's a string
                if (typeof buildInfo.precalc_data === 'string') {
                    try {
                        buildInfo.precalc_data = JSON.parse(buildInfo.precalc_data);
                    } catch (e) {
                        // ignore
                    }
                }
                
                return JSON.stringify(buildInfo);
            } catch(e) {
                return null;
            }
        })()
    "#;
    
    // Poll for up to 30 seconds
    let mut remote_object = tab.evaluate(script, false)?;
    for _ in 0..15 {
        if let Some(val) = &remote_object.value {
            if !val.is_null() {
                break;
            }
        }
        std::thread::sleep(Duration::from_secs(2));
        remote_object = tab.evaluate(script, false)?;
    }

    let value = remote_object.value.clone().ok_or_else(|| anyhow::anyhow!("No value returned from script: {:?}", remote_object))?;
    
    if value.is_null() {
        return Err(anyhow::anyhow!("fetch intercepted payload not found on page"));
    }

    
    if let Some(s) = value.as_str() {
        Ok(s.to_string())
    } else {
        Ok(value.to_string())
    }
}

fn write_character_info(file: &mut File, json: &Value) -> Result<()> {
    let mut name = "N/A".to_string();
    let mut class_id: Option<u8> = None;
    let mut mastery_id: Option<u8> = None;
    let mut level: Option<u64> = None;

    if let Some(bio) = json.get("data").and_then(|d| d.get("bio")) {
        class_id = bio.get("characterClass").and_then(|v| v.as_u64()).map(|v| v as u8);
        mastery_id = bio.get("chosenMastery").and_then(|v| v.as_u64()).map(|v| v as u8);
        level = bio.get("level").and_then(|v| v.as_u64());
        if let Some(n) = bio.get("name").and_then(|v| v.as_str()) {
            name = n.to_string();
        }
    }

    let class_name = match class_id {
        Some(0) => "Primalist",
        Some(1) => "Mage",
        Some(2) => "Sentinel",
        Some(3) => "Acolyte",
        Some(4) => "Rogue",
        _ => "Unknown Class",
    };

    let mastery_name = match (class_id, mastery_id) {
        (Some(0), Some(1)) => "Beastmaster",
        (Some(0), Some(2)) => "Shaman",
        (Some(0), Some(3)) => "Druid",
        (Some(1), Some(1)) => "Sorcerer",
        (Some(1), Some(2)) => "Spellblade",
        (Some(1), Some(3)) => "Runemaster",
        (Some(2), Some(1)) => "Void Knight",
        (Some(2), Some(2)) => "Forge Guard",
        (Some(2), Some(3)) => "Paladin",
        (Some(3), Some(1)) => "Lich",
        (Some(3), Some(2)) => "Necromancer",
        (Some(3), Some(3)) => "Warlock",
        (Some(4), Some(1)) => "Bladedancer",
        (Some(4), Some(2)) => "Marksman",
        (Some(4), Some(3)) => "Falconer",
        _ => "Base Class",
    };

    let level_str = level.map_or("N/A".to_string(), |l| l.to_string());
    writeln!(file, "Name: {}", name)?;
    writeln!(file, "Level: {}", level_str)?;
    writeln!(file, "Class: {}", class_name)?;
    writeln!(file, "Mastery: {}\n", mastery_name)?;

    Ok(())
}

fn write_stats(file: &mut File, json: &Value) -> Result<()> {
    if let Some(data) = json.get("precalc_data").and_then(|p| p.get("data")) {
        if let Some(general) = data.get("general") {
            writeln!(file, "General:")?;
            if let Some(obj) = general.as_object() {
                for (k, v) in obj {
                    writeln!(file, "  {}: {}", k, v)?;
                }
            }
        }
        
        if let Some(attributes) = data.get("attributes") {
            writeln!(file, "\nAttributes:")?;
            if let Some(obj) = attributes.as_object() {
                for (k, v) in obj {
                    writeln!(file, "  {}: {}", k, v)?;
                }
            }
        }
        
        if let Some(resistances) = data.get("resistances") {
            writeln!(file, "\nResistances:")?;
            if let Some(obj) = resistances.as_object() {
                for (k, v) in obj {
                    writeln!(file, "  {}: {}", k, v)?;
                }
            }
        }
        
        if let Some(defences) = data.get("defences") {
            writeln!(file, "\nDefences:")?;
            if let Some(obj) = defences.as_object() {
                for (k, v) in obj {
                    writeln!(file, "  {}: {}", k, v)?;
                }
            }
        }
    } else {
        writeln!(file, "No stats data found.")?;
    }
    Ok(())
}

fn write_skills(file: &mut File, json: &Value, resolver: &Resolver) -> Result<()> {
    // Load full skill trees graph
    let skill_trees_dump: Option<Value> = std::fs::read_to_string("debug_data/le_skill_trees.json")
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok());

    if let Some(data) = json.get("data") {
        if let Some(hud) = data.get("hud").and_then(|h| h.as_array()) {
            writeln!(file, "Active Skills (HUD):")?;
            for skill in hud {
                if let Some(id) = skill.as_str() {
                    let name = resolver.get_skill_name(id);
                    writeln!(file, "  - {}", name)?;
                    let desc = resolver.get_skill_description(id);
                    if !desc.is_empty() {
                        writeln!(file, "    {}", desc)?;
                    }
                } else {
                    writeln!(file, "  - {}", skill)?;
                }
            }
        }
        
        if let Some(trees) = data.get("skillTrees").and_then(|t| t.as_array()) {
            writeln!(file, "\n--- Skill Trees Configured ---")?;
            for tree in trees {
                if let Some(id) = tree.get("treeID").and_then(|v| v.as_str()) {
                    let name = resolver.get_skill_name(id);
                    writeln!(file, "Skill: {}", name)?;
                    
                    let selected = tree.get("selected").and_then(|s| s.as_object());
                    let nodes_obj = skill_trees_dump.as_ref().and_then(|d| d.get(id)).and_then(|t| t.get("nodes")).and_then(|n| n.as_object());

                    if let Some(nodes) = nodes_obj {
                        let mut sorted_keys: Vec<_> = nodes.keys().collect();
                        // Sort so output is deterministic
                        sorted_keys.sort_by_key(|k| k.parse::<u32>().unwrap_or(0));

                        for node_id_str in sorted_keys {
                            if let Some(node_data) = nodes.get(node_id_str) {
                                let points = selected.and_then(|s| s.get(node_id_str)).and_then(|v| v.as_u64()).unwrap_or(0);
                                let node_name = resolver.get_skill_node_name(id, node_id_str);
                                let max_pts = node_data.get("maxPoints").and_then(|v| v.as_u64()).unwrap_or(0);

                                let mut reqs = Vec::new();
                                if let Some(r_arr) = node_data.get("requirements").and_then(|v| v.as_array()) {
                                    for r in r_arr {
                                        if let Some(r_id) = r.get("nodeId").and_then(|v| v.as_u64()) {
                                            let req_pts = r.get("requirement").and_then(|v| v.as_u64()).unwrap_or(0);
                                            let r_name = resolver.get_skill_node_name(id, &r_id.to_string());
                                            reqs.push(format!("Requires {} pts in {}", req_pts, r_name));
                                        }
                                    }
                                }

                                writeln!(file, "  - Name: \"{}\"", node_name)?;
                                writeln!(file, "    AllocatedPoints: {}", points)?;
                                writeln!(file, "    MaxPoints: {}", max_pts)?;
                                if !reqs.is_empty() {
                                    writeln!(file, "    Requirements: \"{}\"", reqs.join(", "))?;
                                } else {
                                    writeln!(file, "    Requirements: None")?;
                                }
                                
                                // Print description
                                let node_desc = resolver.get_skill_node_description(id, node_id_str);
                                if !node_desc.is_empty() {
                                    // To keep YAML multiline, pad it
                                    let padded_desc = node_desc.replace('\n', "\n      ");
                                    writeln!(file, "    Effect: \"{}\"", padded_desc)?;
                                }
                                writeln!(file, "")?;
                            }
                        }
                    } else if let Some(sel) = selected {
                        // Fallback
                        for (node_id, points) in sel {
                            let node_name = resolver.get_skill_node_name(id, node_id);
                            writeln!(file, "  - {} (Points: {})", node_name, points)?;
                        }
                    }
                }
            }
        }
    } else {
        writeln!(file, "No skills data found.")?;
    }
    Ok(())
}

fn get_mastery_name(class_id: u8, mastery_id: u64) -> &'static str {
    match (class_id, mastery_id) {
        (0, 0) => "Primalist",
        (0, 1) => "Beastmaster",
        (0, 2) => "Shaman",
        (0, 3) => "Druid",
        (1, 0) => "Mage",
        (1, 1) => "Sorcerer",
        (1, 2) => "Spellblade",
        (1, 3) => "Runemaster",
        (2, 0) => "Sentinel",
        (2, 1) => "Void Knight",
        (2, 2) => "Forge Guard",
        (2, 3) => "Paladin",
        (3, 0) => "Acolyte",
        (3, 1) => "Necromancer",
        (3, 2) => "Lich",
        (3, 3) => "Warlock",
        (4, 0) => "Rogue",
        (4, 1) => "Bladedancer",
        (4, 2) => "Marksman",
        (4, 3) => "Falconer",
        _ => "Unknown Category",
    }
}

fn write_passives(file: &mut File, json: &Value, resolver: &Resolver) -> Result<()> {
    let class_id = json.get("data")
        .and_then(|d| d.get("bio"))
        .and_then(|b| b.get("characterClass"))
        .and_then(|v| v.as_u64())
        .unwrap_or(255) as u8;
    
    // Load full passive tree graph
    let char_trees_dump: Option<Value> = std::fs::read_to_string("debug_data/le_char_trees.json")
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok());

    if let Some(char_tree) = json.get("data").and_then(|d| d.get("charTree")) {
        writeln!(file, "\n--- Class Passives DAG ---")?;
        
        let selected_pts = char_tree.get("selected").and_then(|s| s.as_object());
        
        let nodes_obj = char_trees_dump
            .as_ref()
            .and_then(|d| d.get("trees"))
            .and_then(|t| t.as_array())
            .and_then(|arr| arr.get(class_id as usize))
            .and_then(|t| t.get("characterTree"))
            .and_then(|c| c.get("nodes"))
            .and_then(|n| n.as_object());

        if let Some(nodes) = nodes_obj {
            let mut mastery_groups: std::collections::HashMap<u64, Vec<&String>> = std::collections::HashMap::new();
            
            for (k, v) in nodes {
                let m = v.get("mastery").and_then(|v| v.as_u64()).unwrap_or(0);
                mastery_groups.entry(m).or_default().push(k);
            }

            let mut sorted_masteries: Vec<_> = mastery_groups.keys().copied().collect();
            sorted_masteries.sort_unstable();

            for mastery_id in sorted_masteries {
                let group_name = get_mastery_name(class_id, mastery_id);
                writeln!(file, "\n  [{}]", group_name)?;
                
                let mut keys = mastery_groups.get(&mastery_id).unwrap().clone();
                keys.sort_by_key(|k| k.parse::<u32>().unwrap_or(0));

                for node_id_str in keys {
                    if let Some(node_data) = nodes.get(node_id_str) {
                        let p = selected_pts
                            .and_then(|s| s.get(node_id_str))
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);

                        if let Ok(node_id) = node_id_str.parse::<u8>() {
                        let mut name = resolver.get_passive_name(class_id, node_id);
                        if name.starts_with("Skills.") {
                            if let Some(node_name_key) = node_data.get("nodeNameKey").and_then(|v| v.as_str()) {
                                name = resolver.get_skill_name_bypassing(node_name_key); // I will create this resolver method
                            }
                        }

                        let max_pts = node_data.get("maxPoints").and_then(|v| v.as_u64()).unwrap_or(0);
                        
                        writeln!(file, "  - Name: \"{}\"", name)?;
                        writeln!(file, "    AllocatedPoints: {}", p)?;
                        writeln!(file, "    MaxPoints: {}", max_pts)?;
                        
                        let mut reqs = Vec::new();
                        let mastery_req = node_data.get("masteryRequirement").and_then(|v| v.as_u64()).unwrap_or(0);
                        if mastery_req > 0 {
                            reqs.push(format!("Requires {} points in mastery", mastery_req));
                        }
                        if let Some(r_arr) = node_data.get("requirements").and_then(|v| v.as_array()) {
                            for r in r_arr {
                                if let Some(r_id) = r.get("nodeId").and_then(|v| v.as_u64()) {
                                    let req_pts = r.get("requirement").and_then(|v| v.as_u64()).unwrap_or(0);
                                    let mut r_name = format!("Node {}", r_id);
                                    if let Some(req_node_data) = nodes.get(&r_id.to_string()) {
                                        r_name = resolver.get_passive_name(class_id, r_id as u8);
                                        if r_name.starts_with("Skills.") {
                                            if let Some(node_name_key) = req_node_data.get("nodeNameKey").and_then(|v| v.as_str()) {
                                                r_name = resolver.get_skill_name_bypassing(node_name_key);
                                            }
                                        }
                                    }
                                    reqs.push(format!("Requires {} pts in {}", req_pts, r_name));
                                }
                            }
                        }
                        
                        let mut scaling = Vec::new();
                        let mut breakpoints = Vec::new();

                        if let Some(stats) = node_data.get("stats").and_then(|v| v.as_array()) {
                            for stat in stats {
                                let no_scaling = stat.get("noScaling").and_then(|v| v.as_bool()).unwrap_or(false);
                                let val = stat.get("value").and_then(|v| v.as_str()).unwrap_or("?");
                                let mut effect_desc = format!("{} to property #{}", val, stat.get("property").and_then(|v| v.as_u64()).unwrap_or(0));
                                
                                if let Some(stat_key) = stat.get("statNameKey").and_then(|v| v.as_str()) {
                                    let resolved_stat = resolver.get_skill_name_bypassing(stat_key);
                                    if val == "?" {
                                        effect_desc = resolved_stat;
                                    } else {
                                        effect_desc = format!("{} {}", val, resolved_stat);
                                    }
                                }
                                
                                if no_scaling {
                                    let threshold = stat.get("noScalingPointThreshold").and_then(|v| v.as_u64()).or_else(|| node_data.get("noScalingPointThreshold").and_then(|v| v.as_u64())).unwrap_or(max_pts);
                                    breakpoints.push(format!("At {} pts: {}", threshold, effect_desc));
                                } else {
                                    scaling.push(format!("{} per point", effect_desc));
                                }
                            }
                        }

                        if !reqs.is_empty() {
                            writeln!(file, "    Requirements: \"{}\"", reqs.join(", "))?;
                        } else {
                            writeln!(file, "    Requirements: None")?;
                        }
                        if !scaling.is_empty() {
                            writeln!(file, "    ScalingEffects:")?;
                            for sc in &scaling { writeln!(file, "      - \"{}\"", sc)?; }
                        }
                        if !breakpoints.is_empty() {
                            writeln!(file, "    Breakpoints:")?;
                            for br in &breakpoints { writeln!(file, "      - \"{}\"", br)?; }
                        }
                        writeln!(file, "")?;
                    }
                }
            }
            }
        } else {
            // fallback
            if let Some(selected) = selected_pts {
                for (node_id_str, points) in selected {
                    writeln!(file, "  - Node {} (Points: {})", node_id_str, points)?;
                }
            }
        }
    }
    Ok(())
}

fn write_equipment(file: &mut File, json: &Value, resolver: &Resolver) -> Result<()> {
    if let Some(equipment) = json.get("data").and_then(|d| d.get("equipment").and_then(|e| e.as_object())) {
        for (slot, item) in equipment {
            writeln!(file, "Slot: {}", slot)?;
            if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                let item_name = resolver.get_item_name(id);
                if let Some(type_name) = resolver.get_item_type_name(id) {
                    writeln!(file, "  Item: {} ({})", item_name, type_name)?;
                } else {
                    writeln!(file, "  Item: {}", item_name)?;
                }
                
                let mut ir: Vec<u8> = Vec::new();
                let mut is_unique = false;
                if let Some(ir_arr) = item.get("ir").and_then(|v| v.as_array()) {
                    ir = ir_arr.iter().map(|v| v.as_u64().unwrap_or(0) as u8).collect();
                    let unique_details = resolver.get_unique_detail(id, &ir);
                    
                    if !unique_details.is_empty() {
                        is_unique = true;
                        writeln!(file, "  Stats:")?;
                        for detail in unique_details {
                            writeln!(file, "    - {}", detail)?;
                        }
                    }
                }

                if !is_unique {
                    let implicits = resolver.get_item_implicits(id, &ir);
                    if !implicits.is_empty() {
                         writeln!(file, "  Implicits: {}", implicits)?;
                    }
                }
            }
            let mut affixes_to_write = Vec::new();
            if let Some(arr) = item.get("affixes").and_then(|a| a.as_array()) {
                for affix in arr {
                    affixes_to_write.push((affix, ""));
                }
            }
            if let Some(sealed) = item.get("sealedAffix") {
                affixes_to_write.push((sealed, " (Sealed)"));
            }
            if let Some(corrupted) = item.get("corruptedAffix") {
                affixes_to_write.push((corrupted, " (Experimental)"));
            }

            if !affixes_to_write.is_empty() {
                writeln!(file, "  Affixes:")?;
                for (affix, tag) in &affixes_to_write {
                    if let Some(id_str) = affix.get("id").and_then(|v| v.as_str()) {
                        let tier = affix.get("tier").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
                        let roll = affix.get("r").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                        
                        let detail = resolver.get_affix_detail(id_str, tier, roll);
                        if !detail.is_empty() {
                            writeln!(file, "    - {}{}", detail, tag)?;
                        } else {
                             let name = resolver.get_affix_name(id_str);
                             writeln!(file, "    - [T{}]{} {}", tier, tag, name)?;
                        }
                    }
                }
            }
            writeln!(file, "")?;
        }
    } else {
        writeln!(file, "No equipment data found.")?;
    }
    Ok(())
}

fn write_idols(file: &mut File, json: &Value, resolver: &Resolver) -> Result<()> {
    if let Some(idols) = json.get("data").and_then(|d| d.get("idols").and_then(|i| i.as_array())) {
        if idols.is_empty() {
            writeln!(file, "No idols equipped in the Idol Altar.")?;
            return Ok(());
        }
        
        for (idx, idol) in idols.iter().enumerate() {
            if let Some(id) = idol.get("id").and_then(|v| v.as_str()) {
                let name = resolver.get_item_name(id);
                if let Some(type_name) = resolver.get_item_type_name(id) {
                    writeln!(file, "Slot {}: {} ({})", idx + 1, name, type_name)?;
                } else {
                    writeln!(file, "Slot {}: {}", idx + 1, name)?;
                }
                
                if let Some(affixes) = idol.get("affixes").and_then(|a| a.as_array()) {
                    for affix in affixes {
                        if let Some(aff_id) = affix.get("id").and_then(|a| a.as_str()) {
                            let tier = affix.get("tier").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
                            let roll = affix.get("r").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                            let detail = resolver.get_affix_detail(aff_id, tier, roll);
                            if !detail.is_empty() {
                                writeln!(file, "    - {}", detail)?;
                            } else {
                                let aff_name = resolver.get_affix_name(aff_id);
                                writeln!(file, "    - [T{}] {}", tier, aff_name)?;
                            }
                        }
                    }
                }
                writeln!(file, "")?;
            }
        }
    } else {
        writeln!(file, "No idols data found.")?;
    }
    Ok(())
}

fn write_blessings(file: &mut File, json: &Value, resolver: &Resolver) -> Result<()> {
    const TIMELINES: [&str; 10] = [
        "Fall of the Outcasts",
        "The Stolen Lance",
        "The Black Sun",
        "Blood, Frost, and Death",
        "Ending the Storm",
        "Fall of the Empire",
        "Reign of Dragons",
        "The Last Ruin",
        "Age of Winter",
        "Spirits of Fire",
    ];

    if let Some(blessings) = json.get("data").and_then(|d| d.get("blessings").and_then(|b| b.as_object())) {
        for i in 1..=10 {
            let key = i.to_string();
            let timeline_name = TIMELINES[i - 1];
            if let Some(blessing) = blessings.get(&key) {
                if blessing.is_null() || blessing.as_object().is_none() {
                    writeln!(file, "{}: No Blessing", timeline_name)?;
                    continue;
                }
                
                if let Some(id) = blessing.get("id").and_then(|v| v.as_str()) {
                    let mut ir: Vec<u8> = Vec::new();
                    if let Some(ir_arr) = blessing.get("ir").and_then(|v| v.as_array()) {
                        ir = ir_arr.iter().map(|v| v.as_u64().unwrap_or(0) as u8).collect();
                    }
                    let name = resolver.get_item_name(id);
                    writeln!(file, "{}: {}", timeline_name, name)?;
                    
                    let implicits = resolver.get_item_implicits(id, &ir);
                    if !implicits.is_empty() {
                        writeln!(file, "  - {}", implicits)?;
                    }
                } else {
                    writeln!(file, "{}: No Blessing", timeline_name)?;
                }
            } else {
                writeln!(file, "{}: No Blessing", timeline_name)?;
            }
        }
    } else {
         writeln!(file, "No blessings data found.")?;
    }
    
    Ok(())
}
