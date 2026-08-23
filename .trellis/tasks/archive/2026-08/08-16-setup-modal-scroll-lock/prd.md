# Setup Modal Scroll Lock

## Goal

Keep the landing page visually fixed while the agent setup dialog is open,
including mobile browsers, then restore the exact prior scroll position when
the dialog closes.

## Acceptance Criteria

- Opening the setup dialog locks background scrolling without changing modal scrolling.
- Escape, the close button, and backdrop dismissal all release the lock.
- Closing restores the page to its pre-open scroll position.
- A browser regression test covers the lock and restoration at a mobile viewport.
