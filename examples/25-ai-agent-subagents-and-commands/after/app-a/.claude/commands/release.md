---
description: Cut a release of app-a — bump version, tag, push.
allowed-tools: Bash, Read, Edit
---

# /release

Cut a release of app-a.

## Procedure

1. Ensure the working tree is clean (`git status`). If not, stop and tell the user.
2. Read the current version from the project manifest.
3. Ask the user which bump: patch, minor, or major.
4. Update the version in the manifest.
5. Commit with message `chore: release vX.Y.Z`.
6. Tag `vX.Y.Z`.
7. Push the commit and the tag.
8. Report the tag name and the commit SHA.

Never run the release on a branch other than `main` without explicit confirmation.
