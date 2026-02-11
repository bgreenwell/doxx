# Package Audit: doxx (February 9, 2026)

## Overview
doxx is a robust, performance-oriented terminal document viewer for `.docx` files. It features rich TUI rendering via `ratatui`, multiple export formats, and advanced document parsing (including equations and images).

## 1. Code Quality and Architecture

### **Current State**
*   **Monolithic Parser:** `src/document.rs` (~2,600 lines) handles data structures, DOCX parsing, numbering logic, search, and outline generation.
*   **XML Workarounds:** Utilizes `quick-xml` for OMML (equation) parsing to supplement `docx-rs` limitations.
*   **Stubs:** Configuration management (`ConfigCommands`) in `src/main.rs` contains stubs without implementation.

### **Recommendations**
*   **Refactor `document.rs`:** Break it into a module folder (`src/document/`) with sub-files for `parser.rs`, `numbering.rs`, `search.rs`, and `models.rs`.
*   **Implement Config:** Complete the TODOs in `src/main.rs` for persistent configuration (likely using `dirs` and `toml` crates already in `Cargo.toml`).
*   **Unify Image Logic:** Create a single rendering abstraction to bridge the gap between `viuer` (CLI mode) and `ratatui-image` (TUI mode).

## 2. Terminal UI and Rendering

### **Current State**
*   **Element-based Scrolling:** The TUI scrolls by element index rather than line. This causes "jumps" when navigating past large tables or paragraphs.
*   **Custom Widget Pattern:** `DocumentWidget` bypasses standard `ratatui` traits to facilitate `Frame` access for image rendering.

### **Recommendations**
*   **Line-based Scrolling:** Implement virtual line mapping within `DocumentWidget` to allow smooth scrolling through tall elements.
*   **Idiomatic Widgets:** Investigate if `ratatui-image` 8.0+ supports patterns that allow `DocumentWidget` to implement the `Widget` trait properly.

## 3. Features and Functionality

### **Current State**
*   **Numbering:** Excellent handling of hierarchical Word numbering via `DocumentNumberingManager`.
*   **Export:** Strong support for Markdown, CSV, JSON, and ANSI (including 16/256/TrueColor depth).
*   **Search:** Basic case-insensitive literal matching.

### **Recommendations**
*   **Search Enhancements:** Add Regex support and "Match Whole Word" toggles.
*   **Table Navigation:** Add a specific "Table View" mode or better horizontal scrolling for wide tables in the TUI.

## 4. Performance and Testing

### **Current State**
*   **Testing:** Strong integration test suite with comprehensive fixtures.
*   **Optimizations:** Good use of LTO and codegen-units for release builds.

### **Recommendations**
*   **Parallel Parsing:** Use `rayon` for heavy parsing tasks if document size becomes a performance bottleneck.
*   **Library Tests:** Add more unit tests in `src/document.rs` (or its future modules) that don't require spawning a full binary process.

## 5. Security and Maintenance

*   **Secrets:** No hardcoded secrets found.
*   **Dependencies:** Uses modern, well-maintained crates (`ratatui`, `tokio`, `clap`).
*   **Cleanup:** `ImageExtractor` temp files persist during the session; ensure a cleanup mechanism exists for long-running processes.

---
*Audit performed by Gemini CLI Agent.*
