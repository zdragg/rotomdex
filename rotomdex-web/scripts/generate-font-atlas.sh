#!/usr/bin/env bash

set -euo pipefail
export LC_ALL=C
export BEAMTERM_NO_FILE_LOGS=1

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly WEB_DIR="$(cd -- "$SCRIPT_DIR/.." && pwd)"
readonly SYMBOLS_DIR="$WEB_DIR/assets/atlas-symbols"
readonly OUTPUT="$WEB_DIR/assets/jetbrains-mono-30.atlas"
readonly ATLAS_TOOL="${BEAMTERM_ATLAS:-beamterm-atlas}"

command -v "$ATLAS_TOOL" >/dev/null || {
    printf "error: required command '%s' is unavailable\n" "$ATLAS_TOOL" >&2
    exit 1
}

shopt -s nullglob
symbol_files=("$SYMBOLS_DIR"/*.txt)
(( ${#symbol_files[@]} > 0 )) || {
    printf "error: place at least one .txt file in '%s'\n" "$SYMBOLS_DIR" >&2
    exit 1
}

combined_symbols="$(mktemp)"
temporary_atlas="$(mktemp "$WEB_DIR/assets/.jetbrains-mono-30.atlas.XXXXXX")"
trap 'rm -f -- "$combined_symbols" "$temporary_atlas"' EXIT

for symbol_file in "${symbol_files[@]}"; do
    command cat -- "$symbol_file" >> "$combined_symbols"
    printf '\n' >> "$combined_symbols"
done

"$ATLAS_TOOL" generate "JetBrains Mono" \
    --font-size 30 \
    --range 0x20..0x7E \
    --symbols-file "$combined_symbols" \
    --check-missing \
    --output "$temporary_atlas"

chmod 0644 "$temporary_atlas"
mv -f -- "$temporary_atlas" "$OUTPUT"

printf 'Merged %d symbol files.\n' "${#symbol_files[@]}"
"$ATLAS_TOOL" inspect "$OUTPUT"
