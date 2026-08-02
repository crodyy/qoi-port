# Fetching the reference implementation

This repo does not embed a copy of the original QOI source, to keep the proof-of-unmodified-source
verifiable rather than trusted.

Original repository: https://github.com/phoboslab/qoi
Commit used for this port: 97bacc86a9c4abf5a2d452102dc26546c4c670b9

## To fetch it yourself

```bash
git clone https://github.com/phoboslab/qoi reference/qoi
git -C reference/qoi checkout 97bacc86a9c4abf5a2d452102dc26546c4c670b9
```

## To verify it matches what this port was built against

```bash
sha256sum -c reference/qoi_hash.txt
```

All lines should report OK. If any line fails, the upstream file has changed since this port
was built and verified — do not trust the port's equivalence claims against a different commit
without re-running the oracle generation and full verify.sh / test suite yourself.

## Test image corpus

The official QOI test image corpus (qoi_test_images) is committed directly in this repo under
`oracle-source/qoi_test_images/` since it's small (~5.6MB) — no separate fetch needed. Its
integrity is checked against `oracle-source_hash.txt` the same way.
