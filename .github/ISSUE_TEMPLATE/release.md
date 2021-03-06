---
name: Release issue template
about: Tracking issue for new releases
title: Tidechain {{ env.VERSION }} Release checklist
---

# Release Checklist

This is the release checklist for Tidechain {{ env.VERSION }}. **All** following
checks should be completed before publishing a new release of the
Tidechain/Lagoon runtime or client. The current release candidate can be
checked out with `git checkout release-{{ env.VERSION }}`

### Runtime Releases

These checks should be performed on the codebase prior to forking to a release-
candidate branch.

- [ ] Verify [`spec_version`](#spec-version) has been incremented since the
      last release for any native runtimes from any existing use on public
      (non-private/test) networks. If the runtime was published (release or pre-release), either
      the `spec_version` or `impl` must be bumped.
- [ ] Verify previously [completed migrations](#old-migrations-removed) are
      removed for any public (non-private/test) networks.
- [ ] Verify pallet and [extrinsic ordering](#extrinsic-ordering) has stayed
      the same. Bump `transaction_version` if not.
- [ ] Verify new extrinsics have been correctly whitelisted/blacklisted for
      [proxy filters](#proxy-filtering).
- [ ] Verify [benchmarks](#benchmarks) have been updated for any modified
      runtime logic.

The following checks can be performed after we have forked off to the release-
candidate branch or started an additional release candidate branch (rc-2, rc-3, etc)

- [ ] Verify [new migrations](#new-migrations) complete successfully, and the
      runtime state is correctly updated for any public (non-private/test)
      networks.
- [ ] Verify [Tidechain Client](#tidechain-client) are up to date with the latest
      runtime changes.
- [ ] Push runtime upgrade to Testnet and verify network stability.

### All Releases

- [ ] Check that the new client versions have [run on the network](#burn-in)
      without issue for 12 hours.
- [ ] Check that a draft release has been created at
      https://github.com/tidelabs/tidechain/releases with relevant [release
      notes](#release-notes)
- [ ] Check that [build artifacts](#build-artifacts) have been added to the
      draft-release

## Notes

### Burn In

Ensure that SEMNET DevOps has run the new release on Tidechain and Lagoon validators for at least 12 hours prior to publishing the release.

### Build Artifacts

Add any necessary assets to the release. They should include:

- Linux binary
- GPG signature of the Linux binary
- SHA256 of binary
- Source code
- Wasm binaries of any runtimes

### Release notes

The release notes should list:

- The priority of the release (i.e., how quickly users should upgrade) - this is
  based on the max priority of any _client_ changes.
- Which native runtimes and their versions are included
- The proposal hashes of the runtimes as built with
  [srtool](https://gitlab.com/tidelabs/srtool)
- Any changes in this release that are still awaiting audit

The release notes may also list:

- Free text at the beginning of the notes mentioning anything important
  regarding this release
- Notable changes (those labelled with B[1-9]-\* labels) separated into sections

### Spec Version

A runtime upgrade must bump the spec number. This may follow a pattern with the
client release (e.g. runtime v12 corresponds to v0.8.12, even if the current
runtime is not v11).

### Old Migrations Removed

Any previous `on_runtime_upgrade` functions from old upgrades must be removed
to prevent them from executing a second time. The `on_runtime_upgrade` function
can be found in `runtime/src/lib.rs`.

### New Migrations

Ensure that any migrations that are required due to storage or logic changes
are included in the `on_runtime_upgrade` function of the appropriate pallets.

### Extrinsic Ordering

Offline signing libraries depend on a consistent ordering of call indices and
functions. Compare the metadata of the current and new runtimes and ensure that
the `module index, call index` tuples map to the same set of functions. In case
of a breaking change, increase `transaction_version`.

To verify the order has not changed, you may manually start the following [Github Action](https://github.com/tidelabs/tidechain/actions/workflows/extrinsic-ordering-check-from-bin.yml). It takes around a minute to run and will produce the report as artifact you need to manually check.

The things to look for in the output are lines like:

- `[Identity] idx 28 -> 25 (calls 15)` - indicates the index for `Identity` has changed
- `[+] Society, Recovery` - indicates the new version includes 2 additional modules/pallets.
- If no indices have changed, every modules line should look something like `[Identity] idx 25 (calls 15)`

Note: Adding new functions to the runtime does not constitute a breaking change
as long as the indexes did not change.

### Proxy Filtering

The runtime contains proxy filters that map proxy types to allowable calls. If
the new runtime contains any new calls, verify that the proxy filters are up to
date to include them.

### Benchmarks

Use the bench-bot on `github` to run the benchmarks.

### Tidechain Client

Ensure that a release of [Tidechain Client]() contains any new types or
interfaces necessary to interact with the new runtime.
