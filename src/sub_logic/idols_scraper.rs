use anyhow::Result;
use headless_chrome::{Browser, LaunchOptions};
use std::time::Duration;
use std::fs::File;
use std::io::Write;

pub fn scrape_idols() -> Result<()> {
    println!("Scraping idols...");

    std::fs::create_dir_all("builds").ok();
    
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
    let categories = [
        "idols11", "idols11_2", "idols21", "idols12", 
        "idols31", "idols13", "idols41", "idols14", "idols22"
    ];

    let mut out_file = File::create("builds/idols.txt")?;

    for cat in categories {
        for sub_type in ["items", "prefixes", "suffixes"] {
            let url = format!("https://www.lastepochtools.com/db/category/{}/{}", cat, sub_type);
            println!("Navigating to {}", url);
            tab.navigate_to(&url)?;
            
            // Wait for items to appear or page body
            match tab.wait_for_element(".main-container") {
                Ok(_) => {
                    std::thread::sleep(Duration::from_secs(3)); // Give it time to load dynamic table
                },
                Err(_) => {
                    println!("Warning: Body not found on page {}", url);
                }
            }

            let extract_js = r#"
                (function() {
                    try {
                        let results = [];
                        let itemBoxes = document.querySelectorAll('.item-card, table tr.item, .db-list-item');
                        
                        if (itemBoxes.length === 0) {
                            return "No items found. Page text preview: " + document.body.innerText.substring(0, 100).replace(/\n/g, ' ');
                        }

                        for (let el of itemBoxes) {
                            let nameEl = el.querySelector('.item-name, td:first-child');
                            let name = nameEl ? nameEl.innerText.trim() : "Unknown";
                            
                            // Get text and replace all newlines with a separator
                            let textContent = el.innerText.trim().replace(/\n+/g, ' | ');
                            results.push(`- ${textContent}`);
                        }
                        
                        // Deduplicate a bit if table matching was messy
                        let unique = [...new Set(results)];
                        return unique.join('\n');
                    } catch(e) {
                         return "Error: " + e.message;
                    }
                })()
            "#;

            let res = tab.evaluate(extract_js, false)?;
            let text = if let Some(val) = res.value {
                val.as_str().unwrap_or("No string returned").to_string()
            } else {
                "No data returned".to_string()
            };
            
            writeln!(out_file, "=== {}/{} ===\n{}\n", cat, sub_type, text)?;
        }
    }

    println!("Saved to builds/idols.txt");
    Ok(())
}
