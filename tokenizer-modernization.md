# Tokenizer Modernization Plan

## Overview

The `lexers` crate (and related tokenizers across the workspace) has grown organically. The `Scanner` API is functional but bespoke, and the tokenizer taxonomy is inconsistent. This document lays out a multi-milestone refactoring to make the crate idiomatic, minimal, and composable.

---

## Milestone 1: Refactor `Scanner` into an Idiomatic Iterator Adapter

### Current Problems

- `Scanner` is a standalone struct with a bespoke API (`buffer_pos`, `set_buffer_pos`, `extract`, `matched_len`) that feels more like a C++ lexer than a Rust iterator adapter.
- It does not compose with standard iterator chains.
- `extract()` is essentially a `drain` but named opaquely.
- State save/restore uses raw `usize` indices, which is error-prone.

### Goals

1. **Make `Scanner` an iterator adapter** (like `std::iter::Peekable`)
   - Add an extension trait `Scanning` on `Iterator` so users can write:
     ```rust
     let scanner = "hello".chars().scanning();
     ```
   - Keep `Scanner::new(iter)` as a fallback constructor.

2. **Replace `extract()` with `drain()`**
   - Rename `extract()` → `drain()` to follow `Vec::drain` naming.
   - Semantics: drain all consumed items from the internal buffer and reset the cursor.

3. **Replace `buffer_pos()` / `set_buffer_pos()` with a `Checkpoint` type**
   - Introduce a lightweight `Checkpoint` struct (or just a type alias wrapper around `usize`) to make save/restore explicit and type-safe.
   - API:
     ```rust
     let checkpoint = scanner.checkpoint();
     // ... try parsing ...
     scanner.restore(checkpoint);
     ```
   - Remove `buffer_pos()` and `set_buffer_pos()` from the public API.

4. **Keep the minimal surface**
   - The following methods are the *only* ones needed on `Scanner`:
     - `next()`, `prev()`, `current()`
     - `peek()`, `peek_prev()`
     - `drain()`
     - `checkpoint()`, `restore()`
     - `accept()`, `ignore()`, `until()` (with `TokenMatcher`)
     - `view()` (for debugging)
   - Remove anything that is not used by existing tokenizers.

### Non-Goals

- Do not add convenience methods for specific grammars to `Scanner`. Those belong in helper functions or tokenizer-specific code.
- Do not change the underlying buffering strategy (it works well).

---

## Milestone 2: Design and Implement the New Tokenizer Taxonomy

### Audit of Existing Tokenizers

| Crate | File | Emits | Notes |
|-------|------|-------|-------|
| `lexers` | `math_tokenizer.rs` | `MathToken` enum | Well-typed, domain-specific |
| `lexers` | `lisp_tokenizer.rs` | `LispToken` enum | Well-typed, domain-specific |
| `lexers` | `ebnf_tokenizer.rs` | `String` | EBNF grammar tokens |
| `lexers` | `delim_tokenizer.rs` | `String` | CSV-style splitting |
| `earlgrey` | `src/earley/mod.rs` (inline) | `String` | Ad-hoc `Scanner` wrapper for math expressions |
| `earlgrey` | `examples/ebnftree.rs` (inline) | `String` | Copy-paste of the above |
| `earlgrey` | `src/ebnf_tokenizer.rs` | `String` | Custom `Peekable<I>` implementation, 95% identical to `lexers::EbnfTokenizer` |
| `numerica` | `src/tokenizer.rs` | `String` | Custom `Peekable<I>` with math/algebra rules (`->`, `/.`, comments, strings) |

### Problems Identified

1. **Duplication:** `earlgrey` duplicates the same math-expression string tokenizer twice (test + example).
2. **Reinvention:** `earlgrey` and `numerica` both wrote custom tokenizers from scratch instead of reusing `lexers` building blocks.
3. **Inconsistency:** `lexers` only provides either (a) fully typed domain-specific tokenizers, or (b) a dumb delimiter splitter. There is no middle-ground generic tokenizer.
4. **Two EBNF tokenizers:** `lexers` and `earlgrey` both have one. They should be unified.

### Proposed Taxonomy

#### Layer 0: `Scanner<I>` (already exists, refactored in Milestone 1)
A rewindable/backtrackable iterator adapter. Not a tokenizer.

#### Layer 1: Generic String Tokenizers (NEW)
Configurable tokenizers that emit raw `String` tokens. Grammar-agnostic.

| Name | Purpose |
|------|---------|
| `DelimTokenizer` | Splits text on delimiter chars (already exists) |
| **`SymbolTokenizer`** *(NEW)* | Chops input at symbol boundaries. User provides single-char and multi-char symbols. Skips whitespace. Accumulates non-symbol runs into string tokens. |

#### Layer 2: Domain-Specific Typed Tokenizers
Built on top of `Scanner`. Interpret semantics and emit typed enums.

| Name | Purpose |
|------|---------|
| `MathTokenizer` | Emits `MathToken` enum |
| `LispTokenizer` | Emits `LispToken` enum |
| `EbnfTokenizer` | Emits `String` tokens for EBNF grammar syntax |

### How `SymbolTokenizer` Would Work

```rust
let tok = SymbolTokenizer::new("a + b * (c - d)")
    .symbols(&['+', '-', '*', '/', '(', ')'])
    .multi_char(&["<=", ">=", "==", ":="]);

// Yields: "a", "+", "b", "*", "(", "c", "-", "d", ")"
```

This single tokenizer would replace the duplicated ad-hoc wrappers in `earlgrey`.

---

## Milestone 3: Consolidate and Migrate Existing Tokenizers

### Actions

1. **Create `SymbolTokenizer`** in `lexers`
   - Implement with `Scanner` (not a custom `Peekable`).
   - Support single-char and multi-char symbols (longest match).
   - Optionally support user-defined predicates for custom token classes.

2. **Migrate `earlgrey` math-expression tokenization**
   - Delete the inline `Tokenizer` in `earlgrey/src/earley/mod.rs`.
   - Delete the copy-paste in `earlgrey/examples/ebnftree.rs`.
   - Replace both with `SymbolTokenizer` configured for math symbols.

3. **Unify EBNF tokenizers**
   - Decide whether `EbnfTokenizer` belongs in `lexers` or `earlgrey`.
   - If it stays in `lexers`, ensure it is rich enough (add `@tag` support, error reporting) to replace `earlgrey/src/ebnf_tokenizer.rs`.
   - Delete the duplicate in `earlgrey`.

4. **Evaluate `numerica`'s tokenizer**
   - Determine if `SymbolTokenizer` + configuration can cover `numerica`'s needs (`->`, `/.`, comments, strings).
   - If not, leave `numerica`'s custom tokenizer as-is and document why.

---

## Open Questions (To Address After Milestone 1)

1. **Multi-char symbol matching in `SymbolTokenizer`:**
   - Should the tokenizer eagerly try longest match (e.g., prefer `:=` over `:` + `=`)?
   - What is the performance impact of checking multi-char symbols on every token?

2. **`EbnfTokenizer` ownership:**
   - Does EBNF tokenization belong in `lexers` (as a generic utility) or in `earlgrey` (as a grammar-specific tool)?

3. **Error reporting in generic tokenizers:**
   - Should `SymbolTokenizer` return `Option<String>` (silent failure) or `Result<String, LexError>`?
   - How should unmatched characters be handled?

4. **`numerica` integration:**
   - Is `numerica`'s tokenizer too specialized to be covered by `SymbolTokenizer`?
   - If so, should we extract common patterns into a tokenizer-builder API?

---

## Success Criteria

- `Scanner` feels like a native Rust iterator adapter (`iter.scanning()`).
- No raw `usize` indices in the public API (`Checkpoint` + `restore` instead).
- `extract` is gone, replaced by `drain`.
- `SymbolTokenizer` exists and covers `earlgrey`'s math-expression use case.
- Duplicate tokenizers in `earlgrey` are deleted.
- All workspace tests pass.
- `cargo clippy` reports zero warnings.

---

## Notes

- Do not commit changes until Milestone 1 is fully complete and tested.
- Each milestone should be its own focused PR (or set of commits).
- This plan may be updated as ambiguities are resolved during implementation.
