# Website, portfolio and adoption work packet

Follow [execution constraints](execution-backlog.md). Website/code drafts are executable local work; contacting people, publishing posts, merges and deployments require explicit authority. Produce the finished reviewable material before stopping for that final action.

## W01

**Outcome:** Every marketing Start CTA reaches a real usable onboarding destination. **Own:** `website/src/components/marketing/marketing-layout.astro`, `focused-marketing-page.astro`, affected pages, `website/src/tests/marketing.spec.ts`.

- [ ] Read CTA defaults. Set shared Start links to `/#onboard` when intentionally referring to the homepage section; use `/guides/agent-ready-onboarding/` for the explicit documentation CTA. A page-relative `#onboard` is allowed only when that page renders the section.
- [ ] Add a parameter/default for CTA destination only if the shared component needs it; update About and other focused pages through that one contract instead of patching each rendered link separately.
- [ ] Extend Playwright marketing tests to visit every marketing route, collect local fragment links, resolve destination path plus fragment and assert the destination element exists. Avoid requiring all cross-domain links to load in local tests.
- [ ] Run `pnpm --dir website test:marketing` and `cargo xtask docs`; manually click Start from homepage and About with keyboard at desktop and mobile size.

**Accept:** `/about/#onboard` is no longer a dead destination; other focused pages have no analogous broken CTA; the target shows current-version setup instructions.

## W02

**Outcome:** A shorter product story with claims that match installable behavior. **Own:** `website/src/pages/index.astro`, marketing components, `website/src/data/performance-summary.json` via its generator, relevant docs and `README.md`.

- [ ] Use the primary copy: “Make repository conventions executable.” Supporting sentence: “Assura gives coding agents precise feedback on project structure, naming and guidance, with the same local policy available in CI.” Keep a concrete wrong-path/naming example near it.
- [ ] Arrange content in this order: problem plus example; runnable config/diagnostic demo; version-aware start; where Assura fits; evidence and limitations; feedback/creator links. Remove repeated signal/drift explanations without deleting supporting technical docs.
- [ ] Show published/candidate status near the first command. Until a candidate is published, provide an explicitly labeled source-build path or supported published alternative. Never present a candidate-only command as available from the default installer.
- [ ] For performance figures, require generator metadata for source SHA, version, date, hardware, fixture count, path and measurement type. If valid current evidence is unavailable/failing, render “Performance evidence under review” plus methodology; do not retain an unqualified speed headline. Preserve historical results labeled by version.
- [ ] Generate one config → violation → repair → check demo from real CLI output using `cargo xtask website-demo-data --check` and the existing generator. Display the difference between example output and interactive execution honestly.
- [ ] Run generated-demo/config-example checks, docs build, marketing tests and manual mobile review. Verify site still clearly states that native linters/tests remain necessary.

**Accept:** Every displayed command is runnable by its displayed install route; each number has provenance; no unsupported universal productivity/security/quality claim; the first-time route is understandable without reading the roadmap.

## W03

**Outcome:** Finish the existing Assura leadership case study rather than duplicate it. **Repo:** `rothnic/nickroth`. **Existing work:** `/Users/nroth/workspace/nickroth-assura-case-study`, branch `codex/assura-case-study`, observed HEAD `19dc2bc`. **Own:** `src/content/work/assura-agentic-project-validation.mdx`, related project image, schema/index only if visibility needs repair.

- [ ] Refresh remote/default branch/PR/deployment state, read that repo's AGENTS and task instructions, and preserve other worktrees. The article already exists and is `featured: true`; determine why it is absent from the live Work index before changing sorting or writing a new slug.
- [ ] Preserve the existing canonical slug `/work/assura-agentic-project-validation/` unless a deliberate redirect decision changes it. Do not introduce `/work/assura` as a duplicate. Verify the actual router's trailing-slash behavior.
- [ ] Revise the article around Nick's role: observed problem; competing options; scope choice; experiment; unfavorable early result; change made; current evidence; next decision. Explain that agents choose reversible structure from evidence; remove the older assertion that routine layout/naming/hook choices should always wait for human answers.
- [ ] Replace unqualified outcomes such as “early feedback changes design quality” with measured results when available or a clearly stated hypothesis. Check the claimed Lighthouse 100s and release-bound performance against original dated evidence; retain as a historical measurement with conditions or omit. Do not invent business impact.
- [ ] Keep the work-in-progress version honest before pilot results. After F02, update counts/failures/retention and links without rewriting the methodology to favor the outcome. Use Nick's approved first-person wording; drafts do not establish personal experiences he has not confirmed.
- [ ] Run the repo's `pnpm exec astro check`, `pnpm exec astro build`, `pnpm test:run` and relevant visual tests using its pinned Node/pnpm. Verify the article and featured card locally. Prepare the PR/deployment changes; publication requires authority.

**Accept:** Existing work is reused; featured visibility is proven locally; factual claims are attributable; full workflow can be understood by a hiring manager; production visibility remains explicitly pending until deployed and checked.

## W04

**Outcome:** Visitors can move between product evidence and Nick's leadership story, and the journey is discoverable and accessible. **Own:** Assura About/footer/metadata; Nick article/homepage featured links; existing analytics config only with consent. No wholesale redesign.

- [ ] Use W03's actual canonical article URL in Assura About and a contextual creator link. Nick's article links to Assura quickstart, an actual demo and GitHub. Keep product navigation focused on using the tool.
- [ ] Audit canonical redirect, sitemap entries, robots policy, titles/descriptions, social previews and actual HTTP status for both affected routes. Ensure the unpublished article is not linked as live before its deployment; retain the existing personal-homepage link meanwhile.
- [ ] Inspect repeated decorative text in Nick's accessibility tree. Mark only duplicate decorative layers `aria-hidden="true"`, retain one meaningful heading/link label, and test keyboard navigation, reduced motion and 375px/desktop overflow. Do not hide semantic content just to silence an audit.
- [ ] Define aggregate events `quickstart_open`, `demo_open`, `case_study_open`, `contact_open` and campaign tags. Reuse existing privacy-conscious analytics if present; otherwise prepare an opt-in implementation proposal. No identity stitching between domains, hidden session recording or fabricated conversion events.
- [ ] Validate local route/fragment links in Playwright and inspect deployed routes after authorized deployment. Use Search Console evidence, if access is available, to diagnose indexing; search-engine result absence is not an indexing proof.

**Accept:** Bidirectional links reach live content; canonicals are noncompeting; a screen reader sees one meaningful heading; measurements are distinguishable from product adoption. Access-dependent SEO/analytics steps have explicit pending status.

## F01

**Outcome:** A bounded demand test begins before weeks of feature work. **Create:** `docs/analysis/assura-pilot-kit.md` with invitation draft, script and evidence template. **Human owner:** Nick for participant selection and invitations.

- [ ] Define target: 3–5 maintainers already using coding agents, with repeated repository-convention failures. Include different stacks and at least two people outside Nick's own projects. Do not select only enthusiastic friends after hearing answers.
- [ ] Prepare this invitation: “I’m testing whether repository-level checks can reduce cleanup when working with coding agents. Would you spend 20 minutes showing me a recent convention failure and how you handled it? I’m looking for counterexamples as well as interest; no installation required.” Nick edits/sends it with authorization.
- [ ] Interview script: show the last concrete failure; where the convention lived; when caught; repair/review effort; existing tools; what those tools could already solve; what would make another tool unwelcome. Avoid pitching features before understanding the example.
- [ ] Record anonymized evidence: participant role/stack, incident, workaround, frequency, observed cost if known, unmet need, objections, willingness to pilot. Do not invent time savings from estimates.
- [ ] After five conversations, make a documented go/narrow/stop decision: proceed if at least three identify a repeated unmet need and want to test; narrow if the same smaller need dominates; pause expansion if existing tools already satisfy it. Do not treat this small sample as market-size proof.

**Accept:** Kit ready locally is `verified`; actual demand outcome is done only after real conversations. If invitations are not authorized, report that boundary and continue technical cards; no fabricated respondents.

## F02

**Outcome:** Real maintainers independently install, use and evaluate the supported workflow. **Own:** pilot protocol/results; fixing discovered bugs belongs to the corresponding technical card.

- [ ] Recruit from F01 after R06/A07. Supply the public install/quickstart and a version pin, not personalized corrective instructions. Observe first use before helping; count each intervention separately.
- [ ] Ask each participant to configure a disposable/sanitized project copy, complete one real small change, then try two weeks of use if appropriate. Collect minimal redacted reproductions, not private source repositories.
- [ ] Record successful setup, prompts/repairs, useful catches, false positives, disabled rules, latency, and whether they kept it enabled. Check that reported “caught issue” was an intended convention and would otherwise have reached review.
- [ ] Hold a weekly review: triage by install/config/hook/noise/performance/docs/out-of-scope. Fix blockers promptly; route requests for unrelated intelligence features to P01 instead of expanding automatically.
- [ ] Produce a result table with all participants and dropouts counted. Target at least three maintainers retaining use for two weeks with a concrete useful catch or workflow improvement; disclose failures and selection limitations.

**Accept:** External observed evidence supports or refutes the wedge. Negative results trigger a roadmap decision, not invented adoption claims. Publish quotes/project identities only with permission.

## F03

**Outcome:** A factual launch package is ready for Nick's review. **Create:** `docs/analysis/assura-launch-package.md` and a demo recording/script artifact in the established media location.

- [ ] Build a 60–90 second script: repository intent → minimal init request → selected policy and hooks → intentional violation → bounded feedback → correct repair → authoritative check. Record an actual released supported path; label edits/time cuts.
- [ ] Draft LinkedIn post with problem, one design decision, one actual trial result, current limitation and a specific feedback ask. Link the case study. Add suggested Featured entries for case study and demo and an accurate project-description paragraph.
- [ ] Prepare a follow-up post only if it contains new evidence: what feedback changed, result and residual failure. Suggested cadence is 1–2 weeks, not an algorithm claim. Choose a time Nick can respond.
- [ ] Create a venue decision table with rules URL, date checked, relevance, self-promotion/AI policy, allowed format, human owner and decision. Default Reddit choices: r/rust only for a substantive relevant technical discussion consistent with current rules; r/opensource and r/LocalLLaMA excluded unless clarified permission and relevance exist. Do not disguise authorship or use generated text in a venue that bans it.
- [ ] Prepare three factual answers: “Why not LS-Lint?” (use it if naming alone suffices); “Why not pre-commit?” (Assura is a check/feedback source that can run there); “Was AI used?” (truthful scope of assistance plus Nick's accountability and evidence). Avoid asserting any specific authorship percentage not recorded.

**Accept:** Every claim has a link/result/limitation; demo works from the public install; current venue rules are documented; no auto-posting or engagement manipulation. Human authorship/review is substantive, not concealment.

## F04

**Outcome:** Approved posts lead to a managed feedback loop. **Human/external action:** Nick approves destination, final wording and timing before posting; coding agent completes draft and verification first.

- [ ] Recheck version, links and venue rules at action time. Present the exact post, audience and known limitations. Do not interpret “prepare launch” as authorization to publish.
- [ ] After authorized posting, record actual URL/time and campaign identifier. Monitor manually or through an explicitly authorized product automation; do not start an unsolicited recurring monitor.
- [ ] Triage feedback with version, attempted outcome, expected/actual behavior, minimal repro, severity, owner card and status. Respond with reproducible fixes; don't argue with hostility or solicit votes.
- [ ] Close the loop on substantive reports: acknowledge, reproduce, fix/test, release when warranted, reply with the verified result. Avoid repeated cross-posting; publish a follow-up only for new evidence.

**Accept:** Posts actually exist at recorded URLs and substantive reports have owners/dispositions. A drafted post is not publication; impressions/clicks are not retained use.

## F05

**Outcome:** A bounded v1 decision with explicit support obligations. **Own:** support policy, release goal, canonical/public roadmap, accumulated pilot/soak evidence.

- [ ] Define the frozen v1 contract: configuration syntax, rule IDs, output schemas, exit codes, installation/upgrade behavior, supported OS/harness matrix and documented exceptions. Publish an upgrade/deprecation policy before claiming stability.
- [ ] Collect the existing required 30-day, 50-session, 3-repository, 4-host soak if four hosts remain supported. Record real sessions, not synthetic counts. Pair runtime soak with A07 correctness, R06 installs and F02 usefulness.
- [ ] Audit all backlog cards and residual issues. Q03–Q06 may be deferred with a bounded rationale; unresolved wrong results, destructive setup, release failures or silent enforcement gaps block maturity. Missing advertised-host evidence cannot be averaged out.
- [ ] Choose go, narrowed beta or continued experiment. Update the public roadmap with the decision and evidence. Drop unsupported expansion tasks explicitly; keep a small later list with entry criteria rather than an unbounded wish list.
- [ ] Prepare a v1 release through R06 only after acceptance. Include what became stable, what remains experimental and how users report regressions.

**Accept:** Stable label is earned by the declared contract and observed evidence. Fewer features with reliable support is a valid outcome; elapsed calendar time alone is not.
