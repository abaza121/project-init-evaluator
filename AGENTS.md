- Refer to CHANGELOG_GUIDELINES.md when writing change logs.

## Rust commenting strategy

- Add a Rust documentation comment (`///`) to every function, struct, and enum, including private and test items. Each comment must explain what the item represents or what behavior it provides, rather than merely restating its name or syntax.
- Within a function body, use inline comments only when the algorithm is complex or the function is longer than 18 lines. Keep them focused on non-obvious intent, invariants, or reasoning rather than narrating individual statements.
- Preserve useful existing comments and update them whenever the associated behavior changes.
- Treat missing, inaccurate, or unnecessary comments as review findings and resolve them before completion.
