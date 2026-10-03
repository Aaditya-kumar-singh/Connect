# Gemini Assistant Instructions for YBM Connect

This repository follows **GitHub Spec Kit** (Spec-Driven Development / SDD) workflow. Both Gemini and ChatGPT collaborate on this project.

## Spec Kit Commands & Workflow
All Spec Kit commands for Gemini are defined in `.gemini/commands/` and executable via the PowerShell scripts in `.specify/scripts/powershell/`:

1. **speckit.constitution**: Project principles and architecture constraints (`.specify/memory/constitution.md`).
2. **speckit.specify**: Create or update specifications in `.specify/` before implementing new features.
3. **speckit.clarify**: Surface and resolve ambiguities before formulating implementation plans.
4. **speckit.plan**: Create an architecture and technical implementation plan (`plan.md`).
5. **speckit.checklist**: Run quality checks on the specifications.
6. **speckit.tasks**: Decompose the plan into structured, dependency-ordered tasks (`tasks.md`).
7. **speckit.analyze**: Verify cross-artifact consistency across spec, plan, and tasks.
8. **speckit.implement**: Execute tasks sequentially with tests and quality gates.
9. **speckit.converge**: Assess the codebase against specifications and reconcile any gaps.

<!-- SPECKIT START -->
For additional context about technologies to be used, project structure,
shell commands, and other important information, read the current plan
at specs/001-backend-infra-health/plan.md
<!-- SPECKIT END -->
