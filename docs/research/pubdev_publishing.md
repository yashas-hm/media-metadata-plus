# Automated pub.dev Publishing vs. Branch/Tag Protection

> Research from debugging the `Release` workflow's pub.dev publish failure. No code from this doc is active yet —
> `release-and-publish`'s publish steps are currently commented out in `.github/workflows/release.yml` and releases
> are published manually (`flutter pub publish` from a local checkout) until one of the designs below is picked.

---

## Bottom Line

pub.dev's GitHub Actions trusted publisher (OIDC) **requires the workflow run itself to have been triggered by a
git tag push** — not a branch push, not a `pull_request` event, not `workflow_dispatch`. Pushing a tag *during* a
run doesn't satisfy this; what matters is how the run's own trigger event set `github.ref`. Compounding this,
**GitHub does not let a push made with the default `GITHUB_TOKEN` retrigger other workflows** (anti-recursion
safeguard), so a tag pushed by a bot job using `GITHUB_TOKEN` won't even fire a `push: tags:` workflow. Combined
with branch protection on `main` (can't push release commits directly) and a tag-protection rule (only specific
actors may create tags), there is no single trigger that is simultaneously: automatic, a real tag push, and
compliant with both protection rules — without adding a dedicated bot identity.

---

## How the Workflow Broke

Commit `0fcfbac` ("fix: release workflow enhancements") replaced the working `on: push: tags:` trigger with
`pull_request: closed` + `workflow_dispatch`, and added a `versioning` job that bumps files, commits, and pushes
the tag itself mid-run. Every release before that commit (`v1.2.0` through `v1.4.2`) shows `"event": "push",
"headBranch": "vX.Y.Z"` in the run history — the pre-regression design relied on a **human** pushing the release
tag directly, which is exactly the kind of event pub.dev's OIDC check wants. The redesign traded that away for PR-
based automation and broke publishing as a side effect.

---

## pub.dev's Actual Constraint

From `dart.dev/go/publishing-from-github`:

> Pub.dev only allows automated publishing from GitHub Actions when the workflow is triggered by pushing a git tag
> to GitHub. Pub.dev rejects publishing from GitHub Actions triggered without a tag.

Confirmed via `dart-lang/pub-dev#7177` (open, unresolved): there is no config toggle on pub.dev's Automated
Publishing admin page to relax this to branch/PR/dispatch triggers. The only configurable fields are the repo,
an optional tag-pattern, and an optional required GitHub Actions environment name.

---

## The GITHUB_TOKEN Self-Trigger Restriction

GitHub deliberately does not let workflow runs cascade off pushes/PRs authored by the repo's default
`GITHUB_TOKEN`, to prevent infinite trigger loops. This means:

- A job using the default checkout token to `git push origin vX.Y.Z` will **not** cause a `push: tags:` workflow
  to fire, even though the tag now exists on the remote.
- This restriction does **not** apply to a Personal Access Token or a **GitHub App installation token** — pushes
  authenticated as either behave like a real external push and retrigger workflows normally.

---

## Design Options Considered

| Option | Satisfies pub.dev tag requirement | Respects tag-protection bypass list as the access gate | Manual steps per release | Notes |
|---|---|---|---|---|
| **Manual tag push** | Yes — real human push | Yes — bypass list *is* the gate | One (`git tag && git push`) | Chosen as fallback; zero new infra |
| **GitHub App bot pushes tag** | Yes — App token isn't subject to the self-trigger restriction | **Weakens it** — fires for anyone who can merge a `[vX.Y.Z]`-titled PR, not just bypass-listed humans, unless paired with an approval gate | Zero | See synthesis below |
| **Drop OIDC, use stored `pub.dev` token** | N/A — sidesteps the tag requirement entirely | No gate at all — publishes on any qualifying trigger | Zero | Simplest, but long-lived credential in secrets; weaker publish-time access control |

### Why "GitHub App bot" alone doesn't satisfy the access-control goal

The stated goal is *"only I or a maintainer I explicitly grant access to can publish packages."* A bot App added
to the tag-protection bypass list will auto-push the tag for **any** collaborator's merged release PR — the real
gate becomes "who can merge a PR titled `[vX.Y.Z]`" (ordinary repo write access), not a curated publisher list.
Tag-protection bypass lists containing only trusted humans map directly to the stated goal; a bypass-listed bot
that fires unconditionally does not.

### Synthesis: App for plumbing + Environment required reviewers for the actual gate

`release-and-publish` already targets `environment: pub.dev`. Configuring that environment (Settings →
Environments → `pub.dev` → **Required reviewers**) makes the job pause before *any* step runs until a named
person approves in the Actions UI — independent of what triggered the run. Paired with the App:

- The App automates the mechanical tag push (satisfies pub.dev's OIDC ref requirement, no manual git commands).
- The environment approval is the actual "only I/maintainers I grant" control, since it gates the publish
  regardless of who or what triggered the pipeline up to that point.

This combination gets both full automation *and* the desired access control, at the cost of setting up one
GitHub App.

---

## GitHub App Private Keys Do Not Provide Per-Repo Isolation

A common misconception worth recording: GitHub allows generating **multiple private keys for one App**, but all
keys are interchangeable proofs of the same App identity — none are tied to a specific installation. Any valid
key can sign a JWT, list *all* of the App's installations, and mint an access token for *any* of them. Multiple
keys exist for **rotation** (swap without a hard cutover), not for containing a leak to one repo. True per-repo
isolation requires **separate Apps**, each with its own key and its own (single-repo) installation — meaningful
overhead multiplied by the number of repos.

---

## Recommendation for This Maintainer (Multiple Packages, Solo)

**One shared "release bot" App**, installed only on repos that need it (not a blanket install), permissions
limited to `Contents: Read & write` + `Pull requests: Read & write` (no admin, no secrets access), combined with
environment required-reviewers on every repo's publish job. Reasoning: the environment approval — not the App
key — is the actual choke point against a malicious auto-publish, so a leaked key's worst case is unauthorized
tag/PR creation across the App's installed repos, not an unreviewed publish. Per-repo App isolation is worth
revisiting if a package's dependency tree becomes meaningfully less trusted than the others, or a co-maintainer
with lower trust is added.

**Secret scoping note:** a personal (non-organization) GitHub account has no account-wide Actions secret store —
`APP_ID`/`APP_PRIVATE_KEY` must be duplicated into each repo's own secrets individually. True shared secrets
(one value, scoped to selected repos) require moving these repos under a GitHub Organization and using
organization-level secrets.

---

## Open Items

- [ ] Decide: GitHub App + environment approval, vs. staying with manual tag pushes indefinitely (current state).
- [ ] If App route is chosen: create the App, install on this repo, add `APP_ID`/`APP_PRIVATE_KEY` secrets, add
      the App to the tag-protection bypass list, wire `actions/create-github-app-token@v2` into `versioning`.
- [ ] Either way: configure `pub.dev` environment required reviewers (Settings → Environments → `pub.dev`).
- [ ] Consider a `CODEOWNERS` entry for `.github/workflows/*` and `rust/.cargo/config.toml` so release-pipeline
      changes require the maintainer's review, independent of the above.
