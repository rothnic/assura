export const installCommand = 'curl -fsSL https://assura.dev/install.sh | sh';

export const onboardingCommand = 'assura agent onboard .';

export const agentSetupSteps = [
  'Inspect manifests, tooling, generated output, and the intentional layout.',
  'Define project-owned rules for the expected stack. Close stable scopes so unexpected paths fail; preserve legitimate paths and ask only where evidence is ambiguous.',
  'Verify with assura review and assura check --format agent.',
];

export const agentSetupInstruction = `Set up Assura for this project. Run \`${onboardingCommand}\`. ${agentSetupSteps.join(' ')}`;

export const agentSetupPrompt = `Install the latest published Assura release:
${installCommand}

${agentSetupInstruction}

Read .assura/onboarding/agent-next.md and follow its evidence-first handoff. If one supported agent host is clear, activate its project-local integration; otherwise leave activation as an explicit next step.`;

export const marketingNav = [
  { label: 'Review', href: '/project-review/' },
  { label: 'Guardrails', href: '/ai-coding-agent-guardrails/' },
  { label: 'Compare', href: '/compare/ls-lint/' },
  { label: 'Performance', href: '/performance/' },
  { label: 'Docs', href: '/guides/quickstart/', mobile: true },
  { label: 'About', href: '/about/' },
];
