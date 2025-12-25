# Copilot Instructions (Repository-Wide)

These instructions apply to all Copilot Chat interactions in this repository.

## 1) Authoritative references (mandatory)
When implementing or modifying Rust code:
- Always consult and follow official Rust documentation first:
  - The Rust Reference (language semantics)
  - The Rust Standard Library documentation (std)
  - Cargo documentation (build/deps/features)
- Prefer standard library solutions when available.
- If proposing a crate, justify it briefly and confirm it aligns with repo conventions.

## 2) Task prioritization (mandatory)
Before starting work, always:
1. Open and read `TODO.md`.
2. Identify the highest-priority relevant task.
3. Work ONLY on that task unless explicitly instructed otherwise.

If `TODO.md` conflicts with another request:
- Ask for clarification and cite the conflict, but do not proceed with lower-priority work.

## 3) Architecture & organization (mandatory)
Before adding new modules, files, or APIs:
1. Open and read `ARCHITECTURE.md`.
2. Follow the defined project structure, layering, and boundaries.
3. Place code in the correct module/crate according to those rules.

If `ARCHITECTURE.md` is unclear or missing guidance:
- Propose the smallest reasonable change and explain how it fits existing structure.

## 4) Prevent scope creep / sidetracking (mandatory)
- Do not introduce refactors, dependency changes, formatting sweeps, or unrelated improvements unless:
  - They are required to complete the prioritized TODO item, OR
  - The user explicitly requests them.
- If you notice a separate issue, note it briefly under “Follow-ups” and return to the task.

## 5) TODO.md must be updated on completion (mandatory)
Whenever you complete a TODO item (fully or partially), you MUST update `TODO.md` in the same change set:
- Mark the item as completed using the file’s existing convention (e.g., checkboxes, DONE section, status tags).
- If only partially complete:
  - Update the item to reflect current state and remaining work.
  - Add new sub-tasks if needed, but keep them scoped and prioritized.
- If new follow-up work is discovered that is not required to finish the current item:
  - Add it under a “Follow-ups” section (or equivalent) with lower priority.
- Do not remove TODO history unless the file’s conventions explicitly allow it.

## 6) Implementation workflow (required)
For each feature or change:
1. Restate the target TODO item (1 sentence).
2. Summarize relevant constraints from `ARCHITECTURE.md`.
3. Identify which Rust docs are relevant (Reference / std / Cargo).
4. Implement the minimal change set.
5. Add/update tests as appropriate.
6. Verify build/test commands used and results.
7. Update `TODO.md` to reflect completion status (mandatory).

## 7) Output format expectations
When responding with code changes:
- Provide a short rationale.
- Include file paths for edits.
- Keep changes minimal and localized.
- If tradeoffs exist, present them clearly and recommend one.

## 8) If blocked
If information is missing (e.g., unclear architecture, missing TODO detail):
- Ask targeted questions.
- Do not guess large architectural decisions.


## 9) Don't stop
Continue down the TODO.md until everything is implemented unless you encounter a block.
- You should only stop implementing if you run into a block you cannot resolve.
- If you encounter this then ask the user for guidance.