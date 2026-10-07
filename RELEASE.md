# Releasing archgram

A release is a tag. Pushing `vX.Y.Z` runs the Release workflow
(`.github/workflows/release.yml`): archgram is built for every platform on a
runner of that platform, each binary draws every example and the drawings
must match byte for byte, the npm packages are assembled and tested, and
then staged on npm with provenance through trusted publishing. A staged
package goes public when you approve it on npm with two-factor
authentication.

## Every release

`main` takes changes only through a pull request whose CI passed, so the
version goes in by a pull request and the tag follows the merge.

1. On a branch `release/X.Y.Z`: set the version in `Cargo.toml`
   (`[workspace.package]`; the npm packages take theirs from it), and let
   `Cargo.lock` follow with `cargo check` (CI builds with `--locked`).
   In `CHANGELOG.md`, turn `Unreleased` into the version and today's date,
   point its compare link at the new tag, and make any version an entry
   names, such as the `archgram@X.Y.Z` the skill runs, the new one: the
   section becomes the GitHub release's notes, which cannot change once
   published. In `README.md`, point the skill's install command at the new
   version, and copy `README.md` over `packages/archgram/README.md`, its
   copy for npm (CI fails while the two differ). In `skills/archgram/`,
   set every `archgram@X.Y.Z` the skill runs to it, and the `version` in
   `.claude-plugin/plugin.json` (the tests in
   `crates/archgram-cli/tests/skill.rs` fail until both do). Commit
   (`chore: set the version to X.Y.Z`).

   From the merge until the packages are approved on npm (step 5), a skill
   installed from `main` names a version npm does not have yet, and says
   so and stops; keep that time short.
2. Push the branch, open a pull request titled as its commit, and merge it
   with **Squash and merge** once CI passes.
3. Tag the merged commit on `main` and push the tag:

   ```sh
   git switch main
   git pull --ff-only
   git tag -a vX.Y.Z -m "archgram X.Y.Z"
   git push origin vX.Y.Z
   ```

4. Watch the Release workflow. Its publish job waits for your approval (the
   `npm` environment); approve it once every other job has passed: the
   builds, **npm packages** and **GitHub release (draft)**. The draft runs
   beside the publish job and fails when the changelog has no section for
   the version, and a version once staged on npm can never be staged
   again. If a job fails, see [When a release fails](#when-a-release-fails).
5. On npmjs.com, in the **Staged Packages** tab, approve the seven staged
   packages: the six `@archgram/cli-*` first, `archgram` last, so the
   launcher never points at a version that is not public yet. Each approval
   asks for two-factor authentication. Approve only version X.Y.Z: the tab
   can still hold packages of an earlier release that failed, and
   approving one makes that version public and its `latest` (four
   `@archgram/cli-*` at 0.6.0, approved after 0.6.1).
6. The workflow drafts the GitHub release: an archive of the skill, its
   checksum, a signed record of the build that made it, and the version's
   section of the changelog as its notes. Read it, then publish it;
   published, an immutable release keeps its tag and files as they are.
7. Check that the release is whole: each of the seven packages public at
   the new version and tagged `latest` on npm (each prints X.Y.Z twice),
   and the skill's archive verified.

   ```sh
   for package in archgram @archgram/cli-darwin-arm64 @archgram/cli-darwin-x64 \
     @archgram/cli-linux-arm64 @archgram/cli-linux-x64 \
     @archgram/cli-win32-arm64 @archgram/cli-win32-x64; do
     echo "$package $(npm view "$package@X.Y.Z" version) $(npm view "$package" dist-tags.latest)"
   done
   gh attestation verify archgram-skill-X.Y.Z.tar.gz -R byAbas/archgram
   ```

8. Run the skill once, as a user would, before the directory has it.
   From the repository's root, copy the eval project
   `evals/archgram/files/notes` to a folder outside the repository, make
   it a git repository of its own, and install the skill from the
   release's archive into it, for Claude Code:

   ```sh
   cp -R evals/archgram/files/notes /tmp/archgram-X.Y.Z && cd /tmp/archgram-X.Y.Z
   git init -q && git add -A && git commit -qm "the project as given"
   mkdir -p .claude/skills && curl -sL https://github.com/byAbas/archgram/releases/download/vX.Y.Z/archgram-skill-X.Y.Z.tar.gz | tar -xz -C .claude/skills
   ```

   The released skill must be the only archgram that Claude Code loads
   there. A personal skill of the same name runs instead of the
   project's
   ([skills](https://code.claude.com/docs/en/skills#resolve-skills-that-share-a-name)),
   such as the one `npx skills add -g` links into
   `~/.claude/skills/archgram`, and the plugin from the directory,
   `archgram@synced`, loads beside it. Look for both:

   ```sh
   ls -d ~/.claude/skills/archgram ~/.agents/skills/archgram 2>/dev/null
   claude plugin list | grep -i archgram
   ```

   The list always shows the released copy itself, as
   `archgram@skills-dir` at `./.claude/skills/archgram`: the skill's
   folder carries a plugin manifest, so Claude Code loads it as a plugin
   from the skills folder
   ([plugin origins](https://code.claude.com/docs/en/plugins/loading)).
   Any other archgram line, or any path the `ls` prints, is another copy.
   Remove a personal copy (`npx skills remove --global archgram -y`) or
   move it out of the skills folder until the run is done, and turn the
   plugin off for the run with `claude plugin disable archgram@synced`,
   on again after with `claude plugin enable archgram@synced`
   ([plugin commands](https://code.claude.com/docs/en/plugins/cli-reference#plugin-disable)).

   Start Claude Code there and ask, as the eval
   `repairs-what-lost-its-code` does: "CI says the architecture diagram
   no longer matches the code. Fix it." The run passes when the skill
   runs `archgram@X.Y.Z` from npm, starts from what `archgram check` says
   has lost its code, ends with `archgram check` passing, draws
   `docs/diagrams/architecture.svg`, and lists each part with its file
   and each edge with its line. If it does not, the directory keeps the
   previous version: fix the cause through a pull request and release
   the next patch version.
9. Publish the plugin's new version in Anthropic's directory, once step 7
   shows the release whole and step 8 has run the skill: the skill names
   `archgram@X.Y.Z`, and a version published before npm has it stops for
   everyone who installs it from there. Open archgram under **Submissions**
   at [claude.ai/directory/manage](https://claude.ai/directory/manage),
   select **Check for new commits**, and once the version of the merged
   commit passes its checks, select **Publish**. Auto-publish is off for
   this reason; leave it off. A version held for a reviewer goes live once
   they clear it
   ([Submit your plugin](https://claude.com/docs/plugins/submit#update-a-published-plugin)).

## When a release fails

What to do depends on whether anything has reached users.

- **Nothing is public yet** (a job failed, or packages are staged but not
  approved): reject any staged package on npm, delete the draft GitHub
  release and the tag, and fix the cause through a pull request. If no
  package was staged, tag the new `main` with the same version. If any
  was, release the next patch version: npm keeps a staged version even
  once it is rejected, and will not stage it again ("Cannot stage
  previously published version", 0.6.0).
- **Some packages are public:** an npm version can never be published
  again. If the public packages are sound, approve the rest; if not, fix
  through a pull request and release the next patch version. A version
  left with only some of its packages public is never whole: deprecate
  each of them with the reason
  (`npm deprecate @archgram/cli-<platform>@X.Y.Z "<reason>"`).
- **`latest` points at the wrong version** (step 7): move it back to the
  newest whole release, package by package
  (`npm dist-tag add <package>@X.Y.Z latest`).
- **A public version is broken:** deprecate it on npm with the reason
  (`npm deprecate archgram@X.Y.Z "<reason>"`), and release the next patch
  version with the fix. A published GitHub release stays as it is.

## The first release, once

npm lets a package take a trusted publisher only once the package exists,
so the first version of each package is published by hand, from the files
the workflow built. Nothing is built on your machine.

1. On npm: the `archgram` organization exists, and your account has
   two-factor authentication for authorization and publishing.
2. Push the tag as above. The workflow builds and packs, and skips
   publishing, since `NPM_TRUSTED_PUBLISHING` is not set yet.
3. From the workflow run's summary, download the `npm-packages` artifact
   and unzip it: seven `.tgz` files.
4. Publish them, the platform packages first:

   ```sh
   npm login
   for package in archgram-cli-*.tgz; do npm publish "$package" --access public; done
   npm publish archgram-0.3.0.tgz
   ```

5. On npmjs.com, for each of the seven packages, open **Settings**, then
   **Trusted Publisher**, choose **GitHub Actions** and enter: organization
   or user `byAbas`, repository `archgram`, workflow `release.yml`,
   environment `npm`. Leave **Allow npm publish** unticked: the publisher
   may only stage, as npm recommends (docs.npmjs.com, Trusted publishing).
6. On each package's **Settings**, under **Publishing access**, choose
   **Require two-factor authentication and disallow tokens**.
7. On GitHub, in the repository's **Settings**: under **Environments**,
   create `npm`, with you as its required reviewer and only `v*` tags
   allowed to deploy to it;
   under **Secrets and variables**, then **Actions**, then **Variables**,
   add `NPM_TRUSTED_PUBLISHING` with the value `true`.

From then on every tag stages its packages by itself, with no token stored
anywhere, and each goes public on your approval.
