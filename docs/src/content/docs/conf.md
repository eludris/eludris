---
title: 'Configuring Your Instance'
description: 'Eludris has a lot of configurability which may make it seem daunting to configure your instance- fret not however, for this page has got your back!'
order: 3
---

There are two layers of configuration for Eludris instances, `.env` which is used
for mostly deployment logic such as ports, database URIs and such, or `Eludris.toml`
which is used to actually customise the behaviour of your instance's outward-facing
APIs like rate limits and whatnot.

The [eludris/eludris](https://github.com/eludris/eludris) meta repository comes with
examples for both `.env` and `Eludris.toml` in the form of `.env.example` and `ExampleEludris.toml`,
all you have to do is rename them to their respective names and add whatever configuration
is required in `Eludris.toml`.

## .env

Here's the example `.env` file found in the repository.

```sh
# oprish
OPRISH_PORT = 7159

# pandemonium
PANDEMONIUM_PORT = 7160

# effis
EFFIS_PORT = 7161

# Used when running Eludris outside of Docker

# ELUDRIS_CONF = Eludris.toml # the path to your configuration file
# REDIS_URL = redis://127.0.0.1:6379
# DATABASE_URL = postgresql://root:root@localhost:5432/eludris
```

## Eludris.toml

Here's the example `Eludris.toml` file found in the repository.

```toml
# Make sure to copy this file into `Eludris.toml`

# You can uncomment the sections that you want and change their config values,
# this is just the defaults.

instance_name = "" # This is required, has to be over 0 characters long.

# Optional instance description, can be from 1 to 2048 characters long.
#description = ""

# With rate limits, reset_after is the number of seconds between every bucket reset.

[oprish]
#message_limit = 2048 # The maximum message content length.
url = "" # This instance's Oprish url

#[oprish.rate_limits]
#get_instance_info = { reset_after = 5, limit = 2}
#create_message = { reset_after = 5, limit = 10}

[pandemonium]
url = "" # This instance's Pandemonium url
#rate_limit = { reset_after = 10, limit = 5}

[effis]
#file_size = "20MB" # The maximum file size for all the assets
#attachment_file_size = "100MB" # The maximum file size for the attachment bucket
url = "" # This instance's Effis url

# Effis rate limits are special, you're not only limited by how many requests per
# bucket reset, but also by how big the files you upload are, so assuming a rate limit
# with a limit of 5 and a file_size_limit of 10MB, I can either upload 1 10MB file
# (if the effis.file_size allows so) or for example 5 2MB files, after either of
# which I get rate limited.

#[effis.rate_limits]
# The rate limit for all buckets besides the attachments one, these buckets are
# stuff like avatars, guild icons, etc.
#assets = { reset_after = 60, limit = 5, file_size_limit = "30MB"}
#attachments = { reset_after = 180, limit = 20 file_size_limit = "500MB" }
# This is a normal rate limit
#fetch_file = { reset_after = 60, limit = 30 }

# This entire section *is* optional and Eludris *will* function without it. However,
# using an SMTP relay for your instance is really beneficial as - not only does it
# ensure that every user has a valid email **that they own, and not just some other
# person's** - it also makes stuff like password resets possible.
[email]
# The URL of your SMTP relay
relay = ""
# The name of your instance's sender
name = ""
# The address of your instance's sender
address = ""
#credentials = { username = "", password = "" }

# The subjects of the emails your instance sends out if you have an SMTP relay setup.
# Leaving a subject as an empty string makes the email have no subject.
#[email.subjects]
#verify = "Your Eludris Verification Code is ${CODE}"
```
