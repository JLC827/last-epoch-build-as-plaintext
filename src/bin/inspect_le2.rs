use anyhow::Result;
use headless_chrome::{Browser, LaunchOptions};
use std::fs;
use std::time::Duration;

fn main() -> Result<()> {
    let options = LaunchOptions {
        headless: true,
        args: vec![
            std::ffi::OsStr::new("--disable-web-security"),
        ],
        ..Default::default()
    };
    let browser = Browser::new(options)?;
    let tab = browser.new_tab()?;
    println!("Navigating...");
    tab.navigate_to("https://www.lastepochtools.com/planner/BZ3ZagqY")?;
    tab.wait_until_navigated()?;
    println!("Wait 5s...");
    std::thread::sleep(Duration::from_secs(5));
    
    println!("Finding window keys...");
    let res = tab.evaluate(r#"
        JSON.stringify(Object.keys(window).filter(k => 
            k.toLowerCase().includes('tree') || 
            k.toLowerCase().includes('db') || 
            k.toLowerCase().includes('le') || 
            k.toLowerCase().includes('skill') ||
            k.toLowerCase().includes('passive') ||
            k.toLowerCase().includes('class')
        ))
    "#, false)?;
    
    if let Some(val) = res.value {
        if let Some(s) = val.as_str() {
            fs::write("window_keys.json", s)?;
            println!("Keys written");
        }
    }
    
    Ok(())
}
