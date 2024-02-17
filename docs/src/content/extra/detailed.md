---
title: 'Eludris Detailed'
description: 'A detailed rundown and initial spec of how Eludris is to eventually function'
order: 0
---

## Overview

The main goal with Eludris is to provide a uniquely fresh but not entirely new experience. One where anyone can find or create a place focused around their interests while having a couple of fundamental policies that greatly improve their experience.

These policies include being secure and respecting privacy, while also not inconveniencing users who don't understand what these things entail.

In addition making your own clients, libraries, or tooling around Eludris is more than welcome and is in fact encouraged. Eludris facilitates this through the [Eludris Community GitHub organisation](https://github.com/eludris-community) and the [Eludris Awesome repository](https://github.com/eludris/awesome).

## IDs

A Eludris ID is a 64 bit (8 byte) number, structured like so:

```
 12345678  12345678  12345678  12345678  12345678  12345678  12345678  12345678
 TTTTTTTT  TTTTTTTT  TTTTTTTT  TTTTTTTT  TTTTTTTT  TTTTTTTT  WWWWWWWW  SSSSSSSS
╰──────────────────────────────────────────────────────────╯╰────────╯╰────────╯
                             │                                  │         │
                             │                                  │8 bit (1 byte) sequence
                             │                    8 bit (1 byte) worker ID
              48 bit (6 byte) Unix timestamp
```

T: A Unix timestamp with the Eludris epoch (1,650,000,000).

W: The id of the worker that generated this ID.

S: The sequence number of this ID

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

Eludris _does not know_ the user's super private key. Eludris gives the user all the unencrypted-public keys and encrypted-private keys when connecting to Pandemonium.

The private keys are encrypted with the user's super public key.

For example, let's say a user creates an account. They create themselves a pair of keys, one public (A) and one private key (B).
They give Eludris their public key (A) and store the private key (B).

They then join an encrypted DM and the other user generates a pair of keys for the DM, one public key (C) and one private key (D). They send Eludris the DM's private key (D) encrypted with the first user's public key (A), Eludris stores this and gives it to the first user when requested and when they connect to pandemonium.

This ensures that every user can always have their keys without any risks of the server being able to decrypt the payloads.

Eludris **_never_** gets access to the non-encrypted private keys of _any_ key pair at any point in time.

To further increase the security Eludris marks all sessions (besides the first) as untrusted and essentially rats it out to everyone, a user can verify their session from their original session in which they securely pass on the super key pair to the new session.

#### Direct Messages

Upon a friend request getting accepted and two users becoming friends, the user who accepted the friend request sends a payload with a public key and a private key for the DM, both encrypted using the other user's super public key.

After that all messages sent in this DM is encrypted using the DM's public key and are decrypted with the DM's private key which is stored on Eludris twice, once encrypted with the first user's super public key, and another encrypted with the second user's super public key.

A user can also request they get a new key from the other end which will entirely scrap the old pair of keys and generate new ones in case the old ones get compromised.

#### Group DMs

Group DMs work in a similar fashion, the host sends the room's public and private keys to all the starting participants on room creation encrypted with their public keys.

When a new user joins, any other user will send Eludris the keys they need. Whenever they're online, Eludris will send them all of the Group DM's keys.

The room's keys can also be re-generated by the Group DM's host.

#### Private Communities

Private communities work similarly to how Group DMs work with the addition that the posts may also be encrypted but follow the same foundations.
