OpenCode Starter Tasks

Branch: opencode/frontend-tui

Goal: Build the React Ink TUI for MVP against openapi.yaml and the stubbed backend.

Priority checklist:

1. Generate TypeScript client/types from openapi.yaml (if available). If not, hand-author minimal types for AnalyzeResponse and SSE chunks.
2. Scaffold frontend with package.json, tsconfig.json (already created).
3. Implement CLI entry src/cli.tsx that renders <App /> (already created).
4. Implement components incrementally (micro-commits): Header, ErrorOutput, CodeBlock, StreamingText, SolutionBlock, InputPrompt, ShortcutBar.
5. Use Zustand store with analyze(), appendChunk(), reAnalyze(), copyFix() actions.
6. Connect to backend: POST /api/analyze and GET /api/analyze/:id/stream (SSE). Use stubbed SSE for initial development.
7. Tests: add a small startup/smoke snapshot for the CLI.

Micro-commit rules:
- Commit often: one logical change per commit.
- Use Conventional Commit format and include [opencode] tag. Example:
  feat(streaming): add StreamingText component [opencode]

Dev command:
- cd frontend
- npm install
- npm run dev -- --cmd "ls /nonexistent"

Acceptance criteria:
- Running the CLI shows the health status and command.
- It connects to the stubbed SSE when you POST /api/analyze and streams chunks.
- Keep commits small and push frequently.

Starter prompt (for an OpenCode agent):

You are OpenCode — integrator + frontend agent for Quack v2. Branch: opencode/frontend-tui.
Goal: Build the React Ink TUI for MVP against openapi.yaml and the stubbed backend.

(See PRD.md and openapi.yaml at repo root for API contract.)

Please follow the tasks above and open PRs to master when components compile and basic flows work.
