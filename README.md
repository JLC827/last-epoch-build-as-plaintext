# Last Epoch Build As Plaintext

Extracts character stats, build, and equipment data from [Last Epoch Tools](https://www.lastepochtools.com/planner/) into plain text format. Useful for LLM build analysis.

## How to get your Build URL

1. Go to [https://www.lastepochtools.com/profile/](https://www.lastepochtools.com/profile/)
2. Enter your account name. *(You can check your account name in the Social Panel: press `H` in game to access it)*
3. Click **View Profile** and select the character you want to get build info for.
4. Click **View in Build Planner**.
5. On the left menu, click **Save/Share** and copy the generated link.
6. Use this link as the URL in the Run command below!

## Quick Start

1. **Install Prerequisites**: Download and install [Rust](https://rustup.rs/) and [Git](https://git-scm.com/).
2. **Get the Code**: Open a command prompt, clone the repository, and navigate into it:
   ```powershell
   git clone https://github.com/JLC827/last-epoch-build-as-plaintext.git
   cd last-epoch-build-as-plaintext
   ```
   *(Keep this command prompt open!)*
3. **Run the Scraper**:
   In the same command prompt from earlier, run the following command (replace `<YOUR_BUILD_URL>` with your pasted link):
   ```powershell
   cargo run "<YOUR_BUILD_URL>"
   # Example: cargo run "https://www.lastepochtools.com/planner/o3ZbjxKn"
   ```
4. **Done!**
   - The plain text build summary is automatically copied to your clipboard.
   - A copy is saved to the `builds/` directory.

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

Credit back to the repository is highly appreciated, but never legally required!
