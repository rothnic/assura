# Phase 1 Review: Issues and Tech Debt

## Critical Issues (Must Fix Before Phase 2)

### 1. **serde(untagged) Fragility** 
Multiple enums use `#[serde(untagged)]` which causes deserialization to try variants in order. This is fragile:

**File: `src/config/ast.rs`**

**ConstraintItem enum (lines 42-60):**
```rust
#[serde(untagged)]
pub enum ConstraintItem {
    Constraint(Constraint),
    ViolationArray(Vec<ViolationEntry>),
    Message(Message),
    ContextOverride { ... },
}
```
**Problem:** ViolationArray is Vec, Constraint is an enum - they could be confused. Message is a struct with HashMap.

**FileItem enum (lines 206-226):**
```rust
#[serde(untagged)]
pub enum FileItem {
    Apply { apply: String },
    ApplyArray { apply: Vec<String> },  // SAME KEY!
    Constraints { constraints: Vec<Constraint> },
    Violation { violation: Vec<ViolationEntry> },
    Exists { exists: Range },
    Message(Message),
}
```
**CRITICAL:** `Apply` and `ApplyArray` both use key `apply`. This will **always** fail for arrays because serde will try `Apply` first (String) and fail on Vec.

**Fix:** Use `#[serde(tag = "type")]` or distinguish by key name:
```rust
ApplySingle { apply: String },
ApplyMultiple { apply_rules: Vec<String> },  // Different key!
```

### 2. **PolicyEntry enum Issues**
```rust
#[serde(untagged)]
pub enum PolicyEntry {
    Directory(PolicyNode),  // HashMap
    File(Vec<FileItem>),    // Vec
    Strict { strict: bool },  // Object with "strict" key
    ViolationDefault { violation: Vec<ViolationEntry> },  // Object with "violation" key
    ContextDef { context: String, violation: Vec<ViolationEntry> },
}
```
**Problem:** Order matters. HashMap vs Struct distinction is fragile.

**Fix:** Use explicit tags:
```rust
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PolicyEntry {
    Directory { entries: PolicyNode },
    File { items: Vec<FileItem> },
    Strict { value: bool },
    // etc
}
```

### 3. **ViolationEntry Parsing is Fragile**
```rust
#[serde(untagged)]
pub enum ViolationEntry {
    Level(String),  // "warn"
    ContextSpecific { context: String, level: String },  // "ci:block"
}
```
**Problem:** How does "ci:block" parse? It's a String with colon, not a struct.

**Fix:** Custom deserializer or change notation:
```yaml
# Current (problematic)
violation: [warn, ci:block]

# Better
violation: [warn, {context: ci, level: block}]
# or
violation: 
  - warn
  - ci: block
```

## Medium Issues (Should Fix Soon)

### 4. **Preprocessor Issues**
**File: `src/config/preprocessor.rs`**

**Bug: normalize_ranges not called:**
```rust
pub fn process(input: &str) -> String {
    // Never calls normalize_ranges!
}
```

**Bug: Regex pattern incomplete:**
```rust
static ref NEEDS_QUOTING: Regex = Regex::new(
    r"^(\s*)(\.\w+|\*\*|\*|\$\w+|\w*\*\w*)(\s*:)$"
).unwrap();
```
The `\w*\*\w*` pattern doesn't anchor properly and could miss cases.

**Missing: No handling of `${name}` patterns as keys**
The regex doesn't properly detect `${name}.tsx` patterns.

### 5. **Range Enum Will Misparse**
```rust
#[serde(untagged)]
pub enum Range {
    Exact(u64),        // 100
    RangeString(String),  // "100.."
}
```
**Problem:** YAML `exists: 100` will parse as `Exact(100)`, but `exists: 100..` needs quotes first.

**Fix:** Always preprocess ranges to strings before parsing.

### 6. **No Preprocessing in Config::from_yaml**
```rust
pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
    serde_yaml::from_str(yaml)  // NO PREPROCESSING!
}
```
This bypasses the preprocessor entirely!

## Minor Issues (Nice to Have)

### 7. **Missing Documentation**
- Many public types lack doc comments
- Examples in docs don't match implementation

### 8. **No Validation of Rule References**
Parser doesn't check if `apply: react` refers to an existing rule.

### 9. **PolicyNode uses flatten**
```rust
#[serde(flatten)]
pub entries: HashMap<String, PolicyEntry>,
```
This works but loses structure in serialized form. Consider if this is intended.

## Recommended Fixes Before Phase 2

### Priority 1 (Critical)
1. Fix `FileItem` enum - distinguish Apply single vs array
2. Add explicit serde tags to PolicyEntry
3. Fix ViolationEntry to use struct notation
4. Call preprocessor in Config::from_yaml

### Priority 2 (Important)
5. Fix preprocessor regex patterns
6. Add range normalization to preprocessor
7. Add validation for rule references

### Priority 3 (Enhancement)
8. Add comprehensive error messages
9. Document all public APIs
10. Add integration tests with real configs

## Test Coverage Gaps

Current tests cover:
- ✅ Basic parsing
- ✅ Preprocessor quoting
- ✅ JSON roundtrip
- ❌ Complex violation arrays with messages
- ❌ File pairing logic
- ❌ Context inheritance
- ❌ Error cases (malformed YAML)
- ❌ Rule reference validation

## Decision Needed

**Question:** Should we fix these issues now or proceed to Phase 2 and address them incrementally?

**Recommendation:** Fix Priority 1 issues now - they're fundamental to the data model and will be hard to change later.

---

*Review completed: 2026-03-24*
*Next: Fix critical issues before proceeding to Phase 2*
