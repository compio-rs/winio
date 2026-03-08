# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.1](https://github.com/compio-rs/winio/compare/winio-ui-win32-v0.4.0...winio-ui-win32-v0.4.1) - 2026-03-08

### Added

- *(win32)* transparency for LinkLabel
- *(win)* log link opening
- *(win32)* add link label
- *(win32)* single selection listbox

### Fixed

- *(win32)* non-transparent LinkLabel bk color
- *(win32)* redraw immediately when WM_SETFONT
- *(win32,qt)* make LinkLabel center aligned
- *(win32)* color of LinkLabel
- *(win32)* rewrite LinkLabel based on Label
- *(win32)* handle uri manually
- *(win)* don't emit Click when uri is not empty
- *(win32)* check if uri is empty

### Other

- remove "authors" field in metadata
