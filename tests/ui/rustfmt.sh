#!/bin/bash
set -euo pipefail

rustfmt=${RUSTFMT:-rustfmt}
version="$($rustfmt -V)"
if [[ $version != *nightly* ]]; then
	rustfmt="$rustfmt +nightly"
fi

return=0
find "$(dirname "$0")" -name '*.rs' -type f | while read file; do
	$rustfmt --config-path "$(dirname "$0")/../../rustfmt.toml" "$@" "$file" || return=1
done

exit $return
