# Contributing

The `Tiedechain` project is an **OPENISH Open Source Project**

## What?

Individuals making significant and valuable contributions are given commit-access to a project to contribute as they see fit. A project is more like an open wiki than a standard guarded open source project.

## Rules

There are a few basic ground-rules for contributors (including the maintainer(s) of the project):

- **No `--force` pushes** or modifying the main branch history in any way. If you need to rebase, ensure you do it in your own repo. No rewriting of the history after the code has been shared (e.g. through a Pull-Request).
- **Non-main branches**, prefixed with a short name moniker (e.g. `evan-my-feature`) must be used for ongoing work.
- **All modifications** must be made in a **pull-request** to solicit feedback from other contributors.
- A pull-request _must not be merged until CI_ has finished successfully.
- Contributors should adhere to the house coding style.

## Merge Process

_In General_

A Pull Request (PR) needs to be reviewed and approved by project maintainers unless:

- it does not alter any logic (e.g. comments, dependencies, docs), then it may be tagged https://github.com/tide-labs/tidechain/pulls?utf8=%E2%9C%93&q=is%3Apr+is%3Aopen+label%3AA2-insubstantial[`insubstantial`] and merged by its author once CI is complete.
- it is an urgent fix with no large change to logic, then it may be merged after a non-author contributor has approved the review once CI is complete.

_Labels TLDR:_

- `A-*` Pull request status. ONE REQUIRED.
- `B-*` Changelog and/or Runtime-upgrade post composition markers. ONE REQUIRED. (used by automation)
- `C-*` Release notes release-priority markers. EXACTLY ONE REQUIRED. (used by automation)
- `D-*` More general tags on the PR denoting various implications and requirements.

_Process:_

- Please tag each PR with exactly one `A`, `B`, `C` and `D` label at the minimum.
- Once a PR is ready for review please add the https://github.com/tide-labs/tidechain/pulls?q=is%3Apr+is%3Aopen+label%3AA0-pleasereview[`A0-pleasereview`] label. Generally PRs should sit with this label for 48 hours in order to garner feedback. It may be merged before if all relevant parties had a look at it.
- If the first review is not an approval, swap `A0-pleasereview` to any label `[A3, A7]` to indicate that the PR has received some feedback, but needs further work. For example. https://github.com/tide-labs/tidechain/labels/A3-inprogress[`A3-inprogress`] is a general indicator that the PR is work in progress and https://github.com/tide-labs/tidechain/labels/A4-gotissues[`A4-gotissues`] means that it has significant problems that need fixing. Once the work is done, change the label back to `A0-pleasereview`. You might end up swapping a few times back and forth to climb up the A label group. Once a PR is https://github.com/tide-labs/tidechain/labels/A8-mergeoncegreen[`A8-mergeoncegreen`], it is ready to merge.
- PRs must be tagged with their release notes requirements via the `B1-B9` labels.
- PRs must be tagged with their release importance via the `C1-C9` labels.
- PRs must be tagged with their audit requirements via the `D1-D9` labels.
- PRs that must be backported to a stable branch must be tagged with https://github.com/tide-labs/tidechain/labels/E1-runtimemigration[`E0-patchthis`].
- PRs that introduce runtime migrations must be tagged with https://github.com/tide-labs/tidechain/labels/E1-runtimemigration[`E1-runtimemigration`]. See the https://github.com/tide-labs/tidechain/blob/master/utils/frame/try-runtime/cli/src/lib.rs#L18[Migration Best Practices here] for more info about how to test runtime migrations.
- PRs that introduce irreversible database migrations must be tagged with https://github.com/tide-labs/tidechain/labels/E2-databasemigration[`E2-databasemigration`].
- PRs that add host functions must be tagged with with https://github.com/tide-labs/tidechain/labels/E4-newhostfunctions[`E4-newhostfunctions`].
- PRs that break the external API must be tagged with https://github.com/tide-labs/tidechain/labels/E5-breaksapi[`E5-breaksapi`].
- PRs that materially change the FRAME/runtime semantics must be tagged with https://github.com/tide-labs/tidechain/labels/E6-transactionversion[`E6-transactionversion`].
- PRs that "break everything" must be tagged with https://github.com/tide-labs/tidechain/labels/E7-breakseverything[`E7-breakseverything`].
- PRs that block a new release must be tagged with https://github.com/tide-labs/tidechain/labels/E10-blocker%20%E2%9B%94%EF%B8%8F[`E10-blocker`].
- No PR should be merged until all reviews' comments are addressed and CI is successful.

_Reviewing pull requests_:

When reviewing a pull request, the end-goal is to suggest useful changes to the author. Reviews should finish with approval unless there are issues that would result in:

- Buggy behavior.
- Undue maintenance burden.
- Breaking with house coding style.
- Pessimization (i.e. reduction of speed as measured in the projects benchmarks).
- Feature reduction (i.e. it removes some aspect of functionality that a significant minority of users rely on).
- Uselessness (i.e. it does not strictly add a feature or fix a known issue).

_Reviews may not be used as an effective veto for a PR because_:

- There exists a somewhat cleaner/better/faster way of accomplishing the same feature/fix.
- It does not fit well with some other contributors' longer-term vision for the project.

## Issues

Please label issues with the following labels:

. `I-*` Issue severity and type. EXACTLY ONE REQUIRED.
. `P-*` Issue priority. AT MOST ONE ALLOWED.
. `Q-*` Issue difficulty. AT MOST ONE ALLOWED.
. `Z-*` More general tags on the issue, denoting context and resolution.

## Releases

Declaring formal releases remains the prerogative of the project maintainer(s).

## Changes to this arrangement

This is an experiment and feedback is welcome! This document may also be subject to pull-requests or changes by contributors where you believe you have something valuable to add or change.

## Heritage

These contributing guidelines are modified from the "OPEN Open Source Project" guidelines for the Level project: https://github.com/Level/community/blob/master/CONTRIBUTING.md
