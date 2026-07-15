// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import sitemap from '@astrojs/sitemap';

// https://astro.build/config
export default defineConfig({
	site: 'https://assura.dev',
	redirects: {
		'/docs/configuration': '/reference/configuration',
		'/docs/rules': '/reference/rules',
		'/reference/config-v2': '/reference/configuration',
		'/docs/api': '/reference/api',
		'/examples/basic': '/examples/basic-setup',
	},
	integrations: [
		sitemap(),
		starlight({
			title: 'Assura',
			logo: {
				src: './src/assets/assura-logo.svg',
				alt: '',
			},
			description: 'Structure-first repository validation and project intelligence',
			social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/rothnic/assura' }],
			customCss: [
				'./src/styles/custom.css',
			],
			sidebar: [
				{
					label: 'Overview',
					items: [
						{ label: 'Introduction', slug: 'introduction' },
						{ label: 'Why Assura?', slug: 'why-assura' },
						{ label: 'Roadmap', slug: 'roadmap' },
					],
				},
				{
					label: 'Getting Started',
					items: [
						{ label: 'Getting Started Guide', slug: 'guides/getting-started' },
						{ label: 'Quick Start', slug: 'guides/quickstart' },
						{ label: 'Agent-Ready Onboarding', slug: 'guides/agent-ready-onboarding' },
						{ label: 'Installation', slug: 'guides/installation' },
						{ label: 'Adoption Walkthrough', slug: 'guides/adoption-walkthrough' },
						{ label: 'LS-Lint Migration', slug: 'guides/ls-lint-migration' },
					],
				},
				{
					label: 'Product Layers',
					items: [
						{ label: 'Structure Validation', slug: 'product/structure-validation' },
						{ label: 'Markdown Validation', slug: 'product/markdown-validation' },
						{ label: 'Content Runtime And Models', slug: 'product/content-models' },
						{ label: 'Query And Search', slug: 'product/query-search' },
						{ label: 'Code Intelligence', slug: 'product/code-intelligence' },
						{ label: 'Agent And Editor Surfaces', slug: 'product/agent-editor-surfaces' },
					],
				},
				{
					label: 'Reference',
					items: [
						{ label: 'Configuration Reference', slug: 'reference/configuration' },
						{ label: 'Rules Reference', slug: 'reference/rules' },
						{ label: 'API Reference', slug: 'reference/api' },
						{ label: 'Agent Feedback Delivery', slug: 'reference/agent-feedback' },
						{ label: 'Release Readiness', slug: 'reference/release-readiness' },
						{ label: 'Performance', slug: 'reference/performance' },
						{ label: 'Performance Test Cases', slug: 'reference/performance-test-cases' },
						{ label: 'Performance Implementation', slug: 'reference/performance-implementation' },
					],
				},
				{
					label: 'Examples',
					items: [
						{ label: 'Basic Project Setup', slug: 'examples/basic-setup' },
						{ label: 'Real Project Feedback', slug: 'examples/real-project-feedback' },
						{ label: 'Custom Constraints', slug: 'examples/custom-constraints' },
						{ label: 'CI/CD Integration', slug: 'examples/ci-cd-integration' },
						{ label: 'Git Hooks Setup', slug: 'examples/git-hooks-setup' },
						{ label: 'Multi-Agent Configuration', slug: 'examples/multi-agent-config' },
						{ label: 'Content Runtime', slug: 'examples/content-runtime' },
						{ label: 'Project Intelligence Demo', slug: 'examples/project-intelligence-demo' },
					],
				},
			],
		}),
	],
});
