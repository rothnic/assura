// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import catppuccin from '@catppuccin/starlight';

// https://astro.build/config
export default defineConfig({
	integrations: [
		starlight({
			title: 'Assura',
			description: 'Dependency-aware file system validation engine',
			social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/rothnic/assura' }],
			plugins: [
				catppuccin({
					flavor: 'mocha',
				}),
			],
			customCss: [
				'./src/styles/custom.css',
			],
			sidebar: [
				{
					label: 'Overview',
					items: [
						{ label: 'Introduction', slug: 'introduction' },
						{ label: 'Why Assura?', slug: 'why-assura' },
					],
				},
				{
					label: 'Getting Started',
					items: [
						{ label: 'Getting Started Guide', slug: 'guides/getting-started' },
						{ label: 'Quick Start', slug: 'guides/quickstart' },
						{ label: 'Installation', slug: 'guides/installation' },
						{ label: 'LS-Lint Migration', slug: 'guides/ls-lint-migration' },
					],
				},
				{
					label: 'Guides',
					items: [
						{ label: 'Configuration', slug: 'docs/configuration' },
						{ label: 'Rules', slug: 'docs/rules' },
					],
				},
				{
					label: 'Reference',
					items: [
						{ label: 'Configuration Reference', slug: 'reference/configuration' },
						{ label: 'API Reference', slug: 'reference/api' },
						{ label: 'Performance', slug: 'reference/performance' },
						{ label: 'Performance Test Cases', slug: 'reference/performance-test-cases' },
						{ label: 'Performance Implementation', slug: 'reference/performance-implementation' },
					],
				},
				{
					label: 'Examples',
					items: [
						{ label: 'Basic Project Setup', slug: 'examples/basic-setup' },
						{ label: 'Custom Constraints', slug: 'examples/custom-constraints' },
						{ label: 'CI/CD Integration', slug: 'examples/ci-cd-integration' },
						{ label: 'Git Hooks Setup', slug: 'examples/git-hooks-setup' },
						{ label: 'Multi-Agent Configuration', slug: 'examples/multi-agent-config' },
					],
				},
			],
		}),
	],
});
