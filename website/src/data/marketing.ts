import releaseSurfaces from '../../../docs/data/release-surfaces.json' with { type: 'json' };

export const installCommand = 'curl -fsSL https://assura.dev/install.sh | sh';

export const releaseStatus = `Published ${releaseSurfaces.current_published_release}.`;
export const releaseScope = 'The installer provides the published release. This source checkout may contain newer changes. Keep assura and assura-full together, confirm the selected version, and follow any installer PATH correction.';
export const verificationCommand = 'assura --version && assura-full --version';
export const onboardingCommand = 'assura agent onboard .';
export const reviewCommand = 'assura review .';
export const checkCommand = 'assura check --format agent .';

export const agentSetupSteps = [
  'Inspect manifests, tooling, generated output, and the intentional layout.',
  'Define project-owned rules for the expected stack. Close stable scopes so unexpected paths fail; preserve legitimate paths and ask only where evidence is ambiguous.',
  'Verify with assura review and assura check --format agent.',
];

export const agentSetupInstruction = `Set up Assura for this project. Run \`${onboardingCommand}\`. ${agentSetupSteps.join(' ')}`;

export const agentSetupPrompt = `${releaseStatus}
${releaseScope}

Install the published Assura release:
${installCommand}

Verify both selected executables and their versions before continuing:
${verificationCommand}

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
