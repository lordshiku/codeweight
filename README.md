# codeweight

**codeweight** is a static code complexity analyzer written in Rust. Point it at a directory of source files and it reports line metrics, function counts, cyclomatic complexity, nesting depth, and a maintainability score — per file and in aggregate.

Complexity metrics do not tell the whole story of code quality, but they surface files that are harder to test, harder to change, and more likely to hide defects. codeweight is designed to be fast, dependency-light, and easy to pipe into other tools.

## Supported languages

| Language | Extensions |
|----------|------------|
| Python | `.py` |
| JavaScript / TypeScript | `.js`, `.jsx`, `.ts`, `.tsx`, `.mjs`, `.cjs` |
| Rust | `.rs` |
| Java | `.java` |

## Installation

From source (requires [Rust](https://rustup.rs/) 1.70+):

```bash
git clone <your-repo-url>
cd codeweight
cargo install --path .
```

The `codeweight` binary is installed to `~/.cargo/bin` (ensure that directory is on your `PATH`).

## Usage

```text
codeweight analyze <path> [--lang <python|js|rust|java|all>] [--json] [--min-complexity <n>] [--sort-by <lines|complexity|functions|score>] [--verbose]
codeweight summary <path> [--lang <python|js|rust|java|all>] [--json] [--verbose]
codeweight --version
codeweight --help
```

### Example 1: Analyze a project directory

```bash
codeweight analyze ./src
```

```text
╭────────────────┬────────────┬───────┬──────┬─────┬────────────┬────────┬──────┬───────╮
│ File           ┆ Lang       ┆ Lines ┆ Code ┆ Fns ┆ Complexity ┆ Max Fn ┆ Nest ┆ Score │
╞════════════════╪════════════╪═══════╪══════╪═════╪════════════╪════════╪══════╪═══════╡
│ lib.rs         ┆ rust       ┆ 42    ┆ 35   ┆ 3   ┆ 8          ┆ 4      ┆ 3    ┆ 78.2  │
│ main.rs        ┆ rust       ┆ 28    ┆ 22   ┆ 2   ┆ 5          ┆ 3      ┆ 2    ┆ 82.1  │
╰────────────────┴────────────┴───────┴──────┴─────┴────────────┴────────┴──────┴───────╯

Summary
╭───────┬───────┬──────┬─────┬────────────┬────────┬──────────┬───────────╮
│ Files ┆ Lines ┆ Code ┆ Fns ┆ Complexity ┆ Max Fn ┆ Max Nest ┆ Avg Score │
╞═══════╪═══════╪══════╪═════╪════════════╪════════╪══════════╪═══════════╡
│ 2     ┆ 70    ┆ 57   ┆ 5   ┆ 13         ┆ 4      ┆ 3        ┆ 80.2      │
╰───────┴───────┴──────┴─────┴────────────┴────────┴──────────┴───────────╯
```

### Example 2: JSON output for scripting

```bash
codeweight analyze ./src --json
```

```json
{
  "files": [
    {
      "path": "lib.rs",
      "language": "rust",
      "total_lines": 42,
      "code_lines": 35,
      "function_count": 3,
      "cyclomatic_complexity": 8,
      "maintainability_score": 78.2
    }
  ],
  "aggregate": {
    "file_count": 1,
    "cyclomatic_complexity": 8,
    "avg_maintainability_score": 78.2
  }
}
```

### Example 3: Filter by language

```bash
codeweight analyze ./project --lang python
```

### Example 4: Surface only complex files

```bash
codeweight analyze ./src --min-complexity 15 --sort-by complexity
```

### Example 5: Aggregate summary only

```bash
codeweight summary ./tests/fixtures --json
```

```json
{
  "aggregate": {
    "file_count": 4,
    "total_lines": 68,
    "code_lines": 52,
    "function_count": 9,
    "cyclomatic_complexity": 24,
    "avg_maintainability_score": 71.4
  }
}
```

## Metrics explained

### Line metrics

| Metric | Description |
|--------|-------------|
| **Total lines** | Every line in the file, including blanks |
| **Blank lines** | Lines that are empty or whitespace-only |
| **Comment lines** | Lines that are comments only (language-specific rules) |
| **Code lines** | Lines that contain executable code |

### Function count

The number of function or method definitions detected using language-specific line patterns (e.g. `def` in Python, `fn` in Rust, `function` / arrow functions in JavaScript).

### Cyclomatic complexity

An **approximate** count of independent execution paths. For each function, codeweight starts at a base complexity of **1** and adds **1** for each detected branch point:

- `if`, `else if` / `elif`, standalone `else`
- `for`, `while`, `match`, `case`, `switch`, `catch`
- `&&`, `||`, and ternary `?` operators

Per-file complexity is the **sum** across all functions. The table also shows **Max Fn** — the highest complexity among individual functions in that file.

### Max nesting depth

The deepest level of nested blocks in a file:

- **Brace languages** (JS, Rust, Java): maximum `{` / `}` nesting depth
- **Python**: maximum indentation depth (4 spaces = one level)

### Maintainability score

A composite score from **0** (difficult to maintain) to **100** (easy to maintain), computed as:

```
function_divisor = max(function_count, 1)
complexity_density = cyclomatic_complexity / function_divisor

complexity_penalty = min(45, complexity_density × 5)
nesting_penalty    = min(25, max_nesting_depth × 4)
length_factor      = ln(1 + code_lines) / ln(1 + 500)
length_penalty     = min(20, length_factor × 20)
sparsity_penalty   = 10 if code_lines > 50 and function_count == 0, else 0

score = clamp(100 − complexity_penalty − nesting_penalty − length_penalty − sparsity_penalty, 0, 100)
```

The score is a heuristic, not an industry standard like Halstead or SQALE. It is most useful for **relative** comparison within a codebase.

### Color thresholds (table mode)

| Metric | Green | Yellow | Red |
|--------|-------|--------|-----|
| Complexity | ≤ 10 | 11–25 | > 25 |
| Score | ≥ 70 | 40–69 | < 40 |

## Design decisions

### Why regex-based parsing instead of full AST parsing?

codeweight deliberately avoids language toolchains (no `rustc`, `tsc`, `javac`, or Python AST modules). Parsing is done with line classification, keyword counting, and delimiter tracking.

**Reasons:**

1. **Speed** — No subprocess spawns, no parser initialization. A directory scan is bounded by disk I/O and simple string passes.
2. **Zero external dependencies** — The binary is self-contained. Install once with `cargo install` and run anywhere.
3. **Uniform interface** — All four languages flow through the same `Analyzer` trait, keeping the architecture simple and testable.
4. **Good enough for triage** — The goal is to flag outliers (very long files, deeply nested logic, high branch density), not to produce compiler-grade metrics.

**Tradeoffs:**

- Function boundaries are inferred from line patterns, not syntax trees. Nested functions, macros, and unusual formatting can confuse function detection.
- Branch counting does not understand string literals perfectly in all edge cases (though inline comments are stripped first).
- Cyclomatic complexity is **approximate** — it may over-count (`else if` was a known pitfall, now normalized) or under-count (implicit branches, short-circuit semantics).
- No cross-file analysis: imports, call graphs, and type complexity are out of scope.

### What would v2 with tree-sitter look like?

A tree-sitter integration would upgrade accuracy at the cost of complexity:

1. **Per-language grammars** — Add `tree-sitter-python`, `tree-sitter-javascript`, etc. as optional features or compile-time flags.
2. **AST-driven metrics** — Walk function nodes for precise complexity; count only real control-flow nodes; ignore branches inside string literals by construction.
3. **Richer metrics** — Cognitive complexity, parameter counts, return-point analysis, and dead-code hints become feasible.
4. **Hybrid architecture** — Keep the fast regex path as `--fast` default; opt into tree-sitter with `--precise` for CI gates on critical modules.

The current v1 design optimizes for **portability and speed**. Tree-sitter is the natural upgrade path when accuracy matters more than binary size.

## Limitations

- Does not resolve imports or build a call graph
- Does not type-check or parse invalid syntax reliably
- Approximates cyclomatic complexity (not a formal McCabe analysis)
- Python nesting uses indentation heuristics (tabs vs spaces may skew depth)
- Java method detection uses keyword heuristics and may miss unusual signatures
- Silently skips unsupported file types (use `--verbose` to see what was skipped)
- Unreadable files produce a warning and are skipped

## Development

```bash
cargo test
cargo clippy --all-targets -- -D warnings
```

## Contributing

Contributions are welcome. Please:

1. Open an issue to discuss significant changes
2. Add or update integration test fixtures for language-specific behavior
3. Ensure `cargo test` and `cargo clippy -- -D warnings` pass
4. Document metric formula changes in this README

## License

MIT
