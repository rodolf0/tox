# Tokenizer Modernization Plan

## Overview

The `lexers` crate (and related tokenizers across the workspace) has grown organically. The `Scanner` API is functional but bespoke, and the tokenizer taxonomy is inconsistent. This document lays out a multi-milestone refactoring to make the crate idiomatic, minimal, and composable.

---

## Milestone 1: Refactor `Scanner` into an Idiomatic Iterator Adapter *(COMPLETED)*

### Current Problems

- `Scanner` is a standalone struct with a bespoke API (`buffer_pos`, `set_buffer_pos`, `extract`, `matched_len`) that feels more like a C++ lexer than a Rust iterator adapter.
- It does not compose with standard iterator chains.
- `extract()` is essentially a `drain` but named opaquely.
- State save/restore uses raw `usize` indices, which is error-prone.

### Goals

1. **Make `Scanner` an iterator adapter** (like `std::iter::Peekable`)
   - Add an extension trait `Scan` on `Iterator` so users can write:
     ```rust
     let scanner = "hello".chars().scanner();
     ```
   - Keep `Scanner::new(iter)` as a fallback constructor.
   - **Status: COMPLETED.**

2. **Replace `extract()` with `lift()`**
   - Rename `extract()` → `lift()`. (`drain` was considered but collides semantically with `Vec::drain`; `take` collides with `Iterator::take`.)
   - Semantics: remove and return all consumed items from the internal buffer, resetting the cursor.
   - **Status: COMPLETED.**

3. **Replace `buffer_pos()` / `set_buffer_pos()` with a `Checkpoint` type**
   - Introduce a lightweight `Checkpoint` unit struct wrapping `usize` to make save/restore explicit and type-safe.
   - API:
     ```rust
     let checkpoint = scanner.checkpoint();
     // ... try parsing ...
     scanner.restore(checkpoint); // panics on out-of-bounds (programming error)
     ```
   - Remove `buffer_pos()` and `set_buffer_pos()` from the public API.
   - **Status: COMPLETED.**

4. **Keep the minimal surface**
   - The following methods are the *only* ones on `Scanner`:
     - `next()`, `prev()`, `current()`
     - `peek()`, `peek_prev()`
     - `lift()`
     - `checkpoint()`, `restore()`
     - `accept()`, `ignore()`, `until()` (with `TokenMatcher`)
   - `view()` was removed (dead weight).
   - **Status: COMPLETED.**

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

- `Scanner` feels like a native Rust iterator adapter (`iter.scanner()`).
- No raw `usize` indices in the public API (`Checkpoint` + `restore` instead).
- `extract` is gone, replaced by `lift`.
- `SymbolTokenizer` exists and covers `earlgrey`'s math-expression use case.
- Duplicate tokenizers in `earlgrey` are deleted.
- All workspace tests pass.
- `cargo clippy` reports zero warnings.

---

## Notes

- Do not commit changes until Milestone 1 is fully complete and tested.
- Each milestone should be its own focused PR (or set of commits).
- This plan may be updated as ambiguities are resolved during implementation.

---

## Additional Open Items to Consider During Modernization

These were identified during the review but deferred to keep the initial focus on `Scanner` ergonomics. They should be revisited as the plan unfolds.

### 1. Error Reporting in Tokenizers
All tokenizers currently return `Option<Token>`, which makes it impossible for a parser to distinguish "end of valid input" from "syntax error encountered." For example, unmatched quotes or EOF during an escape sequence simply return `None` (acting as if the string never started). Consider introducing `Result<Token, LexError>` or at least `Unknown(String)` variants for the other tokenizers (like `MathTokenizer` already has).

### 2. `EbnfTokenizer` Lookahead Bloat
`lexers::EbnfTokenizer` uses a heap-allocated `Vec<String>` to queue at most 2 items (for the quote-splitting logic: opening quote, content, closing quote). Replace this with a lightweight fixed-size buffer like `[Option<String>; 2]` or `std::collections::VecDeque` to eliminate unnecessary heap allocations.

### 3. `earlgrey` Has Its Own `EbnfTokenizer`
`earlgrey/src/ebnf_tokenizer.rs` is a custom `Peekable<I>` implementation that is 95% identical to `lexers::EbnfTokenizer` but adds `Result`-based error reporting. This duplicate must be unified as part of Milestone 3. Decide which crate owns EBNF tokenization.

### 4. `numerica` Tokenizer Reinvention
`numerica/src/tokenizer.rs` is a 250-line custom tokenizer with math/algebra-specific rules (`->`, `/.`, comments, strings, negative numbers). It was built from scratch instead of reusing `lexers` building blocks. Evaluate whether `SymbolTokenizer` + configuration can cover its needs, or if it is too specialized.

### 5. `scan_whitespace` Still Allocates *(PARTIALLY RESOLVED)*
`scan_whitespace()` returns `Option<String>` of skipped whitespace, but internal callers discard the return value. We added `skip_whitespace()` as a zero-allocation alternative and migrated all internal tokenizers to use it. `scan_whitespace()` remains in the public API solely for backward compatibility with `earlgrey`'s inline math tokenizer. Once that is replaced with `SymbolTokenizer` (Milestone 3), `scan_whitespace()` can be removed entirely.

### 6. `view()` Method on `Scanner` *(RESOLVED)*
`Scanner::view()` returned `&[I::Item]` of the consumed buffer. It was unused outside of `scanner.rs` itself. **Removed in Milestone 1.**

### 7. `scan_number` Accepts `'i'` for Imaginary Numbers
`scan_number` greedily consumes `'i'` at the end, producing strings like `"3i"` that `f64::from_str` cannot parse. We fixed the panic by falling back to `MathToken::Unknown(num)`, but the scanner helper itself still accepts syntactically invalid numbers. Should `scan_number` be stricter and leave `'i'` for a higher-level tokenizer to handle?

### 8. The `Tokenizer::scanner()` Wrapping Pattern
`LispTokenizer::scanner()` and `MathTokenizer::scanner()` return `Scanner<Tokenizer>` so a parser can peek at the token stream. Verify this pattern still works cleanly after the `Scanner` refactor (e.g., does `Checkpoint` work correctly across the `Scanner<Tokenizer>` boundary?).
