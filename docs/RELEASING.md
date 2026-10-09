# Releasing

The release workflow builds CLI archives and the WASM package from the tag, then
publishes them with SHA-256 checksum files.

## Release pull request checklist

- [ ] Update `CHANGELOG.md` and the workspace crate versions as needed.
- [ ] Bump `site/SOROBAN_LINT_VERSION` to the exact tag being released. The docs
  site generates its rule catalog from that release's CLI metadata.
- [ ] Confirm the pinned release exists and publishes the Linux CLI archive and
  its `.sha256` checksum before merging the site version bump.
- [ ] Run `cargo test --workspace` and `npm --prefix site run verify`.

After merge, publish the tag to run `.github/workflows/release.yml`. Once the
release is published, `.github/workflows/docs.yml` builds the site against the
new tag and deploys when the workflow runs from `main`.
