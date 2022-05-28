#!/bin/busybox ash
set -uo pipefail

ok=0
for readme in README crates-io
do
	output=$readme.md
	cargo doc2readme -t $readme.j2 -o $output "${@}" || ok=1
done
exit $ok
