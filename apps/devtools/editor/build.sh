#!/bin/sh
# Bundles the editor (CodeMirror 6 and the page-facing wrapper in index.js) into
# ../ui/vendor/editor.js. Needs Node.js; the packages are pinned in package.json.
set -e
cd "$(dirname "$0")"
npm install --no-audit --no-fund --loglevel=error
npx esbuild index.js --bundle --format=esm --minify --target=es2020 --legal-comments=none --outfile=../ui/vendor/editor.js
ls -l ../ui/vendor/editor.js
