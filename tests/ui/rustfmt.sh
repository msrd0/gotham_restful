#!/bin/busybox ash
set -euo pipefail

rustfmt=${RUSTFMT:-rustfmt}
version="$($rustfmt -V)"
case "$version" in
	*nightly*)
		# all good, no additional flags required
		;;
	*)
		# assume we're using some sort of rustup setup
		rustfmt="$rustfmt +nightly"
		;;
esac

return=0
find "$(dirname "$0")" -name '*.rs' -type f | while read file; do
	$rustfmt --config-path "$(dirname "$0")/../../rustfmt.toml" "$@" "$file" || return=1
done

exit $return
