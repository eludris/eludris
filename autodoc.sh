#!/usr/bin/env bash

# Goes to the place the script is actually located
cd "$( dirname "${BASH_SOURCE[0]}" )"

if [ -d autodoc ]; then
  rm autodoc/* -r
else
  mkdir autodoc
fi

mkdir autodoc/todel autodoc/oprish autodoc/effis

ELUDRIS_AUTODOC=1 cargo build --all-features
