export const installScriptUrl = 'https://assura.dev/install.sh';
export const installCommand = `curl -fsSL ${installScriptUrl} | sh`;

export const agentSetupPrompt = `Install Assura in this repository and set it up for agent-ready checks.
If assura is not installed, run:
ASSURA_INSTALL=${installScriptUrl}
curl -fsSL "$ASSURA_INSTALL" | sh

Then run:
assura agent onboard . --agent auto --format json
assura check --format json .

Read .assura/onboarding/agent-next.md, summarize what is active, what is inactive, and what questions need human answers before adding project-specific rules.`;

export const marketingNav = [
  { label: 'Review', href: '/project-review/' },
  { label: 'Guardrails', href: '/ai-coding-agent-guardrails/' },
  { label: 'Compare', href: '/compare/ls-lint/' },
  { label: 'Performance', href: '/performance/' },
  { label: 'Docs', href: '/guides/quickstart/', mobile: true },
  { label: 'About', href: '/about/' },
];
