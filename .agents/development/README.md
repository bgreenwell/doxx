# Development Documentation

Enhancement planning, architecture decisions, and feature analysis for doxx.

## Quick Navigation

### Planning & Strategy
- **[project-audit.md](project-audit.md)** - Current state analysis (Claude + Gemini)
- **[roadmap.md](roadmap.md)** - Feature prioritization and timeline

### Architecture
- **[architecture/document-widget.md](architecture/document-widget.md)** - Custom DocumentWidget design
- **[architecture/widget-integration.md](architecture/widget-integration.md)** - ratatui integration notes

### Enhancements (Planned Features)
- **[enhancements/keyboard-shortcuts.md](enhancements/keyboard-shortcuts.md)** - vim/less keybindings (#26)
- **[enhancements/table-cell-merge.md](enhancements/table-cell-merge.md)** - gridSpan support (#67)
- **[enhancements/position-memory.md](enhancements/position-memory.md)** - Remember last position (#66)
- **[enhancements/font-size-headings.md](enhancements/font-size-headings.md)** - Smart heading detection (#3)

### Infrastructure
- **[infrastructure/homebrew-automation.md](infrastructure/homebrew-automation.md)** - Homebrew tap automation
- **[infrastructure/fixture-cleanup.md](infrastructure/fixture-cleanup.md)** - Test fixture management

## How to Use This Directory

### Starting a New Feature
1. Read **project-audit.md** for current state
2. Check **roadmap.md** for prioritization
3. Read relevant enhancement doc in **enhancements/**
4. Review architectural constraints in **architecture/**
5. Implement with testing strategy from enhancement doc

### Understanding Architecture
- DocumentWidget pattern: See **architecture/document-widget.md**
- Integration with ratatui: See **architecture/widget-integration.md**

### Finding Enhancement Proposals
All planned features have individual docs in **enhancements/** with:
- Problem statement
- Proposed solution
- Implementation plan
- Testing strategy
- Success criteria

## Enhancement Doc Template

When creating new enhancement proposals, use this structure:

```markdown
# Enhancement: [Feature Name]

## Issue Reference
GitHub Issue: #XX

## Problem Statement
[User need / current limitation]

## Proposed Solution
[High-level approach]

## Implementation Plan
1. [Step 1]
2. [Step 2]

## Technical Considerations
- Dependencies: [crates needed]
- Files affected: [src/file.rs:line]
- Breaking changes: [yes/no]

## Testing Strategy
[Test cases needed]

## Success Criteria
- [ ] Acceptance criteria 1
- [ ] Acceptance criteria 2
```

## Related Documentation

- **Operations:** See ../operations/ for day-to-day workflows
- **Main Guide:** See ../../AGENTS.md for overview
- **Archive:** See ../archive/ for historical planning docs
