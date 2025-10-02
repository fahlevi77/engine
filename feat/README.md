# Siddhi Rust Features Documentation

This directory contains detailed documentation for specific features and architectural components of Siddhi Rust.

## Directory Structure

### 📂 [distributed/](distributed/)
Distributed processing framework, cluster coordination, and deployment configurations.

**Documentation**: [DISTRIBUTED.md](distributed/DISTRIBUTED.md)

**Topics Covered**:
- Complete distributed processing architecture
- Runtime mode abstraction (SingleNode/Distributed/Hybrid)
- Transport layer (TCP and gRPC implementations)
- Redis state backend with production-ready features
- Configuration management (YAML, Kubernetes, Docker)
- Deployment guides and troubleshooting

**Related Code**: `src/core/distributed/`

**Status**: Foundation Complete, Extensions In Progress

---

### 📂 [state_management/](state_management/)
Enterprise-grade state management, checkpointing, compression, and persistence.

**Documentation**: [STATE_MANAGEMENT.md](state_management/STATE_MANAGEMENT.md)

**Topics Covered**:
- Comprehensive state management architecture
- StateHolder trait and implementation patterns
- Incremental checkpointing system
- Compression utilities (90-95% compression ratios)
- Point-in-time recovery
- Distributed state coordination

**Related Code**: `src/core/persistence/`

**Status**: Production Complete

---

### 📂 [async_streams/](async_streams/)
High-performance asynchronous stream processing with @Async annotations.

**Documentation**: [ASYNC_STREAMS.md](async_streams/ASYNC_STREAMS.md)

**Topics Covered**:
- Lock-free crossbeam-based event pipeline
- @Async annotation usage patterns
- Backpressure strategies (Drop, Block, ExponentialBackoff)
- Performance characteristics (>1M events/sec)
- Real-world examples (financial, IoT, logs)
- Troubleshooting and best practices

**Related Code**: `src/core/stream/`, `src/core/util/pipeline/`

**Status**: Production Ready

---

### 📂 [grammar/](grammar/)
Parser architecture, SQL integration, and query language design.

**Documentation**: [GRAMMAR.md](grammar/GRAMMAR.md)

**Topics Covered**:
- Parser technology evaluation (LALRPOP, sqlparser-rs, Tree-sitter, Pest)
- Hybrid parser strategy (SQL-first approach)
- Current LALRPOP implementation
- Future SQL compatibility plans
- MATCH_RECOGNIZE implementation strategy
- Grammar design principles

**Related Code**: `src/query_compiler/`

**Status**: Hybrid Parser Strategy Planned

---

### 📂 [error_handling/](error_handling/)
Error handling framework using thiserror and comprehensive error hierarchy.

**Documentation**: [ERROR_HANDLING.md](error_handling/ERROR_HANDLING.md)

**Topics Covered**:
- Hierarchical error types (StateError, QueryError, RuntimeError, IOError, ConfigError)
- Error propagation and context patterns
- Migration guide from String errors
- Best practices and testing strategies
- Error coverage metrics (121+ Result types)

**Related Code**: `src/core/error/`

**Status**: Production Ready

---

### 📂 [implementation/](implementation/)
Developer guides for implementing new features and components.

**Documentation**: [IMPLEMENTATION.md](implementation/IMPLEMENTATION.md)

**Topics Covered**:
- Window processors and stream functions
- Aggregator executors
- Sources and sinks
- Table implementations
- Java-to-Rust translation patterns
- Factory registration and testing strategies
- Performance optimization techniques

**Related Code**: All `src/` directories

**Status**: Complete - Comprehensive Reference

---

## Navigation

### 🏠 Core Project Documentation (Root Directory)
- `../README.md` - Project overview and getting started
- `../CLAUDE.md` - AI assistant context and development guidelines
- `../ROADMAP.md` - Comprehensive technical roadmap (dev-focused)
- `../MILESTONES.md` - User-facing release milestones and product evolution

### 🧑‍💻 For Developers
Start with:
1. `../CLAUDE.md` - Understand project architecture and conventions
2. [implementation/IMPLEMENTATION.md](implementation/IMPLEMENTATION.md) - Learn implementation patterns
3. `../ROADMAP.md` - See current priorities and tasks

### 👥 For Users
Start with:
1. `../README.md` - Get started with Siddhi Rust
2. `../MILESTONES.md` - Understand upcoming features and releases
3. Feature-specific guides in this directory

---

## Documentation Philosophy

### Single-File Approach
Each feature directory now contains a **single comprehensive document** that consolidates all relevant information:
- Eliminates duplicate, outdated, or conflicting content
- Provides a clear, authoritative reference for each feature
- Maintains latest decisions and best approaches
- Easier to maintain and keep up-to-date

### ROADMAP.md vs MILESTONES.md
- **ROADMAP.md** (root): Developer-focused, comprehensive task tracking, technical details, all gaps documented
- **MILESTONES.md** (root): User-facing, release planning, product evolution story, what's shipping when

### Feature Documentation Standards
Each consolidated feature document includes:
- **Overview**: Current status and key features
- **Architecture**: Design and implementation details
- **Implementation**: Code examples and patterns
- **Configuration**: Setup and deployment guides
- **Examples**: Practical usage examples
- **Troubleshooting**: Common issues and solutions
- **Status**: Current implementation progress

---

## Contributing Documentation

When adding new features:
1. Create a subdirectory under `feat/` if it's a major new component
2. Create a single comprehensive document (e.g., `FEATURE_NAME.md`)
3. Include all relevant information in one place
4. Update this README with links to new documentation
5. Keep ROADMAP.md and MILESTONES.md in sync

---

Last Updated: 2025-10-02
