# Eludris CLI

A simple command line utility to help you with setting up, managing and deploying
your very own eludris instance in mere minutes.

## Installation

You can easily install the Eludris CLI using cargo:

```sh
cargo install eludris
```

The Eludris CLI is also available on the [AUR](https://aur.archlinux.org/packages/eludris):

```sh
<your preferred aur helper> -S eludris
```

## Usage

You can find out more by reading the [CLI docs](https://eludevs.pages.dev/docs/cli)
or by running the help command:

```sh
eludris --help
```

The default CLI config directory can be returned using the `eludris conf-dir` command.
You can also change the config directory using the `ELUDRIS_CLI_CONF` environment variable.

The CLI automatically reads any `.env` files in the current directory.
