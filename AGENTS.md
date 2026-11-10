Do not leave comments in the code.

Do not use unsafe blocks, if you need unsafe delegate it to an appropriate library.

Favor offsets and arena/pool/bump allocators over pointers and lifetimes. 

Favor cache-friendly patterns.

Do not make core algorithm changes without consulting me and testing the performance of the changes
with hyperfine prior to and after the change.



## Coding Guidelines

### Correctness
1. **Write the failing test first.** Reproduce the bug with a tiny input, watch the test fail, then fix it. The suite used to pass while 9 of 10 parsers dropped the final line.
2. **Never edit an assertion just to make a test pass.** Work out which side is right. Only change the expected value if it encoded a bug, and say so in the report.
3. **Check invariants across all languages.** Loop over `ALL_LANGS` and assert:
   - `lines == code + comments + blanks`
   - `parse(s) == parse(s + "\n")`
   - no panic when the input is truncated at any byte offset
4. **Audit every EOF path.** For each `break`, `i = content_len` or `return content_len`, ask:
   - Was the current line counted?
   - Is `line_state` right for the tail?
   - Was anything counted twice?
5. **Fix a bug in every parser, not just one.** The parsers are near-copies, so grep all of `src/lang/*.rs` for the same pattern.
6. **Check behavior with the release binary.** Write small probe files in scratch and run `./target/release/lfa <file>`.

### Performance
7. **Keep the hot path unchanged.** Put rare features behind a state-gated table or branch (like `TOKEN_KIND_INTERP`). Never add common bytes (`{ } ( ;`) to the main `TOKEN_KIND`.
8. **Run hyperfine before and after, in the same session, on the same corpus.** Save the old binary first. Use `-N -w 5 -r 30` and report mean ± σ. A difference within σ is noise; don't call it a speedup.
9. **No heap allocation in `parse_into`.** Use fixed stack arrays or the existing arenas.

### Before reporting done
10. Run:
   ```bash
   cargo test && cargo clippy --all-targets && cargo build --release && ./target/release/lfa . | grep Rust
   grep -rn "unsafe {" src
   ```
   The Comments column must be 0 and the grep must find nothing.
11. Follow the rules above: no comments, no `unsafe`, offsets and arenas, compile-time `const` tables with `assert!`.
12. **Ask before changing anything core:** hashing, SIMD routines, the scheduler, or the tokenizer design. A gated new token kind is fine if hyperfine shows no regression; still report the numbers.

### Process
13. Make small commits, one change each. Commit before starting anything risky.
14. If a request is ambiguous (e.g. which "#3"), ask one short question before coding.
15. The final report should cover: what changed, test results, the hyperfine table, and any edge cases not handled.
