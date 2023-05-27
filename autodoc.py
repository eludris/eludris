#!/usr/bin/env python3
# -*- coding: utf-8 -*-
import os
import pathlib
import shutil
import subprocess
import json
import logging

CRATES = ["todel", "oprish", "effis"]  # The other crates do not have autodoc info
EXTRA_COMMAND_ENV = {"ELUDRIS_AUTODOC": "1"}

# Logging setup
logging.basicConfig(format="%(message)s", level=logging.INFO)
log = logging.getLogger(__name__)

if __name__ == "__main__":
    repo_dir = pathlib.Path(os.path.realpath(__file__)).parent
    os.chdir(repo_dir)  # removes a lot of pain

    autodoc_path = pathlib.Path("autodoc")
    if autodoc_path.exists():
        if not autodoc_path.is_dir():
            raise RuntimeError('Found a non-directory "autodoc" file')
        shutil.rmtree(autodoc_path)
    autodoc_path.mkdir()

    os.environ |= EXTRA_COMMAND_ENV

    items = []

    subprocess.run(["cargo", "clean", "-p", "todel_codegen"], env=os.environ)
    for crate in CRATES:
        crate_path = autodoc_path.joinpath(crate)
        crate_path.mkdir()
        log.info(f"\033[3;35mCompiling \033[1;35m{crate}...\033[0m")
        subprocess.run(
            ["cargo", "build", "-p", crate, "--all-features"], env=os.environ
        )
        for item in crate_path.iterdir():
            items.append(f"{crate}/{item.name}")

    metadata = json.loads(
        subprocess.run(
            ["cargo", "metadata", "--no-deps"], capture_output=True
        ).stdout.strip(b"\n")
    )

    autodoc_path.joinpath("index.json").write_text(
        json.dumps({"version": metadata["packages"][0]["version"], "items": items})
    )

    shutil.copytree(
        autodoc_path, repo_dir.joinpath("docs/public/autodoc"), dirs_exist_ok=True
    )
