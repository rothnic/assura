---
type: template
title: Document Template
created: YYYY-MM-DD
tags:
  - template
  - documentation
related:
  - '[[adr-template]]'
  - '[[feature-template]]'
---

# [REQUIRED: Document Title]

[Write a clear, descriptive title that summarizes the document's purpose. Keep it under 60 characters if possible.]

## Overview

[REQUIRED: Provide a brief summary of what this document covers. This should be 2-4 sentences that give readers enough context to understand if this document is relevant to their needs.]

## Context

[Optional but recommended: Explain the background and motivation for this document. What problem does it solve? Who is the intended audience? When should someone read this?]

## Main Content

[REQUIRED: The primary content of your document goes here. Organize into logical sections with clear headings.]

### Section One

[Content for the first major section...]

### Section Two

[Content for the second major section...]

## Related Documents

[Use wiki-links to connect related documents. This creates a knowledge graph that makes navigation easier.]

- [[related-document-1]] - Brief description of how it's related
- [[related-document-2]] - Brief description of how it's related
- [[adr-template]] - For architecture decisions referenced here

## References

[List any external references, links, or sources:]

- [External Resource Name](https://example.com) - Brief description
- [[internal-spec]] - Reference to internal specification

## Notes

[Optional: Any additional notes, caveats, or future considerations.]

---

**Template Usage:**
- Replace all `[REQUIRED: ...]` placeholders with actual content
- Remove optional sections that aren't needed
- Update the `created` date in YAML front matter
- Add relevant tags to the front matter
- Fill in the `related` section with wiki-links to connected documents
