---
title: 'Eludris Detailed'
description: 'A detailed rundown and initial spec of how Eludris is to eventually function'
order: 0
---

## Overview

The main goal with Eludris to provide a uniquely fresh but not entirely new experience. One where anyone can find (or create) a place focused around their interests while having a couple of fundamental policies that greatly improve users' experience.

These policies include being secure, 100% FOSS, privacy respecting and decentralised, while also not inconveniencing users who don't understand what these things entail, but still enhancing their experience.

In addition -- and as with anything Eludris-related -- modifying the source code or making your own clients, libraries, or tooling around Eludris is more than welcome and is in fact encouraged. We have an entire [GitHub organisation](https://github.com/eludris-community) focused on this.

## Takeaways

### It's All About Communities

Communities can be either message-based, post-based, or both.

Message-based communities work like how a Discord server does, having many different channels secluded to their own types. Members can -- as you can guess -- send messages within these channels.

Post-based communities work like how a Subreddit does with members being able to create different types of posts, vote on them and leave comments.

Both community types have shared features however, like roles, nicknames and so on.

Communities can be either public or private.

Communities can _usually_ (depends on the instance) get manually reviewed by an instance admin to get verified if requested by the community's owner.

Verified communities can claim a namespace, getting their own URL invite and are indexed onto list to be easily discoverable. However, doing so adds more restrictions upon them, such as no End-To-End-Encryption, stricter moderation, and so on.

### Accounts Are Unique

Accounts for a **single** instance are unique, but when [federated](./federation#overview), this is broken.

You can follow people or send them friend requests.

There's also a Reddit karma-like point system creatively called Social Credit by the Eludris Team.
You can gain Social Credit by getting more up-votes on your posts, spending time interacting with people, getting rewarded by instance moderators or through events.

### Bots Done Better

Any user can make a bot user which is associated with their account.

Bot users are more or less treated like normal users with a few differences.

1. Bots can register Application commands and message components.
2. Bots cannot have friends. (I tried to convince the other Eludritians but they refused. - Enoki :( )
3. Bots cannot join communities themselves, however, they can be invited into them unlike normal accounts.
4. Bots will not have multiple session tokens and instead will have one token that they can regenerate.

Verification for bots only means that the bot and its owner will be given a little neat badge of honour.

Discord-styled application commands will be available. However -- unlike Discord -- they will not be forced upon people and will have more cool features, uses and will be more flexible. Additionally buttons and more message components will be available to give bot developers more room and tools to make cool stuff.

## Miscellaneous Info

### IDs

A Eludris ID is a 128 bit (16 byte) number, structured like so:

```
 12345678  12345678  12345678  12345678  12345678  12345678  12345678  12345678  12345678  12345678  12345678  12345678  12345678  12345678  12345678  12345678
 TTTTTTTT  TTTTTTTT  TTTTTTTT  TTTTTTTT  TTTTTTTT  TTTTTTTT  TTTTTTTT  TTTTTTTT  IIIIIIII  IIIIIIII  IIIIIIII  IIIIIIII  IIIIIIII  IIIIIIII  SSSSSSSS  SSSSSSSS
╰──────────────────────────────────────────────────────────────────────────────╯╰──────────────────────────────────────────────────────────╯╰──────────────────╯
                                       │                                                                     │                                       │
                                       │                                                                     │                          16 bit (2 byte) sequence.
                                       │                                                       48 bits (6 byte) Instance ID.
                        64 bit (8 byte) Unix timestamp.
```

T: A Unix timestamp with the Eludris epoch (1,650,000,000)─.

I: The id of the instance that generated this ID.

S: The sequence number of this ID

An instance ID is a 48 bit (6 byte) number, structured like so:

```
 12345678  12345678  12345678  12345678  12345678  12345678  12345678 12345678
 TTTTTTTT  TTTTTTTT  TTTTTTTT  TTTTTTTT  TTTTTTTT  TTTTTTTT  TTTTTTTT TTTTTTTT
╰─────────────────────────────────────────────────────────────────────────────╯
                                       │
                                       │
                        48 bit (6 byte) Unix timestamp.
```

T: 48 bits of the current Unix timestamp (also with the Eludris) epoch.

### KeyDB

Eludris uses a non persistent KeyDB instance to store data that should be fetched with low latency, and is ephemeral such as rate-limit info.

Here's the structure of currently used keys:

- `rate_limit:<user-id>:<method>:<route>`

## How It Works

Eludris is split into four main parts, most of which are microservices. These services are:

- Oprish: The Eludris RESTful API.
- Pandemonium: The Eludris websocket-based gateway.
- Effis: The Eludris file server, proxy and CDN.
- Todel: The Eludris model and shared logic crate.

All of the microservices' source code can be found in the [Eludris meta-repository](https://github.com/eludris/eludris).

## The Token

Eludris uses JWT tokens to authenticate users.
These tokens are required for nearly every interaction.
Trying to connect to the Gateway or interact with the API? You'll need a token!
The JWT tokens used within our services use a cryptographically secure pseudo-random string with a randomly generated HC128 string as a secret.

If you wish to get a new token, send an HTTP request to `/auth` with your email and password.

Tokens work on a per-session basis. What this means is that you'll have to generate a new token for every client you use.
This is done to make it easy to invalidate any session without impacting the others.

Changing your password automatically invalidates all your tokens.

## End-To-End-Encryption

End-To-End-Encryption (or E2EE for short) will be available to private communities, private GDMs (group direct messages) and direct messages (DMs) between friends.

### E2EE Implementation

First off, every user is provided a personal and unique pair of a public key and a private key.

Payloads with encrypted data (message, post, etc.) have an extra field in their payload, the `pubkey` field, which contains the public key the user used to encrypt the payload's content. This is done so that the corresponding private key could be fetched from the user's public-private key pairs and requested if the current one is invalid.

As for storing public-private key pairs, storing them locally (on the client's machine) causes a lot of extra complexity, especially with sharing and syncing keys.

For example, issues with a client being offline when it's given a key, multiple clients, and so on.

To combat that, Eludris' E2EE is designed so that each user has a super private-public key pair that their other private keys are encrypted with.

The instance _does not know_ the user's super private key. The instance gives the user all the unencrypted-public keys and encrypted-private keys when connecting to Pandemonium.

The private keys are encrypted with the user's super public key.

For example, let's say a user creates an account. They create themselves a pair of keys, one public (A) and one private key (B).
They give the instance their public key (A) and store the private key (B).

They then join an encrypted DM and the other user generates a pair of keys for the DM, one public key (C) and one private key (D). They send the instance the DM's private key (D) encrypted with the first user's public key (A), the instance stores this and gives it to the first user when requested and when they connect to pandemonium.

This ensures that every user can always have their keys without any risks of the server being able to decrypt the payloads.

The instance **_never_** gets access to the non-encrypted private keys of _any_ key pair at any point in time.

To further increase the security each instance marks all sessions (besides the first) as untrusted and essentially rats it out to everyone, a user can verify their session from their original session in which they securely pass on the super key pair to the new instance.

#### Direct Messages

This one is quite simple, upon a friend request getting accepted and two users becoming friends, the user who accepted the friend request sends a payload with a public key and a private key for the DM, both encrypted using the other user's super public key.

After that all messages sent in this DM is encrypted using the DM's public key and are decrypted with the DM's private key which is stored on the instance twice, once encrypted with the first user's super public key, and another encrypted with the second user's super public key.

A user can also request they get a new key from the other end which will entirely scrap the old pair of keys and generate new ones in case the old ones get compromised.

#### Group DMs

Group DMs can be encrypted too. They work in a similar fashion, the host sends the room's public and private keys to all the starting participants on room creation encrypted with their public keys.

When a new user joins any other user will send the instance the keys they need whenever they're online.

The room's keys can also be re-generated by the GDM's host.

#### Private Communities

Private communities work similarly to how Group DMs work with the addition that the posts may also be encrypted but follow the same foundations.

## Federation

Eludris will be federated, meaning anyone can host their own instance and instances
can communicate with each other so that any user on one instance can interact with
others on any other instance.

### Federation Implementation

All routes where other instances can request or submit data will have an
additional `/external` path (like `/external/this/channels/:channel_id/`).

For info about how IDs are created read [this](/extra/detailed#ids).

`/external` routes will follow specific rules, these being:

A new instance (one the home instance has never seen before) will have to first
send an `identify` payload.
This payload is simple as it just provides a shared secret token key that both instances
can use to identify each other (in case an instance's domain gets compromised or
changed) and the instance's id.

`/external` routes will take both Oprish payloads and Pandemonium payloads in the
form of HTTP requests.

For example, let's say an instance A has a community with a channel that has users
from other instances, one of which is B.
When a user from instance B sends a message to `B's domain/external/A's ID/channels/:channel_id/messages`,
B will send the Oprish message payload to `A's domain/external/this/channels/:channel_id/message`.

When a user from instance A sends a message the opposite will happen with A sending
a request to B's external route in form of a Pandemonium payload.
