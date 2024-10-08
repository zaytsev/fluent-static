# Changelog
All notable changes to this project will be documented in this file. See [conventional commits](https://www.conventionalcommits.org/) for commit guidelines.

- - -
## fluent-static-codegen-v0.5.0 - 2024-08-19
#### Features
- Allow compile-time FluentBundle configuration - (b7aefb2) - zaytsev

- - -

## fluent-static-codegen-v0.4.1 - 2024-08-16
#### Bug Fixes
- Use fully qualified dependency paths in generate! macro - (7b5ec46) - zaytsev
#### Documentation
- Update README to demonstrate generate! macro usage - (122a554) - zaytsev

- - -

## fluent-static-v0.4.0 - 2024-08-08
#### Features
- Add macro to include generated source code - (d45d6d3) - zaytsev

- - -

## fluent-static-codegen-v0.4.0 - 2024-08-08
#### Bug Fixes
- Disable all features by default - (54f6a95) - zaytsev
#### Build system
- Allow independent releases of codegen and main lib - (ff60632) - zaytsev
#### Documentation
- Add readme to crates packages - (0e059da) - zaytsev
#### Features
- Make axum language extraction more flexible - (4d48033) - zaytsev
- Add optional support for maud Render and axum accept-language extractor - (35fbb93) - zaytsev
#### Miscellaneous Chores
- **(version)** fluent-static-codegen-v0.4.0 - (97c33a8) - *zaytsev*
- Add cargo package metadata - (8d7a5b2) - zaytsev
- Initial commit - (bedb557) - zaytsev
#### Refactoring
- Refactor message function code generation - (7a11596) - zaytsev

- - -

## fluent-static-codegen-v0.4.0 - 2024-08-08
#### Build system
- Allow independent releases of codegen and main lib - (ff60632) - zaytsev
#### Features
- **(codegen)** Add macro to simplify codegen - (eb6000b) - *zaytsev*

- - -

## v0.3.2 - 2024-08-08
#### Bug Fixes
- **(codegen)** Fix invalid code generated when variable occurs more than once - (956ba4e) - *zaytsev*

- - -

## v0.3.1 - 2024-08-06
#### Bug Fixes
- **(examples)** Fix examples build errors - (1855751) - *zaytsev*
#### Documentation
- Fix example code in README - (e18614a) - zaytsev
#### Features
- **(codegen)** Generate message functions for attributes - (4c56ef5) - *zaytsev*
- Keep message functions variables order as in fluent resource - (6143b79) - zaytsev

- - -

## v0.3.0 - 2024-08-06
#### Bug Fixes
- Re-run codegen on any changes in resource directory - (431da6d) - Nazar Mokrynskyi
#### Features
- **(codegen)** Add support for message attributes - (73ac722) - *zaytsev*
- **(codegen)** Simplify generated API  - (340fc6e) - Nazar Mokrynskyi
#### Miscellaneous Chores
- Bump flake inputs - (ac85eb8) - zaytsev

- - -

## v0.2.4 - 2024-06-06
#### Documentation
- Update crate versions referenced in README - (88bc2f5) - zaytsev
#### Features
- **(codegen)** Add support for message, term and function references - (e9ac878) - *zaytsev*
#### Miscellaneous Chores
- Try to make  to automatically update crate versions refernced in README - (86b62c2) - zaytsev

- - -

## v0.2.3 - 2024-06-05
#### Documentation
- Add readme to crates packages - (0e059da) - zaytsev
#### Miscellaneous Chores
- Bump examples dependencies - (68a3fce) - zaytsev
#### Tests
- Ensure that multiline messages are supported - (9d81bb5) - zaytsev

- - -

## v0.2.2 - 2024-06-05
#### Miscellaneous Chores
- disable warning: unused import: `FluentValue` with translation without args - (7cb48f7) - AlbanMinassian

- - -

## v0.2.1 - 2024-05-28
#### Miscellaneous Chores
- Fix examples build failing due to a version constraints - (6fa1bc8) - zaytsev
#### Performance Improvements
- Detect language once on message struct creation - (74344a8) - zaytsev

- - -

## v0.2.0 - 2024-05-26
#### Bug Fixes
- Use proper language naming in axum example - (9533532) - zaytsev
#### Features
- New code generator: create message bundle structs - (79ff0ca) - zaytsev
- Make axum language extraction more flexible - (4d48033) - zaytsev
#### Miscellaneous Chores
- Fix package publishing in cog post bump hook - (cdc2bcb) - zaytsev
- Fix cargo manifest keywords - (bf16ffa) - zaytsev
#### Refactoring
- Refactor codegen - (10ea355) - zaytsev

- - -

## v0.1.0 - 2024-05-24
#### Bug Fixes
- Fix invalid message format in example project - (f89b0b0) - zaytsev
- Disable all features by default - (54f6a95) - zaytsev
#### Documentation
- Add webapp example to README - (0daa386) - zaytsev
- Add README - (39249cf) - zaytsev
#### Features
- Add compile time message bundle integrity check to prevent missing on incompatible messages - (2c633df) - zaytsev
- Add optional support for maud Render and axum accept-language extractor - (35fbb93) - zaytsev
#### Miscellaneous Chores
- Add cargo package metadata - (8d7a5b2) - zaytsev
- Fix cog bump config - (925ec5a) - zaytsev
- Add Axum webapp example - (abb5fb1) - zaytsev
- Rename generate crate - (c67fc1b) - zaytsev
- Add LICENSE - (693909d) - zaytsev
- Initial commit - (bedb557) - zaytsev
#### Refactoring
- Improve examples structure - (ea43a9c) - zaytsev
- Refactor message function code generation - (7a11596) - zaytsev
- Reorganize codegen exports - (0d56aee) - zaytsev

- - -

Changelog generated by [cocogitto](https://github.com/cocogitto/cocogitto).