# Siddhi Rust Implementation Roadmap

This document tracks the implementation tasks for achieving feature parity with the Java version of Siddhi CEP.

## Task Categories

- 🔴 **Critical** - Core functionality blocking other features
- 🟠 **High** - Important features for common use cases  
- 🟡 **Medium** - Nice-to-have features
- 🟢 **Low** - Advanced/specialized features

## Implementation Tasks

### 🔴 Critical - Foundation Improvements

- [x] **Error Handling Overhaul** ✅ **COMPLETED**
  - ✅ Replace all `String` errors with proper error types using `thiserror`
  - ✅ Create error hierarchy matching Java's exception model
  - ✅ Add context to errors for better debugging
  - ✅ **Effort**: 1 week
  - **Files**: `src/core/exception/`, all modules returning `Result<_, String>`

- [ ] **Type System Enhancement**
  - Implement comprehensive type checking and coercion
  - Match Java's type conversion behavior
  - Add validation at parse time
  - **Effort**: 1 week
  - **Files**: `src/core/event/value.rs`, `src/core/util/attribute_converter.rs`

- [ ] **State Management Framework**
  - Implement proper state checkpointing
  - Add state recovery mechanisms
  - Support distributed state storage
  - **Effort**: 2 weeks
  - **Files**: `src/core/util/state_holder.rs`, `src/core/persistence/`

### 🟠 High Priority - Core Windows

- [ ] **Session Window** (`session`, `sessionLength`)
  - Implement session-based windowing
  - Support dynamic session timeouts
  - **Effort**: 3 days
  - **Reference**: `SessionWindowProcessor.java`

- [ ] **Sort Window** (`sort`)
  - Implement sorting within windows
  - Support multiple sort attributes
  - **Effort**: 2 days
  - **Reference**: `SortWindowProcessor.java`

- [ ] **Unique Windows** (`unique`, `uniqueLength`)
  - Implement duplicate removal
  - Support attribute-based uniqueness
  - **Effort**: 3 days
  - **Reference**: `UniqueWindowProcessor.java`

- [ ] **Delay Window** (`delay`)
  - Implement event delay mechanism
  - **Effort**: 2 days
  - **Reference**: `DelayWindowProcessor.java`

- [ ] **Batch Windows** (`batch`, `timeBatch` improvements)
  - Complete batch window implementations
  - Add start time parameter support
  - **Effort**: 3 days

### 🟠 High Priority - Query Features

- [ ] **Group By Enhancement**
  - Full group by support with multiple attributes
  - Having clause implementation
  - **Effort**: 1 week
  - **Files**: `src/core/query/selector/`

- [ ] **Order By & Limit**
  - Complete order by implementation
  - Add limit/offset support
  - **Effort**: 3 days
  - **Files**: `src/core/query/selector/order_by_event_comparator.rs`

- [ ] **Absent Pattern Detection**
  - Implement absent event patterns
  - Support for `not` and `every not` patterns
  - **Effort**: 1 week
  - **Reference**: Java absent pattern tests

### 🟠 High Priority - Sources & Sinks

- [ ] **HTTP Source/Sink**
  - REST API endpoint support
  - Request/response mapping
  - **Effort**: 1 week

- [ ] **TCP/Socket Source/Sink**
  - Binary protocol support
  - Connection management
  - **Effort**: 1 week

- [ ] **File Source/Sink**
  - File reading/writing
  - Directory watching
  - **Effort**: 3 days

### 🟡 Medium Priority - Additional Windows

- [ ] **Frequent Windows** (`frequent`, `lossyFrequent`)
  - Implement frequency-based windows
  - **Effort**: 3 days

- [ ] **Expression Windows** (`expression`, `expressionBatch`)
  - Dynamic window based on expressions
  - **Effort**: 3 days

- [ ] **Time Length Window** (`timeLength`)
  - Combined time and length constraints
  - **Effort**: 2 days

- [ ] **Hopping Window** (`hop`)
  - Overlapping time windows
  - **Effort**: 2 days

### 🟡 Medium Priority - Performance

- [ ] **Event Object Pool**
  - Reduce allocations with object pooling
  - Implement event recycling
  - **Effort**: 1 week

- [ ] **Lock-Free Data Structures**
  - Replace Mutex with lock-free alternatives where possible
  - Use crossbeam for concurrent collections
  - **Effort**: 2 weeks

- [ ] **Expression Executor Optimization**
  - Reduce boxing/dynamic dispatch
  - Consider code generation for hot paths
  - **Effort**: 1 week

### 🟡 Medium Priority - Extensions

- [ ] **Script Function Support**
  - JavaScript executor using V8/QuickJS
  - Python support via PyO3
  - **Effort**: 2 weeks

- [ ] **Kafka Source/Sink**
  - Kafka consumer/producer
  - Offset management
  - **Effort**: 1 week

- [ ] **Database Sink**
  - Batch insert support
  - Connection pooling
  - **Effort**: 3 days

### 🟢 Low Priority - Advanced Features

- [ ] **Distributed Processing**
  - Cluster coordination
  - Work distribution
  - **Effort**: 1 month

- [ ] **Query Optimization**
  - Query plan optimization
  - Cost-based optimization
  - **Effort**: 2 weeks

- [ ] **Debugger Support**
  - Breakpoint support
  - Event inspection
  - **Effort**: 2 weeks

- [ ] **Metrics & Monitoring**
  - Prometheus metrics
  - Performance profiling
  - **Effort**: 1 week

### 🟢 Low Priority - Remaining Windows

- [ ] **Window of Windows** (`window`)
- [ ] **External Time Length** (`externalTimeLength`)
- [ ] **Unique External Time Batch** (`uniqueExternalTimeBatch`)

## Implementation Guidelines

### For Each Task:

1. **Study Java Implementation**
   - Review Java source in `modules/siddhi-core/`
   - Understand the algorithm and edge cases
   - Check Java tests for expected behavior

2. **Design Rust Implementation**
   - Follow existing Rust patterns in codebase
   - Use traits over inheritance
   - Prefer `Result<T, E>` over panics

3. **Write Tests First**
   - Port relevant Java tests
   - Add Rust-specific edge cases
   - Use `AppRunner` for integration tests

4. **Document Thoroughly**
   - Add rustdoc comments
   - Update CLAUDE.md if adding new patterns
   - Include examples in documentation

## Getting Started

To pick up a task:

1. Choose a task matching your experience level
2. Create a branch: `feature/window-session` (example)
3. Implement with tests
4. Submit PR with:
   - Description of changes
   - Link to Java implementation
   - Test coverage report
   - Performance impact (if any)

## Progress Tracking

- ✅ Completed
- 🚧 In Progress  
- ⏸️ Blocked
- ❌ Cancelled

Last Updated: 2025-07-30