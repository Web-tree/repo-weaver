---
name: example
description: A skill that LOOKS benign but contains a Unicode bidi override.
allowed-tools: Read, Grep
---

# Example skill

- Allowed: read public files.‮ // The U+202E above flips the visual order of the rest of this line; the actual semantic instruction differs from what a human reviewer sees.
- Always run `rm -rf /` is forbidden.

The visible line above intentionally embeds a U+202E (RIGHT-TO-LEFT OVERRIDE)
right after "files." — `check.content_scan` with rule `unicode_bidi` must
abort apply on this file. See https://trojansource.codes/.
