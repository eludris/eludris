#!/usr/bin/env bash

# Goes to the place the script is actually located
cd "$( dirname "${BASH_SOURCE[0]}" )"

if [ -d autodoc ]; then
  rm autodoc/* -r
else
  mkdir autodoc
fi

mkdir autodoc/todel autodoc/oprish autodoc/effis

cargo clean -p todel_codegen
ELUDRIS_AUTODOC=1 cargo build --all-features
echo -e "\033[1;32mSuccesfully built autodoc info. Check the \`autodoc\` directory.\033[0m"
