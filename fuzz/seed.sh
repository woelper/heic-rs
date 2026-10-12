#!/bin/sh
# The test fixtures as a seed corpus for every target: real files reach far deeper than random
# bytes do, and the fuzzer mutates from there.
set -eu
cd "$(dirname "$0")"
for target in decode probe boxes; do
    mkdir -p "corpus/$target"
    cp ../tests/fixtures/*.heic "corpus/$target/"
    # And every input that has ever found a bug: they reach deep, and must stay fixed.
    cp ../tests/fixtures/fuzz/* "corpus/$target/"
done
