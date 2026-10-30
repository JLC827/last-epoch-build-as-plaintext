# Last Epoch Build As Plaintext

Extracts character stats, build, and equipment data from [Last Epoch Tools](https://www.lastepochtools.com/planner/) into plain text format. Useful for sharing, searching, or for LLM build analysis.

## Supported Build URLs

1. Go to [https://www.lastepochtools.com/profile/](https://www.lastepochtools.com/profile/)
2. Enter your account name. *(You can check your account name in the Social Panel: press `H` in game to access it)*
3. Open the character profile you want to scrape.
4. Pass either the character profile URL or its Build Planner URL to the scraper. For a profile URL, the scraper follows **View in Build Planner** automatically.

## Quick Start

1. **Install Prerequisites**: Download and install [Rust](https://rustup.rs/) and [Git](https://git-scm.com/).
2. **Get the Code**: Open a command prompt, clone the repository, and navigate into it:
   ```powershell
   git clone https://github.com/JLC827/last-epoch-build-as-plaintext.git
   cd last-epoch-build-as-plaintext
   ```
   *(Keep this command prompt open)*
3. **Run the Scraper**:
   In the same command prompt from earlier, run the following command with either a character profile URL or a planner URL:
   ```powershell
   cargo run "https://www.lastepochtools.com/profile/Bingus/character/Bongus"
   # A planner URL also works: cargo run "https://www.lastepochtools.com/planner/QkYRnrdw"
   ```
4. **Done**
   - The plain text build summary is automatically copied to your clipboard.
   - A copy is saved to the `builds/` directory.

---

## Scraping All Idol Data
You can also scrape the entire database of idols, including their prefixes, suffixes, and item details. This command does not require a build URL (the two are mutually exclusive).
- **Execution**: `cargo run -- --idols`
- **Output**: Generates a comprehensive plaintext file at `builds/idols.txt` aggregating all idol sizes, drop conditions, and affix data.

---

## Features
- **Character Stats Extraction**: Pulls level, class, attributes, resistances, defenses, and general stats.
- **Skills and Passives Extraction**: Extracts active skills, nodes on the passive skill tree, and skill modifiers. Skill tree requirements and passive bonus breakpoints are also listed.
- **Equipment Extraction**: Identifies equipped items, including item names, types, rarities, affixes, and special effects.

## How it Works
The scraper uses the `headless_chrome` crate to automate a hidden browser instance.
1. Navigates to a target Build Planner URL.
2. Waits for React to load and render the page.
3. Serializes and extracts internal JSON states (e.g., `window.buildInfo`, `window.itemDB`).
4. Performs an authenticated in-page `fetch()` to retrieve translation databases.
5. Saves raw diagnostic data to `debug_data/` and writes a human-readable text file to `builds/`.

## Repository

[![GitHub repo](https://img.shields.io/badge/GitHub-JLC827%2Flast--epoch--build--as--plaintext-blue?logo=github)](https://github.com/JLC827/last-epoch-build-as-plaintext)

## License
This project is released into the public domain under **The Unlicense**. You are entirely free to copy, modify, distribute, or use this for any purpose without seeking permission.

Credit back to the repository is highly appreciated, but never legally required.
