# .agents/ Directory

AI agent resources for doxx development - detailed documentation to keep AGENTS.md concise.

## Quick Start

- **New to the project?** Read [../AGENTS.md](../AGENTS.md) first
- **Working on a feature?** Check [development/enhancements/](development/enhancements/)
- **Debugging CI?** See [operations/ci-troubleshooting.md](operations/ci-troubleshooting.md)
- **Preparing release?** See [operations/release.md](operations/release.md)
- **Current priorities?** Read [development/project-audit.md](development/project-audit.md)

## Directory Structure

### development/
Enhancement planning, architecture decisions, and feature analysis

```
development/
├── README.md                          # Development docs index
├── project-audit.md                   # Current state (Claude + Gemini)
├── roadmap.md                         # Feature prioritization
├── architecture/                      # Design patterns
│   ├── document-widget.md            # Custom DocumentWidget
│   └── widget-integration.md         # ratatui integration
├── enhancements/                      # Feature proposals
│   ├── keyboard-shortcuts.md         # #26 - vim/less keybindings
│   ├── table-cell-merge.md           # #67 - gridSpan support
│   ├── position-memory.md            # #66 - remember position
│   └── font-size-headings.md         # #3 - smart heading detection
└── infrastructure/                    # DevOps & tooling
    ├── homebrew-automation.md        # Homebrew tap
    └── fixture-cleanup.md            # Test management
```

### operations/
Day-to-day development procedures and current constraints

```
operations/
├── known-issues.md                    # Active bugs (#58, #45, #24)
├── ci-troubleshooting.md             # Fix common CI/CD failures
├── workflows.md                       # Cargo commands & examples
├── performance.md                     # Benchmarks & profiling
└── release.md                         # Release process steps
```

### archive/
Historical planning documents and completed work

```
archive/
└── audit-2024.md                      # Previous audit (reference)
```

## Navigation Guide

### I want to...

**...understand current priorities**
→ Read [development/project-audit.md](development/project-audit.md) and [development/roadmap.md](development/roadmap.md)

**...implement a planned feature**
→ Check [development/enhancements/](development/enhancements/) for proposal docs

**...fix a CI/CD failure**
→ See [operations/ci-troubleshooting.md](operations/ci-troubleshooting.md)

**...understand architecture decisions**
→ Read [development/architecture/](development/architecture/)

**...prepare a release**
→ Follow [operations/release.md](operations/release.md)

**...optimize performance**
→ See [operations/performance.md](operations/performance.md) for benchmarks

**...run manual tests**
→ Use commands in [operations/workflows.md](operations/workflows.md)

## About This Directory

**Purpose:** Detailed reference material to keep AGENTS.md concise and focused

**Status:** Gitignored - local agent resources only

**Maintenance:** Update as project evolves

**Organization:** Three categories
- **development/** - Forward-looking (planning, enhancements, architecture)
- **operations/** - Current state (procedures, constraints, workflows)
- **archive/** - Historical context (completed work, old plans)

## Contributing

When adding new documentation:
1. Choose appropriate subdirectory (development vs operations vs archive)
2. Use clear, descriptive filenames
3. Follow enhancement doc template for new features
4. Update this README if adding new categories
5. Link from AGENTS.md if essential for all agents

---

Start with [../AGENTS.md](../AGENTS.md) for essential guidance, then dive into specific docs here as needed.
