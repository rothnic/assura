export const installScriptUrl = 'https://assura.dev/install.sh';
export const installCommand = `curl -fsSL ${installScriptUrl} | sh`;

export const agentSetupPrompt = `Install Assura in this repository and set it up for agent-ready checks.
If assura is not installed, run:
ASSURA_INSTALL=${installScriptUrl}
curl -fsSL "$ASSURA_INSTALL" | sh

Then run:
identify the active host as codex, claude, opencode, or pi; do not guess if the evidence is ambiguous
assura agent onboard . --agent codex --activate --format json
assura check --format json .

If the active host is not Codex, replace codex with the detected host before running the command. Read .assura/onboarding/agent-next.md, summarize what is active, what is inactive, and what questions need human answers before adding project-specific rules.`;

export const marketingNav = [
  { label: 'Review', href: '/project-review/' },
  { label: 'Guardrails', href: '/ai-coding-agent-guardrails/' },
  { label: 'Compare', href: '/compare/ls-lint/' },
  { label: 'Performance', href: '/performance/' },
  { label: 'Docs', href: '/guides/quickstart/', mobile: true },
  { label: 'About', href: '/about/' },
];
