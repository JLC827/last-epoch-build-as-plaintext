use anyhow::Result;
use clap::Parser;
use headless_chrome::{Browser, LaunchOptions};
use std::fs::File;
use std::io::Write;
use std::time::Duration;
use serde_json::Value;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// URL of the build planner to scrape
    #[arg(short, long, default_value = "https://www.lastepochtools.com/planner/AL0aE1k4")]
    pub url: String,

    /// Output file path
    #[arg(short, long, default_value = "build_data.txt")]
    pub output: String,
}

pub fn run() -> Result<()> {
    let args = Args::parse();

    println!("Scraping URL: {}", args.url);
    
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
    
    // Navigate to the URL
    tab.navigate_to(&args.url)?;
    
    // Wait for the page to load
    println!("Waiting for page to load...");
    tab.wait_for_element("body")?;
    
    // Give React some time to render the dynamic content
    std::thread::sleep(Duration::from_secs(5));

    println!("Page loaded. Starting extraction...");

    // Probe 9: Dump window.le_
    println!("Dumping window.le_...");
    let probe9_script = r#"
        (function() {
            const seen = new WeakSet();
            return JSON.stringify(window.le_, (key, value) => {
                if (typeof value === "object" && value !== null) {
                    if (seen.has(value)) {
                        return;
                    }
                    seen.add(value);
                }
                return value;
            });
        })()
    "#;
    let probe9_res = tab.evaluate(probe9_script, false);
    if let Ok(res) = probe9_res {
        if let Some(val) = res.value {
            if let Some(s) = val.as_str() {
                let mut file = File::create("le_dump.json")?;
                file.write_all(s.as_bytes())?;
                println!("Saved le_dump.json");
            }
        }
    }

    // 1. Extract Translations
    println!("Extracting translations...");
    let translation_script = r#"
        (async () => {
            const version = 'version135';
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
            let mut file = File::create("translations.json")?;
            file.write_all(s.as_bytes())?;
            println!("Saved translations.json");
        }
    }

    // 2. Extract LEAbilities
    println!("Extracting LEAbilities...");
    let abilities_res = tab.evaluate("JSON.stringify(window.LEAbilities || {})", false)?;
    if let Some(val) = abilities_res.value {
        if let Some(s) = val.as_str() {
            let mut file = File::create("le_abilities.json")?;
            file.write_all(s.as_bytes())?;
            println!("Saved le_abilities.json");
        }
    }

    // 3. Extract coreDB
    println!("Extracting coreDB...");
    let coredb_res = tab.evaluate("JSON.stringify(window.coreDB || {})", false)?;
    if let Some(val) = coredb_res.value {
        if let Some(s) = val.as_str() {
            let mut file = File::create("core_db.json")?;
            file.write_all(s.as_bytes())?;
            println!("Saved core_db.json");
        }
    }

    // 4. Extract itemDB
    println!("Extracting itemDB...");
    let itemdb_res = tab.evaluate("JSON.stringify(window.itemDB || {})", false)?;
    if let Some(val) = itemdb_res.value {
        if let Some(s) = val.as_str() {
            let mut file = File::create("item_db.json")?;
            file.write_all(s.as_bytes())?;
            println!("Saved item_db.json");
        }
    }

    let build_info = extract_build_info(&tab)?;
    let build_json: Value = serde_json::from_str(&build_info)?;

    let mut file = File::create(&args.output)?;
    writeln!(file, "Build Data for {}\n", args.url)?;
    
    writeln!(file, "--- Character Stats ---")?;
    write_stats(&mut file, &build_json)?;
    
    writeln!(file, "\n--- Skills & Passives ---")?;
    write_skills(&mut file, &build_json)?;
    
    writeln!(file, "\n--- Equipment ---")?;
    write_equipment(&mut file, &build_json)?;

    println!("Data saved to {}", args.output);

    Ok(())
}

fn extract_build_info(tab: &headless_chrome::Tab) -> Result<String> {
    let script = r#"
        (function() {
            const buildInfo = window.buildInfo;
            if (!buildInfo) return null;
            
            // Parse precalc_data if it's a string
            if (typeof buildInfo.precalc_data === 'string') {
                try {
                    buildInfo.precalc_data = JSON.parse(buildInfo.precalc_data);
                } catch (e) {
                    // ignore
                }
            }
            
            return JSON.stringify(buildInfo);
        })()
    "#;
    
    let remote_object = tab.evaluate(script, false)?;
    let value = remote_object.value.ok_or_else(|| anyhow::anyhow!("No value returned from script"))?;
    
    if value.is_null() {
        return Err(anyhow::anyhow!("buildInfo not found on page"));
    }
    
    if let Some(s) = value.as_str() {
        Ok(s.to_string())
    } else {
        Ok(value.to_string())
    }
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

fn write_skills(file: &mut File, json: &Value) -> Result<()> {
    if let Some(data) = json.get("data") {
        if let Some(hud) = data.get("hud").and_then(|h| h.as_array()) {
            writeln!(file, "Active Skills (HUD):")?;
            for skill in hud {
                writeln!(file, "  - {}", skill)?;
            }
        }
        
        if let Some(trees) = data.get("skillTrees").and_then(|t| t.as_array()) {
            writeln!(file, "\nSkill Trees Configured:")?;
            for tree in trees {
                if let Some(id) = tree.get("treeID") {
                    writeln!(file, "  - ID: {}", id)?;
                }
            }
        }
    } else {
        writeln!(file, "No skills data found.")?;
    }
    Ok(())
}

fn write_equipment(file: &mut File, json: &Value) -> Result<()> {
    if let Some(equipment) = json.get("data").and_then(|d| d.get("equipment").and_then(|e| e.as_object())) {
        for (slot, item) in equipment {
            writeln!(file, "Slot: {}", slot)?;
            if let Some(id) = item.get("id") {
                writeln!(file, "  Item ID: {}", id)?;
            }
            if let Some(affixes) = item.get("affixes").and_then(|a| a.as_array()) {
                writeln!(file, "  Affixes:")?;
                for affix in affixes {
                    let id = affix.get("id").unwrap_or(&Value::Null);
                    let tier = affix.get("tier").unwrap_or(&Value::Null);
                    let range = affix.get("r").unwrap_or(&Value::Null);
                    writeln!(file, "    - ID: {}, Tier: {}, Roll: {}", id, tier, range)?;
                }
            }
            writeln!(file, "")?;
        }
    } else {
        writeln!(file, "No equipment data found.")?;
    }
    Ok(())
}
