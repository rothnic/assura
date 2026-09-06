# Website, portfolio and launch strategy

Live review: 2026-09-04. Objective: technical product / AI systems leadership. Recommendations are proposals; no site changes or posts were published.

## Positioning

Working product description: **Assura makes repository conventions executable, so coding agents can catch structural drift while they work.**

Supporting explanation: define local rules for layout, naming, required files and agent guidance; reuse suitable patterns; receive concise repair context; run the same deterministic checks in CI. The agent chooses the architecture within repository intent. Assura makes agreed conventions inspectable and enforceable.

This is a sharper entry point than a general project-intelligence or quality platform. Prefer examples of a wrong-path file, missing test relationship or broken guidance reference over a broad claim to “make AI trustworthy.” A passing structural policy cannot establish secure code, good architecture or correct business behavior.

[LS-Lint](https://ls-lint.org/) is a good choice when naming rules suffice. [pre-commit](https://pre-commit.com/) already manages multi-language hooks. [Semgrep](https://docs.semgrep.dev/) addresses source analysis. Assura should show the additional repository convention and agent-feedback use case, while coexisting with those tools. Do not imply other tools or teams cannot run checks during editing.

## Live site findings

| Surface | What works | Highest-value refinement |
| --- | --- | --- |
| [assura.dev](https://assura.dev/) | Polished restrained design; clear Review/Check distinction; real config/output examples; explicit configured/unchecked states | Shorten repeated signal/drift sections; place release status and an executable start path near the CTA; demonstrate a complete setup/change/repair loop |
| Assura performance strip | Cold and warm comparisons are labeled and linked to methodology | Do not lead with 1.18x/15.91x while the current master performance gate fails. Date/version/hardware/sample scope belong beside claims; show absolute times and limitations; warm-vs-cold is workflow evidence, not equal-execution speed evidence |
| Assura onboarding | Honest about unresolved specialization | Current page leaves naming/framework choices to human decisions. Show how an agent infers low-risk choices and records them; distinguish the future promise from current behavior |
| [Assura About](https://assura.dev/about/) | Already connects Nick's product/systems judgment with the tool and links both personal site and LinkedIn | Two Start links use `/about/#onboard`, but the DOM has no `onboard` element. Repair to an actual onboarding destination. Replace some principles-only language with one concrete result and case-study link |
| [NickRoth.com](https://www.nickroth.com/) | Distinctive personality; explicit product/AI positioning | Feature Assura as a major systems/product case study rather than leading only with portfolio-building and resume-tool cards |
| [Work index](https://www.nickroth.com/work/) | Numerous concrete technical articles and some business outcome cases | No visible Assura entry. Group related implementation posts beneath larger case studies; make product ownership, tradeoffs and outcomes easier for a hiring reader to find |

This was a desktop content/journey review, not a full accessibility, mobile or Core Web Vitals audit. The accessibility tree repeats decorative name/tech text on Nick's site; inspect assistive-technology semantics during the next UI pass before calling it an accessibility defect. Search queries returned no useful results for these domains, but that alone does not prove indexing failure.

Avoid another full visual redesign. The highest return is credible content, repaired navigation and release-aligned onboarding. On Assura, keep a compact sequence: problem + one concrete example; runnable demo; first setup; fit/boundaries; evidence; feedback. Move deeper detail to docs.

## Cross-site content architecture

Proposed canonical case study: `https://www.nickroth.com/work/assura` (not yet created). Assura owns product instructions and technical reference. Nick's site owns the leadership story. Link them contextually rather than duplicating the same article on both domains.

Execution discovery supersedes the proposed new slug: an existing `codex/assura-case-study` worktree contains `src/content/work/assura-agentic-project-validation.mdx`. W03 in the execution backlog reuses that article and its canonical route, refreshes the PR/deployment state and investigates missing live visibility before creating any new content path.

Case study outline:

1. Repeated observed problem: agents follow individual instructions but repository conventions drift between them.
2. Nick's responsibility: problem framing, prioritization, architecture/UX decisions, evaluation design, release and contribution standards. Identify collaborators and AI assistance honestly.
3. Scope choice: repository policy and workflow integration; reasons for postponing intelligence/semantic search/agent orchestration.
4. Evidence: initially clean checks missed intended naming constraints and hook choices varied. Show the independent acceptance rubric and what changed afterward.
5. Technical judgment: shared policy model, bounded feedback, native-tool integration, cold/warm distinction and a maintenance-cost tradeoff.
6. Outcome: measured pilot results with sample sizes and limitations. Until outside results exist, label the piece an engineering case study in progress.
7. What was removed or deferred after learning; next experiment; a relevant contact CTA.

Publish a short primary case study plus two supporting notes: “What a green configuration check does not prove” and “Designing contribution gates when agents can produce patches faster than people can review them.” Put detailed benchmark methodology on Assura and link to it.

Link plan: Assura About/footer → case study; Nick homepage/Work → case study → Assura quickstart and GitHub; GitHub README → product docs and a short creator/case-study link; LinkedIn Featured → case study and working demo. Keep the product's primary CTA about using Assura.

SEO/discovery tasks: verify canonical redirects, sitemap and robots, distinct page titles/descriptions, useful social preview images, crawlable internal links, and author/software structured data where accurate. Do not add fabricated ratings or outcome schema. Prefer task-specific pages supported by real examples over many near-duplicate keyword pages. Inspect Search Console before drawing indexing conclusions.

Measurement: privacy-conscious aggregate tagged campaign links, demo/quickstart clicks, case-study visits and qualified contact events. Use separate campaign identifiers for LinkedIn launch, follow-up and a specific permitted community post. Avoid tracking visitors across sites by identity. Product adoption needs voluntary usage/pilot evidence, not click attribution alone.

## Launch sequence

| Timing/condition | Channel | Purpose and content |
| --- | --- | --- |
| Now, before broad promotion | Existing peers, 3–5 maintainers | Personally invite a bounded setup test. Observe friction and ask for a counterexample. Do not label this a mature release |
| After stage-0 release truth and a demo | LinkedIn personal profile | Explain the problem and one product decision, link the case study, invite specific practitioners to test |
| After independent setup proof | GitHub Release/README/discussions or issues | Publish reproducible examples, support matrix, known limits and feedback templates; pin a concise feedback route |
| After meaningful engineering evidence and code cleanup | r/rust, only if current rules and moderator guidance allow | Author a technical discussion around a concrete Rust design/performance tradeoff and what was learned; disclose ownership/AI assistance |
| After evidence of demand and stable installation | Show HN, potentially | Consider a directly usable tool with a short demo and clear limitations. Recheck venue rules then; no sign-up barrier merely to try a local CLI |
| After results or a significant repair | LinkedIn follow-up / original feedback thread where allowed | Close the loop: what feedback changed, failures retained in the evaluation and new result. Do not repost the launch repeatedly |

LinkedIn initial cadence proposal: one substantial post, then a results/learning follow-up after 1–2 weeks when there is new evidence. Post when Nick can answer questions that day; there is no verified universal best hour or algorithm trick in this assessment. Keep the work in Featured and in the relevant project/experience section with an accurate status.

Draft LinkedIn post, for Nick to edit with his own experience before publishing:

> I’m building Assura around a problem I keep encountering with coding agents: writing conventions down does not guarantee that a project stays within them.
>
> Assura turns repository structure and guidance rules into local checks with concrete repair feedback. The product decision I’m testing is how much it can handle reliably before another layer of automation becomes more maintenance than help.
>
> Our first small initialization trials produced passing checks but inconsistent hook setup and incomplete naming policies. That changed the evaluation: setup now needs to prove it catches intentional violations and supports a later feature change, not just produce a green report.
>
> I’m looking for a few maintainers using coding agents who would try this on a small repository and show me where the workflow falls short. The case study covers the design choices, experiment and current limits: [case study link once live].

Use actual measured improvement in a later post only after collecting it. A short screen recording should show the failing path, relevant policy, exact diagnostic and repair—not a dashboard of arbitrary scores.

## Reddit: selective participation, no promise of avoiding criticism

There is no way to guarantee a friendly reception. Reduce avoidable objections with utility, truthful claims, visible ownership and evidence. Do not disguise promotion as an independent discovery or hide AI assistance.

Rules retrieved on 2026-09-04 through web results are snapshots and must be checked again immediately before posting:

- [r/rust rules](https://www.reddit.com/r/rust/about/rules.json): posts must be relevant and thoughtful; submissions appearing AI-generated may be removed at moderator discretion. A technical tradeoff with code, reproducible data and human-authored discussion is a possible fit, not guaranteed permission.
- [r/opensource rules](https://www.reddit.com/r/opensource/about/rules.json): excessive promotion is restricted, promotional flair is required for sharing a project, and AI-generated content is described as ban-worthy. Exclude it from the default launch plan; seek clarification if considering a post about an AI-assisted codebase. Do not paste generated marketing copy there.
- [r/LocalLLaMA rules](https://www.reddit.com/r/LocalLLaMA/about/rules.json): LLM relevance, ownership disclosure, limited promotion, and restrictions on primarily generated copy/code. A general structural CLI is a weak fit. Reconsider only for a relevant local-model experiment and clear venue permission.

Other agent-specific communities may fit better, but none is approved by this review: verify their own rules, recurring showcase threads and recent moderation before choosing. Do not shotgun the same content into several subs, manufacture engagement, solicit upvotes, use alternate accounts, or automate replies. Participation should be useful even when Assura is not mentioned.

Possible technical topic for Nick to author: “What I learned separating a Rust CLI's cold check path from its warm agent feedback loop.” Include fair comparison conditions and remaining failures. Ask one genuine question, such as which repository convention their current toolchain cannot express. A title alone does not make a post acceptable; current content rules still apply.

Response approach: acknowledge valid limitations, request minimal repros for defects, explain deliberate boundaries with evidence, and thank useful counterexamples. Avoid debating hostility. Log actionable feedback, fix it, and reply with a result when allowed. Human-authored posts are not a technique for concealing generated code.

## Feedback and announcement gates

Use a short feedback template: Assura version; stack/harness; attempted outcome; config/reproducer; expected versus actual; setup time/extra prompts; whether the user would keep it enabled. Ask users to redact private paths/content; accept minimal public toy reproductions.

Classify feedback into install defects, policy expressiveness, false positives, missed violations, confusing instructions, runtime overhead and out-of-scope requests. Review weekly. Publish a small “what changed after feedback” note. Broader feature requests require repeated evidence from target maintainers, not a single enthusiastic comment.

Before broad promotion: working advertised install; current support matrix; release CI truth; 60–90 second reproducible demo; useful README; accountable maintainer identity; contributor guide; open license; known limitations; a feedback route Nick can monitor. Code review should show concrete maintenance improvements, not merely cosmetic removal of AI phrasing.
