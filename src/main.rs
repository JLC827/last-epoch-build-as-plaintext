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
        ..Default::default()
    })?;

    let tab = browser.new_tab()?;
    
    // Navigate to the URL
    tab.navigate_to(&args.url)?;
    
    // Wait for the page to load
    println!("Waiting for page to load...");
    tab.wait_for_element("body")?;
    
    // Give React some time to render the dynamic content
    // A better way would be to wait for a specific element that we know exists in the rendered app
    // But for now, a sleep is a simple way to ensure scripts have run.
    std::thread::sleep(Duration::from_secs(5));

    println!("Page loaded. Extracting data...");

    let stats = extract_stats(&tab)?;
    let skills = extract_skills(&tab)?;
    let equipment = extract_equipment(&tab)?;

    let mut file = File::create(&args.output)?;
    writeln!(file, "Build Data for {}\n", args.url)?;
    
    writeln!(file, "--- Character Stats ---")?;
    writeln!(file, "{}", stats)?;
    
    writeln!(file, "\n--- Skills & Passives ---")?;
    writeln!(file, "{}", skills)?;
    
    writeln!(file, "\n--- Equipment ---")?;
    writeln!(file, "{}", equipment)?;

    println!("Data saved to {}", args.output);

    Ok(())
}

fn extract_stats(tab: &headless_chrome::Tab) -> Result<String> {
    let script = r#"
        (function() {
            const stats = {};
            
            function findTextContent(text) {
                const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT, null, false);
                let node;
                while(node = walker.nextNode()) {
                    if(node.textContent.includes(text)) {
                        return node.parentElement.textContent.trim();
                    }
                }
                return "Not found";
            }

            stats.class = findTextContent("Class:");
            stats.level = findTextContent("Level:");
            
            // Try to find attributes
            // They are usually in a list. We'll look for specific attribute names.
            const attributes = ["Strength", "Dexterity", "Intelligence", "Attunement", "Vitality"];
            stats.attributes = {};
            
            attributes.forEach(attr => {
                // Find the element that contains exactly the attribute name
                const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT, null, false);
                let node;
                while(node = walker.nextNode()) {
                    if (node.textContent.trim() === attr) {
                        // The value is likely in a sibling or parent's other child
                        // Let's try to get the parent's text content
                        stats.attributes[attr] = node.parentElement.parentElement.textContent.trim();
                        break;
                    }
                }
            });

            // Resistances
            const resistances = ["Fire", "Lightning", "Cold", "Physical", "Poison", "Necrotic", "Void"];
            stats.resistances = {};
            resistances.forEach(res => {
                 const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT, null, false);
                let node;
                while(node = walker.nextNode()) {
                    if (node.textContent.trim() === res) {
                        stats.resistances[res] = node.parentElement.parentElement.textContent.trim();
                        break;
                    }
                }
            });

            return JSON.stringify(stats);
        })()
    "#;
    
    let remote_object = tab.evaluate(script, false)?;
    let result: Value = remote_object.value.unwrap();
    
    let mut output = String::new();
    if let Some(c) = result["class"].as_str() { output.push_str(&format!("{}\n", c)); }
    if let Some(l) = result["level"].as_str() { output.push_str(&format!("{}\n", l)); }
    
    output.push_str("\nAttributes:\n");
    if let Some(attrs) = result["attributes"].as_object() {
        for (k, v) in attrs {
            if let Some(val) = v.as_str() {
                // Clean up the string if it contains the label twice or extra info
                output.push_str(&format!("{}: {}\n", k, val));
            }
        }
    }

    output.push_str("\nResistances:\n");
    if let Some(res) = result["resistances"].as_object() {
        for (k, v) in res {
            if let Some(val) = v.as_str() {
                output.push_str(&format!("{}: {}\n", k, val));
            }
        }
    }
    
    Ok(output)
}

fn extract_skills(tab: &headless_chrome::Tab) -> Result<String> {
    // Skills are usually images with tooltips or text.
    // We'll look for the "Skills" section.
    let script = r#"
        (function() {
            const skills = [];
            // Strategy: Look for elements that might be skills.
            // In LE Tools, skills are often in a bar.
            // Let's try to find the "Skills" header and then look for siblings.
            
            // Fallback: Dump all text that looks like a skill name (hard to know without list).
            // Better: Look for the specific container.
            // Let's assume there are 5 specialized skills.
            
            // Try to find images that have 'skill' in their src or alt
            const images = document.querySelectorAll('img');
            const skillImages = [];
            images.forEach(img => {
                if (img.src.includes('skills') || img.alt.includes('Skill')) {
                    // This might be a skill icon.
                    // The name might be in the alt text or a sibling.
                    if (img.alt) skills.push(img.alt);
                }
            });
            
            // Also look for "Passives"
            // This is hard. Let's just return what we found.
            return JSON.stringify(skills);
        })()
    "#;
    
    let remote_object = tab.evaluate(script, false)?;
    let result: Value = remote_object.value.unwrap();
    
    let mut output = String::new();
    if let Some(arr) = result.as_array() {
        for item in arr {
            if let Some(s) = item.as_str() {
                output.push_str(&format!("- {}\n", s));
            }
        }
    }
    
    if output.is_empty() {
        output.push_str("No skills found (extraction logic needs refinement based on DOM).");
    }
    
    Ok(output)
}

fn extract_equipment(tab: &headless_chrome::Tab) -> Result<String> {
    // Equipment is usually in slots.
    // We can try to find the item names.
    // Items in LE Tools often have a specific class or structure.
    // Let's try to find all elements with a tooltip attribute or similar.
    
    let script = r#"
        (function() {
            const items = [];
            // Look for elements that look like items.
            // Often they are links or divs with background images.
            // Let's try to find text that matches known item rarities or types? No.
            
            // Let's try to find the equipment grid.
            // It's usually near the character model.
            
            // A generic approach: Find all elements that have a 'data-item-id' or similar?
            // Or just dump all text in the "Equipment" area if we can find it.
            
            // Let's try to find the "Equipment" header? It might not exist.
            
            // Let's look for the item slots by their typical names in the DOM if possible.
            // But we don't know them.
            
            // Let's try to find all images that look like items (e.g. /items/ in src)
            const images = document.querySelectorAll('img');
            images.forEach(img => {
                if (img.src.includes('/items/') || img.src.includes('/unique-items/')) {
                    // This is likely an item.
                    // Try to get the name from alt or title.
                    if (img.alt) items.push(img.alt);
                    else if (img.title) items.push(img.title);
                }
            });
            
            return JSON.stringify(items);
        })()
    "#;
    
    let remote_object = tab.evaluate(script, false)?;
    let result: Value = remote_object.value.unwrap();
    
    let mut output = String::new();
    if let Some(arr) = result.as_array() {
        for item in arr {
            if let Some(s) = item.as_str() {
                output.push_str(&format!("- {}\n", s));
            }
        }
    }
    
    if output.is_empty() {
        output.push_str("No equipment found (extraction logic needs refinement based on DOM).");
    }
    
    Ok(output)
}
