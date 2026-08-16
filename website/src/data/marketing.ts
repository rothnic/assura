export const sourceRepositoryUrl = 'https://github.com/rothnic/assura';
export const sourcePreviewRevision = '5dcd7d702da5cd46f729fb737e8e472031e402fd';
export const installCommand = `cargo install --git ${sourceRepositoryUrl} --rev ${sourcePreviewRevision} --locked --bin assura assura`;

export const agentSetupPrompt = `Install the exact Assura revision used to generate and verify the examples on assura.dev:
${installCommand}

Then run:
identify the active host as codex, claude, opencode, or pi; ask if the evidence is ambiguous
assura agent onboard . --agent <host> --activate --format json
assura check --format json .

Replace <host> with the detected host. Read .assura/onboarding/agent-next.md, summarize what is active, what is inactive, and what questions need human answers before adding project-specific rules.`;

export const marketingNav = [
  { label: 'Review', href: '/project-review/' },
  { label: 'Guardrails', href: '/ai-coding-agent-guardrails/' },
  { label: 'Compare', href: '/compare/ls-lint/' },
  { label: 'Performance', href: '/performance/' },
  { label: 'Docs', href: '/guides/quickstart/', mobile: true },
  { label: 'About', href: '/about/' },
];
