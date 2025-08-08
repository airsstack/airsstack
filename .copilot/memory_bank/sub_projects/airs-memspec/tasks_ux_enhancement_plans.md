# Tasks UX Enhancement Plans

**Created:** 2025-08-08  
**Context:** Task 013 Integration Testing revealed critical UX issues  
**Problem:** Current `tasks list` output shows 177 tasks - overwhelming and impractical for engineering teams  
**Objective:** Transform into focused, actionable tool for daily engineering workflow

## Current State Analysis

**Critical UX Issues Identified:**
- **Information Overload**: 177 tasks in one view (cognitive overload)
- **Poor Scanning**: Hard to find relevant tasks quickly
- **Mixed Context**: Tasks from different projects mixed together  
- **Verbose Details**: Too much detail for a list view
- **No Prioritization**: No way to focus on what matters most

**Current Output Structure:**
```
🚀 In Progress (4 tasks) - airs-mcp: 2, airs-memspec: 2
📋 Pending (40 tasks) - mixed projects  
✅ Completed (133 tasks) - historical data
```

---

## Development Plan 1: Smart Filtering by Default

**Objective**: Transform overwhelming 177-task list into focused, actionable view by default.

### Current State Analysis
- Default shows all 177 tasks (2 in-progress, 40 pending, 133 completed)
- No intelligence about what users actually need to see
- Cognitive overload prevents effective task management

### Implementation Strategy

#### Phase 1.1: Define Smart Default Rules
```yaml
Smart Default Logic:
  - Always show: All "In Progress" tasks
  - Priority show: "Pending" tasks from active project (from current_context.md)
  - Limit: Max 15 tasks total in default view
  - Sort order: In Progress first, then pending by recent activity
  - Exclude: Completed tasks (unless specifically requested)
```

#### Phase 1.2: Implement Default Filtering Logic
- Modify `tasks list` command to apply smart filters
- Read `current_context.md` to determine active project
- Create filtering pipeline: Status -> Project -> Limit -> Sort
- Preserve current behavior with `--all` flag

#### Phase 1.3: Add Contextual Hints
```
🎯 Showing active work for airs-memspec (use --all for complete view)
📊 Hidden: 38 pending tasks from other projects, 133 completed tasks
```

**Files to Modify:**
- `src/cli/commands/tasks.rs` - Add smart filtering logic
- `src/cli/args.rs` - Add `--all` flag
- `src/utils/task_filtering.rs` - New module for filtering logic

---

## Development Plan 2: Tiered Information Architecture

**Objective**: Provide multiple levels of detail - summary, focused, and comprehensive views.

### Architecture Design

#### Tier 1: Summary View (Default)
```
🚀 Active Work (4)
📋 Next Up (6) 
📊 Overview: 42 pending • 133 done • 2 projects
```

#### Tier 2: Focused View (`--detailed`)
```
🚀 Active Work (4 tasks)
  ▶ [airs-memspec] 13.1 Test AIRS workspace
  ▶ [airs-memspec] 13.2 Validate parsing
  
📋 Next Up (6 tasks)  
  ▶ [airs-memspec] 13.3 Test relationships
  ▶ [airs-memspec] 14.1 Error handling
```

#### Tier 3: Comprehensive View (`--all`)
- Current full display format
- All projects, all statuses
- Complete task details

### Implementation Strategy

#### Phase 2.1: Create Display Tiers
- Define data structures for each tier
- Create rendering engines for summary/focused/comprehensive
- Implement progressive disclosure pattern

#### Phase 2.2: Smart Information Density
- Summary: Counts and high-level status only
- Focused: Task IDs, names, basic status
- Comprehensive: Full details as current

#### Phase 2.3: Navigation Hints
```
📖 Use --detailed for task details, --all for complete view
🔍 Filter: --project X --status Y --limit N
```

**Files to Modify:**
- `src/utils/display_tiers.rs` - New module for tiered display
- `src/utils/templates.rs` - Add summary templates
- `src/cli/args.rs` - Add `--detailed` flag

---

## Development Plan 3: Improved Filtering Options

**Objective**: Provide engineers with precise tools to find exactly what they need.

### Filter Categories

#### 3.1: Status Filters
```bash
--status in-progress    # Only active work
--status pending        # Next up tasks  
--status completed      # Done tasks
--status blocked        # Blocked tasks
--status recent         # Updated in last week
```

#### 3.2: Project/Scope Filters  
```bash
--project airs-memspec  # Single project
--workspace             # All projects in workspace
--mine                  # Based on current context
```

#### 3.3: Time-Based Filters
```bash
--today                 # Updated today
--this-week            # Updated this week
--since 2025-08-01     # Since specific date
--updated-after 3d     # Updated after N days ago
```

#### 3.4: Limiting and Sorting
```bash
--limit 10             # Max N tasks
--sort recent          # By update time
--sort priority        # By importance
--sort project         # Group by project
```

### Implementation Strategy

#### Phase 3.1: Filter Engine Architecture
```rust
pub struct TaskFilter {
    status: Option<Vec<TaskStatus>>,
    project: Option<String>,
    time_range: Option<TimeRange>,
    limit: Option<usize>,
    sort_by: SortOrder,
}
```

#### Phase 3.2: Composable Filtering
- Chain filters with logical AND
- Support multiple values: `--status pending,in-progress`
- Validate filter combinations

#### Phase 3.3: Filter Validation & Hints
- Error messages for invalid filters
- Suggestions when no results found
- Count preview: "Filter would show 5 of 177 tasks"

**Files to Modify:**
- `src/utils/task_filter.rs` - New comprehensive filter engine
- `src/cli/args.rs` - Add all filter options
- `src/models/task.rs` - Add filtering traits

---

## Development Plan 4: Condensed Format

**Objective**: Maximize information density while maintaining readability.

### Current vs. Proposed Format

#### Current Format (Verbose)
```
▶ [airs-memspec] 13.1 - Test with AIRS workspace
  📝 ✅ Basic commands working
  🕒 Updated: 2025-08-08
```

#### Proposed Condensed Format
```
▶ [airs-memspec] 13.1 Test AIRS workspace ✅ 2025-08-08
```

### Design Principles

#### 4.1: Information Hierarchy
- **Primary**: Task ID, name, status indicator
- **Secondary**: Project, update date  
- **Tertiary**: Detailed notes (on-demand)

#### 4.2: Visual Efficiency
- Single line per task in condensed view
- Meaningful emojis for quick status recognition
- Consistent alignment for scanning

#### 4.3: Progressive Disclosure
```bash
# Condensed (default)
▶ [airs-memspec] 13.1 Test AIRS workspace ✅

# Expanded (with --verbose)
▶ [airs-memspec] 13.1 - Test with AIRS workspace
  📝 ✅ Basic commands working  
  🕒 Updated: 2025-08-08
```

### Implementation Strategy

#### Phase 4.1: Format Templates
- Create condensed line template
- Maintain expanded template for detailed view
- Design responsive formatting based on terminal width

#### Phase 4.2: Smart Truncation
- Intelligently truncate long task names
- Preserve key information (project, ID, status)
- Add ellipsis with hover/expansion capability

#### Phase 4.3: Status Indicators
```
✅ Complete    🚀 In Progress    📋 Pending
🚫 Blocked     ⚠️ Issues        🎯 High Priority
```

**Files to Modify:**
- `src/utils/display_format.rs` - New condensed formatting
- `src/utils/templates.rs` - Update task display templates
- `src/cli/args.rs` - Add `--verbose` flag

---

## Implementation Priority & Dependencies

**Recommended Execution Order:**

### Phase 1: Smart Filtering (Foundation)
- **Priority**: Immediate impact
- **Dependencies**: None
- **Estimated Time**: 4-6 hours
- **Value**: Transforms overwhelming tool into usable one

### Phase 2: Condensed Format (Visual Enhancement)  
- **Priority**: High impact
- **Dependencies**: None (works with any filters)
- **Estimated Time**: 3-4 hours
- **Value**: Improves information density and scanning

### Phase 3: Improved Filtering (Power User Features)
- **Priority**: Medium-high impact
- **Dependencies**: Phase 1 (builds on smart filtering)
- **Estimated Time**: 6-8 hours  
- **Value**: Provides precise control for complex workflows

### Phase 4: Tiered Architecture (Advanced UX)
- **Priority**: High sophistication
- **Dependencies**: All others (integrates everything)
- **Estimated Time**: 4-5 hours
- **Value**: Professional-grade progressive disclosure

**Total Estimated Development Time**: 17-23 hours for complete UX transformation

---

## Success Metrics

### Before (Current State)
- 177 tasks displayed always
- Cognitive overload prevents usage
- Engineers avoid the tool
- No contextual intelligence

### After (Target State)
- 5-15 relevant tasks by default
- Quick scanning and actionability
- Context-aware recommendations
- Progressive disclosure for power users

### Validation Criteria
1. **Default view shows ≤15 tasks** (cognitive load management)
2. **Active project tasks prioritized** (context awareness)
3. **Multi-tier access to information** (progressive disclosure)
4. **Sub-second response times** (performance maintained)
5. **Intuitive filter combinations** (power user efficiency)

---

## Integration with Existing Architecture

### Memory Bank Integration
- Reads `current_context.md` for active project detection
- Respects existing task status and metadata
- Maintains compatibility with existing memory bank structure

### CLI Framework Integration  
- Extends existing `clap` argument structure
- Maintains existing global options (`--path`, `--verbose`, etc.)
- Preserves backward compatibility with `--all` flag

### Template System Integration
- Builds on existing template infrastructure
- Adds new condensed templates alongside current ones
- Maintains existing formatting capabilities for detailed views

---

*This document serves as the comprehensive specification for transforming the airs-memspec tasks command from an overwhelming data dump into a focused, professional engineering tool.*
