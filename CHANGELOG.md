# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Add parser struct
- Add logging support

### Changed

- Sanitize as filename
- Return the largest text on the pages
- Text encoding from fonts
- Collect positioned text
- Keep track of font size, leading and y
- Warn if page couldn't be parsed
- Warn user if pages are skipped
- Replace lopdf with pdf crate
- Parse Tj, TJ, " and '
- Parse Td, TD, Tm and T*
- Deduplicate parsing pdf operations
- Parse TL
- Parse BT and ET
- Keep track of the font size
- Iterate through the first N pages
- Make clippy happy
- Load pdf document
- Create README.md

### Fixed

- Reset text state for each text object
