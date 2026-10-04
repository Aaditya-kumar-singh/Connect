# Agent Instructions for YBM Connect

This repository follows **GitHub Spec Kit** (Spec-Driven Development / SDD) workflow. Both ChatGPT and Gemini collaborate on this project.

## Spec Kit Skills & Workflow
All Spec Kit skills are available under `.agents/skills/` and scripts under `.specify/scripts/powershell/`:

1. **/speckit-constitution**: Consult and update `.specify/memory/constitution.md` for project architecture rules and tech stack constraints.
2. **/speckit-specify**: Create or update specifications in `.specify/` before implementing new features.
3. **/speckit-clarify**: Surface and resolve ambiguities before formulating implementation plans.
4. **/speckit-plan**: Create an architecture and technical implementation plan (`plan.md`).
5. **/speckit-checklist**: Run quality checks on the specifications.
6. **/speckit-tasks**: Decompose the plan into structured, dependency-ordered tasks (`tasks.md`).
7. **/speckit-analyze**: Verify cross-artifact consistency across spec, plan, and tasks.
8. **/speckit-implement**: Execute tasks sequentially with tests and quality gates.
9. **/speckit-converge**: Assess the codebase against specifications and reconcile any gaps.

## Technology Boundaries: Rust + Erlang/OTP

This project uses a dual-runtime architecture (see ADR-018):

- **Rust** is the primary application backend language for all business logic, API endpoints, data access, WebSocket/WebRTC protocols, and client-facing services.
- **Erlang/OTP** is used exclusively for the reliability/fault-tolerance sidecar defined in Phase 15 (`specs/015-fault-tolerance/`): OTP supervision trees, dependency health monitors, circuit breakers, and recovery coordination.

### AI Agent Rules for Erlang/OTP

1. **Do NOT introduce Erlang into application or business logic.** Erlang must only be used for reliability responsibilities defined in the Phase 15 specification.
2. **Do NOT rewrite Rust modules in Erlang.** Existing Rust code is the authoritative implementation for all application features.
3. **Do NOT add Erlang dependencies without architectural justification.** Follow the same dependency discipline as Rust (Rule 4).
4. **The Rust/Erlang boundary follows the Phase 15 specification.** Communication is via Redis Pub/Sub channels only (`ybm:reliability:events`, `ybm:reliability:status`).
5. **Ponytail rules apply equally to Erlang code.** Minimal implementation, no speculative features, no unnecessary abstractions.
6. **Erlang tests use rebar3 eunit and rebar3 ct.** Same quality standards as Rust tests.

## Mandatory AI Development Policy: Spec Kit + Ponytail

**This policy is mandatory for every AI agent and every AI-generated change in this repository.** It applies to ChatGPT, Codex, Gemini, Claude, Copilot, Cursor, and any other coding agent that reads these instructions.

### Rule 0: No direct coding before understanding the task
An AI must not start writing code immediately after receiving a request. It must first understand the requested outcome, identify the affected area, and inspect the relevant existing code.

### Rule 1: Spec Kit controls WHAT is built
For a new feature or meaningful change, follow this order:
**Constitution → Specify → Clarify (when needed) → Plan → Checklist → Tasks → Analyze → Implement → Converge**

The current specification, plan, and tasks are the source of truth for implementation scope.

If the requested work already has an existing specification/task, use and update that artifact when appropriate instead of creating a parallel design.

### Rule 2: Ponytail controls HOW MUCH is built
Implement the smallest complete solution that satisfies the approved specification/task.

AI must not:
- invent extra features;
- create speculative future architecture;
- add unnecessary abstractions;
- add duplicate helpers/services/components;
- add dependencies without justification;
- rewrite unrelated code;
- perform opportunistic refactors;
- create placeholder implementations for future phases;
- add configuration that is not required;
- generate boilerplate merely because it is conventional;
- change APIs, schemas, file structure, naming, or architecture without a requirement.

### Rule 3: Mandatory repository inspection
Before implementation, the AI must search for:
- existing implementations;
- reusable functions/types/components/services;
- existing dependencies;
- existing patterns;
- related tests;
- relevant specifications, plans, and tasks.

**Reuse before create. Extend before duplicate.**

### Rule 4: Dependency discipline
Before adding a dependency, prove that:
1. the requirement cannot reasonably be satisfied with existing code;
2. the standard library/platform cannot satisfy it;
3. an already-installed dependency cannot satisfy it;
4. the new dependency is justified by the specification or architecture.

Do not add a dependency merely for convenience.

### Rule 5: Scope lock
Only modify files necessary for the current task.

If an unrelated issue is discovered:
- do not silently fix it;
- record it separately;
- continue the requested task without expanding scope.

### Rule 6: Ambiguity gate
If the requirement materially affects architecture, security, data models, APIs, or scope and is ambiguous, clarify it before implementing the expanded interpretation.

Never assume extra requirements.

### Rule 7: Security and correctness are mandatory
Minimalism never permits removal of necessary:
- authentication and authorization;
- validation;
- input/output safety;
- data-integrity protections;
- error handling;
- rate limiting;
- privacy protections;
- accessibility;
- required observability;
- tests and quality gates.

### Rule 8: Implementation discipline
During implementation:
- make the smallest coherent patch;
- preserve existing working behavior;
- avoid unrelated formatting changes;
- avoid unnecessary comments;
- avoid duplicate error handling;
- avoid unnecessary wrappers;
- avoid premature optimization;
- avoid premature abstraction;
- do not build infrastructure for hypothetical future requirements.

### Rule 9: Test the actual requirement
Tests must verify the behavior required by the current specification/task.

Do not create large test frameworks or broad test suites unrelated to the change.

Do not weaken or delete tests simply to make a change pass.

### Rule 10: Mandatory pre-completion audit
Before declaring the task complete, the AI must:
1. run relevant tests/checks;
2. inspect the final diff;
3. inspect changed files for accidental or unrelated changes;
4. compare the implementation against the specification, plan, and tasks;
5. remove unnecessary code;
6. verify no speculative functionality was introduced;
7. report any remaining unrelated issues separately.

### Rule 11: Stop conditions
An AI must stop implementing when the current specification/task is fully satisfied.

Do not continue adding "improvements", "nice-to-haves", future-proofing, extra abstractions, or additional features unless they are explicitly requested or added to the approved project scope.

### Rule 12: Conflict resolution
When instructions appear to conflict:
1. project security and correctness requirements;
2. current approved specification/constitution;
3. current implementation plan and tasks;
4. this Spec Kit + Ponytail policy;
5. user-requested implementation details;
6. optional improvements.

When a user explicitly requests additional scope, update the relevant Spec Kit artifact when required, then implement only that approved scope.

### AI response requirement
When reporting completed work, the AI should briefly state:
- what requirement was implemented;
- which files were changed;
- what verification was run;
- any unrelated issues discovered but intentionally left untouched.

**Default behavior: inspect → specify → plan → task → search/reuse → minimal implementation → verify → diff audit → stop.**

## Minimal-Change / Ponytail Engineering Rules

These rules apply to every implementation, refactor, bug fix, and generated change.

### Core principle
Implement the smallest change that fully satisfies the current requirement. Do not generate speculative code, future features, duplicate abstractions, unnecessary files, unnecessary dependencies, or unrelated refactors.

### Mandatory decision order
1. Read the relevant specification, plan, tasks, and existing implementation before changing code.
2. Search the repository for an existing implementation, helper, type, service, component, dependency, or pattern that can be reused.
3. Prefer modifying existing code over creating a new abstraction when the existing design can support the requirement cleanly.
4. Prefer the Rust/TypeScript standard library and existing project dependencies before adding a dependency.
5. Prefer native/platform capabilities before introducing wrappers.
6. Add a new file, abstraction, dependency, service, or framework only when the requirement or architecture clearly requires it.
7. Keep the patch narrowly scoped to the requested task.
8. Do not refactor unrelated code merely because it could be improved.
9. Do not add comments, documentation, configuration, tests, logging, error types, utilities, or abstractions unless they provide concrete value for the current requirement or quality gate.
10. Before finishing, inspect the diff and remove every change that is not required.

### Anti-overengineering gate
Before adding anything, ask:
- Is this required by the current spec/task?
- Does equivalent functionality already exist?
- Can the existing implementation be extended instead?
- Can the standard library or an installed dependency solve it?
- Is this needed now, or only potentially useful later?

If the answer is only "potentially useful later", do not implement it.

### Safety and correctness exceptions
Minimal code must still preserve security, validation, authorization, data integrity, error handling, accessibility, observability required by the specification, and tests required by the quality gate. Never remove necessary safeguards just to reduce line count.

### Change discipline
- Do not silently broaden task scope.
- Do not rewrite working modules without a requirement.
- Do not rename public APIs or move files unless required.
- Do not introduce duplicate helpers or parallel implementations.
- Do not add a library for functionality already available in the project.
- Do not generate placeholder implementations for future roadmap phases.
- Do not implement undocumented behavior merely because it seems useful.
- If the requirement is ambiguous, clarify it before expanding the implementation.

### Completion review
After implementation:
1. Run the relevant formatter/checker/tests.
2. Inspect `git diff`.
3. Compare every changed file against the current task.
4. Remove unnecessary code and unrelated formatting changes.
5. Confirm the implementation satisfies the specification without adding future scope.
<!-- SPECKIT START -->
For additional context about technologies to be used, project structure,
shell commands, and other important information, read the current plan
at specs/001-backend-infra-health/plan.md
<!-- SPECKIT END -->
