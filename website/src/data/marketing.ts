export const sourceRepositoryUrl = 'https://github.com/rothnic/assura';
export const sourcePreviewRevision = 'cbcd2b4855a7ac958e27d962371dcbe97f4ff625';
export const installCommand = `cargo install --git ${sourceRepositoryUrl} --rev ${sourcePreviewRevision} --locked --bin assura assura`;

export const onboardingCommand = 'assura agent onboard .';

export const agentSetupSteps = [
  'Inspect manifests, tooling, generated output, and the intentional layout.',
  'Define project-owned rules for the expected stack. Close stable scopes so unexpected paths fail; preserve legitimate paths and ask only where evidence is ambiguous.',
  'Verify with assura review and assura check --format agent.',
];

export const agentSetupInstruction = `Set up Assura for this project. Run \`${onboardingCommand}\`. ${agentSetupSteps.join(' ')}`;

export const agentSetupPrompt = `Install the exact Assura revision used to verify assura.dev:
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
