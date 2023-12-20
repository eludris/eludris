#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# ruff: noqa: E501
import os
import signal
import subprocess
import pathlib
import logging
import time
import sys
from urllib.request import urlopen

CRATES = ["oprish", "pandemonium", "effis"]

logging.basicConfig(format="%(message)s", level=logging.INFO)
log = logging.getLogger(__name__)


def kill_microservices(pids: dict[str, int]):
    for crate, pid in pids.items():
        log.info(f"\033[3;35mStopping \033[1;35m{crate}...\033[0m")
        os.kill(pid, signal.SIGINT)


if __name__ == "__main__":
    repo_dir = pathlib.Path(os.path.realpath(__file__)).parent
    os.chdir(repo_dir)  # removes a lot of pain
    pids = {}
    instance_url = os.getenv("INSTANCE_URL") or "http://0.0.0.0:7159"

    outbuff = None if "--logs" in sys.argv else subprocess.DEVNULL
    workspace_tests = "--no-workspace" not in sys.argv

    env = os.environ
    if "RUST_LOG" in env and not outbuff:
        env.pop("RUST_LOG")
    else:
        env["RUST_LOG"] = "DEBUG"

    for crate in CRATES:
        log.info(f"\033[3;35mCompiling \033[1;35m{crate}...\033[0m")
        process = subprocess.run(
            ["cargo", "build", "-p", crate],
            env=env,
            stdout=outbuff,
            stderr=outbuff,
        )
        if process.returncode != 0:
            log.error(
                f"\033[1;31mFailed to compile {crate} with error code {process.returncode}\033[0m."
                " Consider running again with `--logs` for more info"
            )
            kill_microservices(pids)
            exit(1)

    env["ELUDRIS_CONF"] = "tests/Eludris.toml"
    if workspace_tests:
        log.info("\033[3;35mTesting workspace...\033[0m")
        process = subprocess.run(
            ["cargo", "test"],
            stdout=outbuff,
            stderr=outbuff,
        )
        if process.returncode != 0:
            log.error(
                f"\033[1;31mWorkspace tests failed with code {process.returncode}\033[0m."
                " Consider running again with `--logs` for more info"
            )
            kill_microservices(pids)
            exit(1)
