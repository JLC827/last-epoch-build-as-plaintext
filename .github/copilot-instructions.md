# Copilot / AI Agent Instructions — letools_scraper

Purpose
This rust project is a web scraper to extract stats, builds, and equipment data from Last Epoch Tools. LE Tools is a popular third-party website for the game Last Epoch that provides a build planner and item database, as well as builds. It is a single page application built with React, that dynamically loads content via JavaScript. Use headless chromium to render the pages and extract the relevant data, and output the data to a plaintext file.

The output should include the following features:
Character Stats Extraction: Extract character stats such as level, class, attributes (strength, dexterity, intelligence, vitality), resistances, and other relevant stats displayed on the build planner page.
Skills and Passives Extraction: Extract information about the skills and passive skills selected in the build planner, including skill levels, nodes chosen in the passive skill tree, and any modifiers applied to skills.
Equipment Extraction: Extract details about the equipment items equipped on the character, including item names, types, rarities, affixes, and any special properties or effects.

Use https://www.lastepochtools.com/planner/AL0aE1k4 as a test url for generating and validating the scraper output.


Coding requirements:
fail quick, fail fast. KISS principle. 
You work with text files, so saving of files to allow you to parse output is encouraged, such as to help debug HTML content or test that data is being retrieved correctly.
Be aware of rate limiting and implement delays or retries as needed to avoid being blocked by the target website, if needed. Watch out for cloudlflare blocks.
We seem to be getting inconsistent results with headless chromium, so keep that in mind if something works then later does not. Consider adding logging to help debug such issues. Also, while we are in the development phase, consider keeping code, rather than deleting it, if it might help debug intermittent issues. Maybe using console args to allow running different scraper attempts.
The terminal is powershell on windows 11. Avoid using unix specific commands.

You can use curl to fetch JS files if needed to help parse out data.
e.g. "curl -o hashed_js/planner_d0feed.js https://www.lastepochtools.com/data/version135/planner/js/d0feedd12833161e6575dc3d36021eab.js;"

Don't delete main.rs. Make changes as needed instead or add new files.

### Rust Architecture: Library + Binaries Pattern

To ensure code reusability across the main application and standalone sub-system scripts, follow this structure:

#### 1. Directory Structure
```text
├── Cargo.toml
├── src/
│   ├── lib.rs            # Main library (shared logic)
│   ├── main.rs           # Primary application entry point
│   ├── sub_logic/        # Feature-specific modules
│   │   └── mod.rs
│   └── bin/              # Standalone executable scripts
│       └── sub_system.rs # Runs specific sub-system logic
```

#### 2. Setup Logic (`src/lib.rs`)
Expose your modules so both `main.rs` and `bin/` files can access them:
```rust
// src/lib.rs
pub mod sub_logic; 
```

#### 3. Main Application (`src/main.rs`)
```rust
// src/main.rs
use your_project_name::sub_logic;

fn main() {
    sub_logic::run(); // Call shared logic
}
```

#### 4. Standalone Sub-system (`src/bin/sub_system.rs`)
Every file in `src/bin/` is a separate executable.
```rust
// src/bin/sub_system.rs
use your_project_name::sub_logic;

fn main() {
    println!("Running standalone sub-system...");
    sub_logic::run(); 
}
```

#### 5. Commands
*   **Run Main App:** `cargo run`
*   **Run Sub-system:** `cargo run --bin sub_system`
*   **Test Sub-system:** Create files in `/tests` for integration testing.

#### Why this pattern?
*   **No Code Duplication:** Logic lives in `lib.rs` and is imported by all binaries.
*   **Isolation:** Sub-systems in `src/bin/` can be tested or debugged without running the full application.
*   **Idiomatic:** This is the standard Rust approach for multi-binary projects.