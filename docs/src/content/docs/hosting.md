---
title: 'Hosting Your Own Instance'
description: 'All you need to know to get your own Eludris instance running!'
order: 1
---

One of our goals ever since the start is to make hosting Eludris instances as simple
and as friction free as possible. To that extent Eludris has provided an [`Eludris.toml`](./conf.md)
where you can configure your instance as much as you wish and also official docker
support to help you get started scot-free.

## Deploying a Production Instance With Docker

We really recommend using [Docker](https://www.docker.com/) for production instances,
it helps keep all the microservices and the external tools they rely on all in one
neat group of containers without conflicting with anything else related to the host
system, and without you having to go around installing OS-dependent C libraries.

Deploying a production-ready Eludris instance with docker is only 2 simple steps:

1. Install the [Eludris CLI](/docs/cli)
   You can find more info about this in the [CLI docs](/docs/cli).

2. Run `eludris deploy`
   This will take you through the process of setting up your own Eludris instance step-by-step
   without you having to worry about messing up anywhere.

And it is as simple as that.

If all goes right, you can find Oprish at `0.0.0.0:7159`, Pandemonium at `0.0.0.0:7160` and Effis
at `0.0.0.0:7161`.

## Deploying on Bare Metal / Running a Development Instance

Now this is just a little bit more complicated, here's a quick rundown of what you
have to do:

1. Get the required dependencies
   You will need [KeyDB](https://docs.keydb.dev/docs/download) and [PostgreSql](https://postgresql.org/downloads/),
   the latest version of both should work.

2. Get the latest version of `eludris/eludris`.

```sh
git clone https://github.com/eludris/eludris && cd eludris
```

You may also want update the submodules to use the `next` branch if you're trying
to run a development instance or test a new feature. If so run this instead:

```sh
git clone https://github.com/eludris/eludris --branch=next && cd eludris
```

3. Configure the `Eludris.toml` file & `.env`.

`.env` is mostly optional thanks to somewhat sane defaults, however you'll need to
have an `eludris` database for PostgreSql and a root user with the password as root,
you can also just change the database URI using the `DATABASE_URL` environment variable
and your own `.env`.

Additionally, I'd recommend throwing a `RUST_LOG="debug"` in there for debugging
& logging purposes.

For more info check out the [configuration page](/docs/conf) of the docs.

4. Start the dependencies.

5. Start the microservices.

Now what I'd recommend doing here first is building all the microservices at once,
to do that, just run this command:

```sh
cargo build --release
```

After which you can just run these commands to start the microservices.

```sh
cargo run --bin oprish --release
cargo run --bin pandemonium --release
cargo run --bin effis --release
```

If you want faster build time for the price of slightly worse performance when
you're for example testing, you can run the microservices in debug mode like so:

```sh
cargo build
```

```sh
cargo run --bin oprish
cargo run --bin pandemonium
cargo run --bin effis
```

You will need to run each of these commands in its own terminal session since they
are seperate processes.

Depending on your hardware and whether you're building in release mode, this may
take a bit :D

Congratulations, now you're self-hosting a development instance of Eludris on bare
metal!

If you did everything like in this documentation, you should have Oprish
at `127.0.0.1:7159`, Pandemonium at `127.0.0.1:7160` and Effis at `127.0.0.1:7161`.
