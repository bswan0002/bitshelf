# Signing and provenance policy

For the first 0.1.x release, Apple Developer ID signing, notarization and independent
build attestations are explicitly deferred. This is a release decision, not an
accidental omission. The ARM Mach-O linker emits an ad-hoc signature; that is not
Developer ID identity or notarization. Local inspection on macOS 15.7.4 showed
`Signature=adhoc` and no TeamIdentifier. Gatekeeper's `spctl --assess --type execute`
rejected the local candidate as having no usable trusted signature. Running the
local extracted executable is a separate test and does not prove a browser-
quarantined download will be accepted. No quarantined browser download has been tested.

Downloaded macOS binaries may be blocked. Do not advertise trusted/frictionless
browser installation or recommend disabling Gatekeeper, removing quarantine
recursively, or indiscriminately overriding security checks. Source builds from
a reviewed tag are the fallback. Release notes and installation docs carry this
limitation. Homebrew installation tests cover the archives and published tap;
they do not stand in for a quarantined-browser-download test.

Each archive contains BUILD-INFO.json (source commit, toolchain, runner/SDK and
lock hashes); SHA256SUMS covers immutable archives. Checksums detect corruption
and mismatched assets but are not independent proof of origin. Trust currently
rests on the reviewed source/tag and the repository's authenticated GitHub release
workflow. No signing or attestation claim is made.

All CI actions are pinned to reviewed upstream commit SHAs. Build/verify jobs have
read-only repository access; only the manually approved release job gets
contents-write. Configure required reviewers on the release environment. A tap
token must be restricted to contents-write on that one tap repository; it is never
needed to build/test source or packages. Do not expose publication credentials to
pull-request jobs. Keep the same least-privilege split when adding attestation.

Follow-up before broad macOS distribution: provision a Developer ID identity and
notarization credentials, sign/notarize/staple where applicable, test actual
quarantined downloads on the oldest supported macOS, and add verifiable build
attestations with documented consumer verification. These are visible deferred
improvements; changing the policy requires new evidence and release notes.
