# Rendered Proof

Captured from local preview at `http://127.0.0.1:4327/guides/agent-ready-onboarding/`
on 2026-07-03. Binary screenshots were generated for visual inspection, then
moved out of the checked tree because `.trellis/**` is validated as text.
Local screenshot artifacts are currently in
`/tmp/assura-agent-ready-onboarding-evidence/`.

Reproduction commands:

```bash
pnpm --dir website preview --host 127.0.0.1 --port 4327
playwright screenshot --browser chromium --viewport-size=1440,1200 http://127.0.0.1:4327/guides/agent-ready-onboarding/ /tmp/assura-agent-ready-onboarding-evidence/agent-ready-onboarding-desktop.png
playwright screenshot --browser chromium --viewport-size=390,1200 http://127.0.0.1:4327/guides/agent-ready-onboarding/ /tmp/assura-agent-ready-onboarding-evidence/agent-ready-onboarding-mobile.png
```

| Viewport | Evidence |
| --- | --- |
| Desktop, 1440x1200 | Screenshot captured and visually checked; PNG hash `272e7a2f1dd730d4bfc1e34a79667744586d3b05c832467198c57dc75f4eccdc`. |
| Mobile, 390x1200 | Screenshot captured and visually checked; PNG hash `756e43599c32f3db9854de0b64cd36b8fedcb19bb3448049f0152f660a0268a5`. |

The route returned HTTP 200 and the screenshots show the dedicated guide,
sidebar placement, section table of contents, and mobile command overflow
behavior.

Rendered HTML was also checked for these markers:

- `Agent-Ready Onboarding`
- `href="/guides/agent-ready-onboarding/" aria-current="page"`
- `First-Run Phases`
- `Report Shape`
- `Generated Packet`
- `Agent-Next Questions`
- `Checked Versus Unchecked`
- `Content And Project Packs`
- `Lifecycle Profiles`
- `Specialization Flow`
- `assura agent onboard . --agent auto --format json`
