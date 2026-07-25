# Alan Compiler Refactoring Plan

## Context

The Alan compiler has two codegen backends (Rust and JS) that share the same IR (`Microstatement`) and type system (`CType`) but maintain separate implementations of the code generation logic. Over time, structural duplication has accumulated, and several modules have grown large enough to warrant splitting.

This document identifies concrete refactoring opportunities, ordered by impact-to-effort ratio.

---

## 1. Abstract Backend Codegen (Highest Impact)

### Problem

`lntors/function.rs` (3249 lines) and `lntojs/function.rs` (2131 lines) each implement `from_microstatement` — a massive match on `Microstatement` variants. The control flow is identical:

- Both handle the same 8 variants in the same order
- The `CType::Function` resolution block is copy-pasted verbatim (~80 lines), including the same TODO comments and error messages
- The `FnKind` match arm structure in the `FnCall` handler is identical (7 arms, same groupings)
- The `generate` function follows the same 7-step skeleton
- Helper functions in `typen.rs` share verbatim blocks (`TString` char filtering, `AnyOf` handling, `Infer` error message, fallback error)

Additionally, the "Special-casing for Option and Result mapping" pattern appears 16 times total (6 in Rust, 10 in JS), each marked with the same TODO: "Make this more centralized."

### Proposed Approach

Create a shared codegen module that handles the IR traversal, with backend-specific rendering delegated via a trait.

#### New module structure

```
alan_compiler/src/
  codegen/
    mod.rs          # Backend trait + shared traversal
    common.rs       # Shared helpers (function resolution, FnKind dispatch, Option/Result)
    lntors/
      function.rs   # Backend-specific rendering only
      typen.rs      # (stays, already backend-specific)
    lntojs/
      function.rs   # Backend-specific rendering only
      typen.rs      # (stays, already backend-specific)
```

#### The Backend trait

```rust
pub trait Backend {
    fn render_microstatement(
        ms: &Microstatement,
        parent_fn: &Function,
        scope: &Scope,
    ) -> Result<String, Box<dyn Error>>;

    fn render_type(typen: Arc<CType>) -> Result<String, Box<dyn Error>>;
    fn generate_type_defs(typen: Arc<CType>, out: &mut OrderedHashMap<String, String>) -> ...;
    fn register_native_dependency(d: &CType, deps: &mut OrderedHashMap<String, String>);
    fn render_function_header(name: &str, args: &[...], rettype: Arc<CType>) -> String;
    // ... other hooks for backend-specific formatting
}
```

#### Shared traversal

The common `from_microstatement` would:
1. Match on `Microstatement` variant
2. For structural logic (function resolution, inlining checks, arg iteration), use shared code
3. For rendering, delegate to `Backend::render_*` methods

The `CType::Function` resolution block (currently ~80 lines of verbatim copy) moves to `common.rs`.

The Option/Result special-casing (16 occurrences) consolidates into a single helper.

#### What stays backend-specific

- Rust: ~29 helper functions for ownership model (liveness, move optimization, borrow hazard guards, `Shared` handling) — these have no JS equivalent
- JS: ~10 helper functions for async/promise handling, primitive boxing, reserved word renaming — these have no Rust equivalent

**Estimated impact**: Reduces combined backend function.rs from ~5400 lines to perhaps ~3500 (shared module ~1500 + lntors ~1200 + lntojs ~800). More importantly, new IR variants or dispatch logic only needs to be written once for the shared path.

#### Risks / Tradeoffs

- The `from_microstatement` signatures differ: Rust takes `shared_vars`, JS doesn't. The trait or shared function would need to handle this (e.g., optional parameter, or the shared function doesn't take it and Rust-specific logic is in the Rust backend's render method).
- The `out` and `deps` OrderedHashMaps are threaded through every call. The trait approach could accumulate output in a context struct instead, which would be a broader improvement.
- This is a large refactor. Should be done in incremental steps:
  1. Extract `CType::Function` resolution to shared module
  2. Extract Option/Result special-casing to shared helper
  3. Extract `generate` function skeleton
  4. Introduce Backend trait
  5. Move shared match arms to common module

---

## 2. Consolidate Backend Entry Points

### Problem

`lntors/mod.rs` (104 lines) and `lntojs/mod.rs` (142 lines) share an identical bootstrap sequence:
```
set_target_lang -> Program::load -> get_program -> scope_by_file -> check main export
-> get main func -> asserts -> compute_inline_targets -> fn_generate -> format output
```

The `register_rust_dependency` and `register_nodejs_dependency` functions are also structurally identical — only the CType variant name differs.

### Proposed Approach

Move the shared bootstrap to a common function in the `codegen` module:

```rust
pub fn compile<BE: Backend>(
    entry_file: String,
    set_target: fn(),
) -> Result<(String, OrderedHashMap<String, String>), Box<dyn Error>> {
    set_target();
    Program::load(entry_file.clone())?;
    // ... shared logic ...
    // ... backend-specific wrapping via trait ...
}
```

Dependency registration becomes a single function that takes the CType variant to match on, or is handled by the `Backend::register_native_dependency` trait method above.

**Estimated impact**: ~100 lines of duplication eliminated. Small but clean.

---

## 3. Split `ctype.rs` (299KB, 6239 lines)

### Problem

The entire type system lives in one file. The `impl CType` block spans ~5300 lines and contains unrelated responsibilities.

### Current Structure

| Section | Lines | Size |
|---------|-------|------|
| Native arg helpers | 23-55 | ~30 |
| `CType` enum definition | 58-172 | ~115 |
| Static OnceLock/LazyLock constants | 134-170 | ~35 |
| **String rendering** (to_string, to_strict_string, to_functional_string, to_callable_string) | 175-1367 | ~1200 |
| **has_infer, degroup** | 1368-1445 | ~80 |
| **Generic inference** (infer_generics_inner_loop, infer_generics) | 1656-2905 | ~1250 |
| **accepts** (type compatibility) | 2906-2949 | ~45 |
| **to_functions** (type expansion) | 2950-3790 | ~840 |
| **from_ast** (AST to type) | 3791-3954 | ~165 |
| **from_ctype, swap_subtype** | 3955-4287 | ~335 |
| **Constructor functions** (binds, casts, tuple, either, prop, etc.) | 4288-4684 | ~400 |
| **Operator functions** (add, sub, mul, div, eq, lt, etc.) | 4698-5501 | ~800 |
| Module-level helpers | 5503-5800 | ~300 |

### Proposed Split

```
program/
  ctype.rs          # Enum definition + statics (~200 lines)
  ctype/
    mod.rs          # Re-exports
    display.rs      # to_string, to_strict_string, to_functional_string, to_callable_string (~1200 lines)
    generics.rs     # infer_generics, has_infer, degroup (~1350 lines)
    compatibility.rs# accepts, to_functions (~900 lines)
    construction.rs # from_ast, from_ctype, swap_subtype, constructors (~900 lines)
    operators.rs    # add, sub, mul, div, eq, lt, etc. (~800 lines)
```

### Dependencies to Check

- `display.rs` depends on: enum definition, statics, helpers (lookup_declared_type_name)
- `generics.rs` depends on: enum definition, display (for string comparison)
- `compatibility.rs` depends on: enum definition, generics, display
- `construction.rs` depends on: enum definition, scope, parse
- `operators.rs` depends on: enum definition

No circular dependencies within the split — just layered dependencies from construction/compatibility upward.

**Estimated impact**: Each file becomes manageable (<1500 lines). Developers working on type display don't need to scroll past generic inference. No functional changes needed.

#### Risks / Tradeoffs

- The `impl CType` block would be split across multiple files using inherent impl blocks. Rust allows multiple `impl CType` blocks, so this is straightforward.
- The static OnceLock constants (operator symbols) could move to their own module or stay in ctype.rs.
- The `display.rs` module currently uses a clever stack-based rendering technique with CType::Infer as sentinel values. This is self-contained and a good candidate for the first extraction.

---

## 4. Replace Thread-Local State with Explicit Context

### Problem

Thread-local state is scattered across 5 modules:

| Module | Thread-locals | Purpose |
|--------|---------------|---------|
| `program/program.rs` | `PROGRAM_RS`, `PROGRAM_JS`, `TARGET_LANG_RS`, `COMPILE_ENV` | Compilation target, env vars |
| `program/function.rs` | `RESOLVING`, `DEF_COUNTER`, `FN_DEF_INDEX`, `VISIBILITY_STACK` | Lazy function resolution |
| `program/inline.rs` | `INLINE_TARGETS`, `INLINE_COUNTER` | Function inlining |
| `lntors/function.rs` | `STMT_IDX`, `UNTRUSTED_DEPTH`, `FN_VALUE_REFS` | Ownership move optimization |
| `parse.rs` | (2 thread-locals) | Parser state |

Additionally, `scope.rs` uses `OnceLock<Mutex<...>>` for root scope caching, and `ctype.rs` uses `LazyLock<Mutex<...>>` for string rendering caches and operator symbol constants.

The `get_program()` / `return_program()` pattern in `program.rs` is a borrow/return protocol that's easy to get wrong — forgetting to return the program leaves it in a taken state.

### Proposed Approach

**Phase 1: Low-risk — consolidate the Program borrow/return pattern**

Wrap `get_program` / `return_program` in a RAII guard:

```rust
struct ProgramGuard<'a> {
    program: Program<'a>,
}
impl Drop for ProgramGuard<'a> {
    fn drop(&mut self) {
        Program::return_program(self.program);
    }
}
```

This makes the pattern `?`-safe and prevents forgotten returns.

**Phase 2: Medium-risk — pass compilation context explicitly**

Replace `PROGRAM_RS` / `PROGRAM_JS` / `TARGET_LANG_RS` with a `CompilationContext` struct that's passed as a parameter. The backends' entry points create the context and pass it down.

**Phase 3: Higher-risk — replace remaining thread-locals**

The `RESOLVING`, `DEF_COUNTER`, `FN_DEF_INDEX`, `VISIBILITY_STACK` state in `function.rs` is used during lazy resolution. This could become part of a `ResolutionContext` passed through the resolution functions.

The `STMT_IDX` and `UNTRUSTED_DEPTH` in the Rust backend are already using RAII guards (`StmtCtxGuard`, `UntrustedGuard`). These could be folded into a `CodeGenContext` struct.

**Estimated impact**: Makes the compiler safe for concurrent compilation, easier to test (no thread-local setup/teardown), and easier to reason about (state is explicit, not implicit).

#### Risks / Tradeoffs

- This is the most invasive refactor. Nearly every function in the compilation pipeline would need signature changes to accept context parameters.
- The `get_program()` / `RAII guard` change (Phase 1) is low-risk and can be done independently.
- The full context-passing refactor (Phases 2-3) would need to be done in a single pass or carefully staged, since partial conversion leaves an inconsistent state.

---

## 5. Consolidate `typen.rs` Duplication

### Problem

`lntors/typen.rs` (565 lines) and `lntojs/typen.rs` (230 lines) both implement `ctype_to_*type` — a match on `CType` variants that lowers to backend-specific representations. Several arms are verbatim identical:

- `TString` character filtering (identical char map)
- `AnyOf` handling (identical comment + collapse logic)
- `Infer` error message (verbatim)
- `Int`/`Float`/`Bool` literals (identical)
- Fallback error message (nearly identical)

### Proposed Approach

Extract the shared variant handling into a helper that both backends call, or fold this into the Backend trait from refactoring #1. The `ctype_to_rtype` / `ctype_to_jtype` functions become thin wrappers around a shared traversal that calls backend-specific renderers for the variants that differ.

**Estimated impact**: ~100 lines of duplication eliminated. Can be done as part of refactoring #1 or independently.

---

## Execution Order

Recommended order to minimize risk and maximize early wins:

1. **Consolidate backend entry points** (Refactoring #2) — Small, isolated, low risk.
2. **Split ctype.rs** (Refactoring #3) — Pure file reorganization, no behavioral changes. Start with extracting `display.rs` (string rendering), then `operators.rs`, then `generics.rs`.
3. **RAII guard for get_program/return_program** (Refactoring #4, Phase 1) — Small, improves safety immediately.
4. **Extract Option/Result special-casing** — ~16 occurrences consolidated into one helper. Can be done before or during #1.
5. **Abstract backend codegen** (Refactoring #1) — Biggest payoff, staged as described in its "Risks / Tradeoffs" section.
6. **Replace remaining thread-locals** (Refactoring #4, Phases 2-3) — Most invasive, deferred until after codegen abstraction is in place.
7. **Consolidate typen.rs** (Refactoring #5) — Can fold into #1 or do independently.

---

## Not In Scope

- The `program/scope.rs` module (61KB) is large but appears well-structured with clear responsibilities. Could be split later if needed.
- The `program/microstatement.rs` module (117KB) is large but represents the IR definition + lowering logic, which is naturally monolithic.
- The `fmt.rs` file (~56KB) — code formatter, separate concern.
