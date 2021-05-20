#!/bin/busybox ash
set -o pipefail

check=n
if [ "$1" == "--check" ]
then
	check=y
fi

set -u

ok=0
for readme in README crates-io
do
	output=$readme.md
	[ $check == y ] && output=$(mktemp)
	cargo doc2readme -t $readme.j2 -o $output
	if [ $check == y ]
	then
		diff $readme.md $output || ok=1
		rm $output
	fi
done
exit $ok
