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