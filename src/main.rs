use anyhow::Result;
use clap::Parser;
use headless_chrome::{Browser, LaunchOptions};
use std::fs::File;
use std::io::Write;
use std::time::Duration;
use serde_json::Value;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// URL of the build planner to scrape
    #[arg(short, long, default_value = "https://www.lastepochtools.com/planner/AL0aE1k4")]
    url: String,

    /// Output file path
    #[arg(short, long, default_value = "build_data.txt")]
    output: String,
}

fn main() -> Result<()> {
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

    println!("Page loaded. Extracting data...");

    // Probe 8: Fetch language files
    let probe8_script = r#"
        (async () => {
            let result = {};
            try {
                let siteMarker = window['langSiteCacheMarker'] || '96';
                let url = '/static_data/i18n/en.json?' + siteMarker;
                result.url = url;
                
                let resp = await fetch(url);
                if (resp.ok) {
                    result.en_json = await resp.json();
                    result.status = "success";
                } else {
                    result.status = "failed";
                    result.http_code = resp.status;
                    
                    // Try fallback
                    let fallbackUrl = '/data/i18n_fallback/en.json';
                    let resp2 = await fetch(fallbackUrl);
                    if (resp2.ok) {
                        result.en_json = await resp2.json();
                        result.status = "success_fallback";
                    } else {
                        result.fallback_status = resp2.status;
                    }
                }
            } catch (e) {
                result.status = "error";
                result.error = e.toString();
            }
            return JSON.stringify(result);
        })()
    "#;

    println!("Running Probe 8 (Language Fetch)...");
    let probe8_result = tab.evaluate(probe8_script, true)?;
    if let Some(val) = probe8_result.value {
        if let Some(s) = val.as_str() {
             let mut file = File::create("probe8_result.json")?;
             file.write_all(s.as_bytes())?;
             println!("Probe 8 result written to probe8_result.json");
        } else {
            println!("Probe 8 returned non-string: {:?}", val);
        }
    } else {
        println!("Probe 8 returned no value");
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
    
    // The value returned by evaluate is a serde_json::Value. 
    // If it's a string (JSON.stringify result), we need to unquote it? 
    // Actually headless_chrome returns the value as is. 
    // If the script returns a string, value is a String.
    // Wait, I returned JSON.stringify(buildInfo), so it's a string.
    
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
                    // We could list selected nodes here if needed
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

// Removed unused functions
