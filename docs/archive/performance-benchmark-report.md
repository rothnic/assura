---
title: 'Performance Benchmark Report'
status: historical
---

# Performance Benchmark Report

**Date**: 2026-03-26  
**Assura Version**: 0.1.0  
**LS-Lint Version**: v2.3.1  
**Benchmark Suite**: ls_lint_comparison

---

## Executive Summary

**Assura achieves 6.8x speedup over LS-Lint** on large projects, exceeding the 2x target by 240%.

| Metric | Assura | LS-Lint | Speedup |
|--------|--------|---------|---------|
| **Large Project (6,250 files)** | 9.6ms | 65.8ms | **6.8x** ✅ |
| **Throughput** | 650K f/s | 95K f/s | **6.8x** ✅ |

---

## Detailed Results by Scenario

### 1. Cold Start Performance

Measures first-run initialization overhead.

| Test | Assura | LS-Lint | Speedup |
|------|--------|---------|---------|
| Small (100 files) | 1.71ms | 39.4ms | **23x** ✅ |

**Analysis**: Assura's cold start is dramatically faster due to minimal initialization overhead.

---

### 2. Full Validation by Project Size

Measures complete validation across different project sizes.

#### Small Projects (50 files)
| Metric | Assura | LS-Lint | Speedup |
|--------|--------|---------|---------|
| Time | 1.66ms | 73.0ms | **44x** ✅ |
| Throughput | 36K f/s | 822 elem/s | **44x** ✅ |

#### Medium Projects (500 files)
| Metric | Assura | LS-Lint | Speedup |
|--------|--------|---------|---------|
| Time | 2.60ms | 77.2ms | **30x** ✅ |
| Throughput | 239K f/s | 8K f/s | **30x** ✅ |

#### Large Projects (5,000 files)
| Metric | Assura | LS-Lint | Speedup |
|--------|--------|---------|---------|
| Time | 8.96ms | 102.6ms | **11.5x** ✅ |
| Throughput | 697K f/s | 61K f/s | **11.5x** ✅ |

**Analysis**: Performance advantage scales with project size. Assura's parallel architecture shines on larger projects.

---

### 3. Throughput Test (6,250 files)

Raw validation speed at maximum throughput.

| Metric | Assura | LS-Lint | Speedup |
|--------|--------|---------|---------|
| Time | 9.89ms | 105.4ms | **10.7x** ✅ |
| Throughput | 627K f/s | 59K f/s | **10.7x** ✅ |

---

### 4. Project Type Performance

Tests on realistic project structures.

#### Rust Projects
| Metric | Assura | LS-Lint | Speedup |
|--------|--------|---------|---------|
| Time | 2.29ms | 75.3ms | **33x** ✅ |
| Throughput | 59K elem/s | 1.8K elem/s | **33x** ✅ |

#### JavaScript Projects
| Metric | Assura | LS-Lint | Speedup |
|--------|--------|---------|---------|
| Time | 1.64ms | 78.3ms | **48x** ✅ |
| Throughput | 140K elem/s | 2.9K elem/s | **48x** ✅ |

**Analysis**: Real-world project types show even better performance than synthetic tests.

---

### 5. Incremental Validation

Single file change validation (microseconds).

| Metric | Assura |
|--------|--------|
| Time | 10.9µs |

**Note**: LS-Lint doesn't support true incremental validation - it always does full scans.

---

### 6. Complex Rules Performance

Multiple constraints (4 rules: naming, directory, file size, etc.).

| Metric | Assura |
|--------|--------|
| Time | 5.66ms |
| Throughput | 109K elem/s |

---

## Key Optimizations Applied

### 1. Parallel Directory Walking (jwalk)
Using `jwalk` instead of `walkdir` provides 2-3x improvement through parallel filesystem traversal.

### 2. Non-Recursive Directory Validation
Fixed O(n²) issue by disabling recursive directory validation when using external file walking.

**Before**: Each directory validation re-walked all subdirectories  
**After**: Single parallel walk, non-recursive directory checks

### 3. Parallel Constraint Validation
Using `rayon` for parallel validation of constraints across files.

---

## Performance Pitfalls Documented

See CONSTITUTION.md Section 6.2 for detailed performance anti-patterns:

1. **O(n²) Directory Walking**: Recursive constraints + external walking
2. **Redundant Metadata Calls**: Multiple `fs::metadata()` on same file
3. **Sequential File Walking**: Not using parallel walkers

---

## Conclusion

✅ **All performance targets exceeded**
- Target: 2x faster than LS-Lint
- Achieved: **6.8x faster** on large projects
- Up to **48x faster** on small/realistic projects

Assura's Rust + parallel architecture provides significant performance advantages across all tested scenarios.

---

*Benchmarks run on: Linux x86_64, with jwalk parallel walking enabled*
