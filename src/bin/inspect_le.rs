use anyhow::Result;
use headless_chrome::{Browser, LaunchOptions};
use std::fs;
use std::time::Duration;

fn main() -> Result<()> {
    let options = LaunchOptions {
        headless: true,
        args: vec![
            std::ffi::OsStr::new("--disable-web-security"),
            std::ffi::OsStr::new("--user-data-dir=C:\\temp\\le_chrome_profile"),
        ],
        ..Default::default()
    };
    let browser = Browser::new(options)?;
    let tab = browser.new_tab()?;
    
    println!("Navigating...");
    tab.navigate_to("https://www.lastepochtools.com/planner/BZ3ZagqY")?;
    tab.wait_until_navigated()?;
    
    println!("Waiting for page loads...");
    std::thread::sleep(Duration::from_secs(5));
    
    println!("Extracting keys of window.le_...");
    let keys_eval = tab.evaluate("JSON.stringify(Object.keys(window.le_ || {}))", false)?;
    if let Some(val) = keys_eval.value {
        if let Some(s) = val.as_str() {
            println!("Keys: {}", s);
        }
    }
    
    println!("Extracting window.le_.skillTreeData...");
    let tree_eval = tab.evaluate("JSON.stringify(Object.keys(window.le_.skillTreeData || {}))", false)?;
    if let Some(val) = tree_eval.value {
        if let Some(s) = val.as_str() {
            println!("skillTreeData Keys: {}", s);
        }
    }

    println!("Extracting full window.le_.skillTreeData...");
    let full_tree = tab.evaluate("JSON.stringify(window.le_.skillTreeData || {})", false)?;
    if let Some(val) = full_tree.value {
        if let Some(s) = val.as_str() {
            fs::write("le_skillTreeData.json", s)?;
            println!("Saved le_skillTreeData.json");
        }
    }
    
    println!("Extracting window.le_.classTreeData...");
    let class_tree = tab.evaluate("JSON.stringify(window.le_.classTreeData || {})", false)?;
    if let Some(val) = class_tree.value {
        if let Some(s) = val.as_str() {
            fs::write("le_classTreeData.json", s)?;
            println!("Saved le_classTreeData.json");
        }
    }
    
    Ok(())
}
