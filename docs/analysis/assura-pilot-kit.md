---
title: Assura pilot interview kit
status: current
owner: Nick Roth
---

# Assura pilot interview kit

## Purpose and boundary

This is a bounded demand-discovery kit for Assura's repository-policy wedge:
helping maintainers using coding agents express and catch repository
conventions. It is not a sales script, market-size study, or pilot result.

Nick selects participants, edits and sends invitations, conducts or approves
interviews, and decides whether to contact anyone. Creating this document does
not authorize invitations, installations, recording, publication, or claims of
adoption.

## Participant frame

Seek 3–5 maintainers who already use coding agents and have experienced
repeated repository-convention failures. Include different stacks and at least
two participants outside Nick's own projects. Do not select only enthusiastic
friends after hearing their answer.

Use a small, sanitized sample. Do not request private repositories, secrets, or
source excerpts not needed to understand the incident. Record only the minimum
anonymized evidence below.

## Invitation draft

> I’m testing whether repository-level checks can reduce cleanup when working
> with coding agents. Would you spend 20 minutes showing me a recent convention
> failure and how you handled it? I’m looking for counterexamples as well as
> interest; no installation required.

Nick may adapt the wording for a known recipient while preserving its
counterexample-seeking and no-installation commitments.

## 20-minute interview script

1. What was the last concrete convention failure involving a coding agent?
2. Where did that convention live at the time: source layout, a document,
   review feedback, a script, CI, or somewhere else?
3. When did the team catch it, and what repair or review effort followed?
4. How often does this kind of failure recur? Record an observed count or mark
   frequency unknown; do not turn an estimate into a time-savings claim.
5. What tools or checks already address any part of it? What do those tools
   solve well?
6. What remains unmet, if anything? What would make another tool unwelcome?
7. Is there a small, sanitized example appropriate for a later pilot? Do not
   ask for installation or pitch features before understanding the example.

## Anonymized evidence template

Create one record per conversation. Leave a field `unknown` when it was not
observed.

| Field | Record |
| --- | --- |
| Participant | Anonymous ID, role, stack, and whether outside Nick's projects |
| Incident | Summary, convention location, when caught, workaround, frequency, observed cost |
| Existing tools | Tool and what it already solves well |
| Need | Unmet need, objections, willingness to pilot, sanitized follow-up availability |
| Notes | Minimum redacted notes only |

Do not keep an ID-to-person mapping in the evidence record. If a participant
explicitly consents to follow-up, Nick may keep the minimum necessary contact
note separately in a Nick-controlled location; no agent or repository process
may access it, and it must be deleted when follow-up ends. Do not publish
names, quotes, project identities, or private transcripts without explicit
permission.

## Decision record after five conversations

Record all five conversations, including dropouts and counterexamples, before
choosing one outcome:

| Outcome | Evidence threshold | Required next action |
| --- | --- | --- |
| Go | At least 3 identify a repeated unmet need and want to test it. | Prepare a bounded F02 pilot only after its technical dependencies are ready. |
| Narrow | The same smaller need dominates, but the broader wedge does not. | Update support scope and route the evidence to the owning card. |
| Stop / pause expansion | Existing tools satisfy the need, or fewer than 3 identify a repeated unmet need and want to test. | Record negative evidence and pause expansion rather than invent adoption claims. |

This sample does not establish market size, time savings, retention, or a
population success rate. Those require later observed evidence.

## Completion state

The kit is ready for Nick's review. F01 is not done until actual conversations
and the resulting decision record exist. Until invitations are specifically
authorized, retain the state as locally prepared with an external-action block.
