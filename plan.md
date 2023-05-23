# You should not be reading this file

## Instead, check out the [docs](https://eludevs.pages.dev/extra/detailed).

# Eludris

This markdown file is dedicated to explaining and organising how the features and components of Eludris will work.

This file is changed every while with new changes and additions appearing.

## Overview

The goal is to provide a uniquely fresh but not totally new experience focused around programming but also any other topic / interest whilst being secure, free (as in monetary value), free (as in freedom), privacy respecting, open source and decentralised without inconveniencing users who may not have anything to do with the aforementioned topics.

Additionally and as with anything Eludris related, modifying the source code or making your own clients, libraries and tooling around Eludris and such is more than welcome ~~as long as you respect the license anyway~~.

Here are the core takeaways:

- You can create communities.

Communities can be an either message based, post based or both.

Message based communities work like how a Discord server does with there being different channels of different types and members being able to send -you guessed it- messages.

Post based communities work like how a Reddit subreddit does with member being able to create different types of posts, vote on them and leave comments.

**Both** community types have shared features however, like roles, nicknames and so on.

Communities can be either public or private.

Communities can get manually reviewed by a staff member to get verified if requested by the community's owner.

Verified communities can claim a namespace getting their own URL invite and are indexed in a list to be easily discover-able, doing that however adds more restrictions upon them however
(like no End-To-End-Encryption, stricter moderation and so on).

A community can get unverified at any point in time if they break the Eludris EULA, Terms of Service or instance rules, that however can be appealed and, unjustified unverification will not happen.

- Accounts are unique.

Much like Reddit and Twitter only one person can have a specific account name, _however_ that limitation is broken upon multiple instances, refer to the [federation](#federation) section for more information on the matter.

You can follow people or send them friend requests.

You can also gain a Reddit Karma like form of point creatively called Social Credit by the Eludris Team, you gain it by getting more up-votes on your posts, spending time interacting with people, getting rewarded by instance moderators or through events.

- Bots done _right_

Bots will be user accounts like you'd see on Reddit or twitter which would mean that they wouldn't have any bot-specific API limitations, however you have to set your bot as one for it to get verified.

Verification for bots only means that the bot and it's owner will be given a little neat badge of honour.

Discord style application commands will be available, however unlike discord they will not be forced upon people and will have more utilities, uses and will be more flexible, additionally buttons and more message components will be available to give bot developers more room and tools to make cool stuff.

## Tokens

About tokens, to be able to make API requests and connect to the Eludris gateway you will need a token, Eludris uses JWT tokens with a cryptographically secure pseudo-random string with HC128 as a secret, to get a Token you send an HTTP request to `/auth` with your email and password.

Tokens work on a per-session basis which means that you have to generate a Token for every client you use to make it easy to invalidate any sessions without impacting others.

Changing your password automatically invalidates all your tokens.

## End-To-End-Encryption

End to End encryption will be available in private communities, private GDMs (group direct messages) and direct messages (DMs) between friends.

### End-To-End-Encryption Implementation

For starters, all users have a personal public key and private key.

Events with encrypted data (message, post, etc) have an extra field in their payload, the `pubkey` field which contains the public key the message's content was encrypted with so that the corresponding private key would be fetched from the user's public-private key pairs and requested if the current one is invalid.

As for storing public-private key pairs, storing them locally causes a lot of extra complexity especially with sharing and syncing keys and issues with a client being offline when it's given a key, so each user has a super key pair that their keys are encrypted with without the instance knowing the private key, the instance gives the user all the public and private keys
(encrypted by the public key) on connecting to Pandemonium, the instance never has access to the non-encrypted key pairs at any point in time.

To further increase the security each instance marks all session's (besides the first) as untrusted and essentially rats it out to everyone, a user can verify their session from their original session in which they securely pass on the super key pair to
the new instance.

#### DMs

This one is quite simple, upon a friend request getting accepted and two users becoming friends, the user who accepted the friend request sends a payload with a public key and a private key for the DM, both encrypted using the other user's
public key.

After that all messages sent in this DM is encrypted using the DM's public key and are encrypted with their private key which is stored on the instance **encrypted** with the each user's super public-private key pair along with their other keys.

A user can also request they get a new key from the other end which will entirely scrap the old pair of keys and generate new ones in case the old ones get compromised.

#### Group DMs

Group DMs can be encrypted too, they work in a similar fashion, the host sends the room's public and private keys to all the starting participants on room creation
encrypted with their public keys.

Any user can request they get the private and public keys of the room again in which any available user would just encrypt it using the request's user's public key and send it back. Upon a new user joining the GDM they could just request the room's keys.

The room's key's can also be re-generated by the GDM's host.

#### Private Communities

Private communities work similarly to how Group DMs work with the addition that the posts may also be encrypted but follow the same foundations.

## Federation

Eludris will be federated, meaning anyone can host their own instance and instances can communicate with other instances so that any user on one instance can interact with others on any other instance.

### Side note about federation

Making your own implementation in the language you prefer is actually encouraged, same with forking this one and adding new stuff to it, just make sure to give your new features an id and name so that clients that work with multiple instances can
use your features & not break if you made substantial changes, incidentally everything included in the official Eludris implementation at Eludris/eludris is called `base` and has an id of `0`, all other implementations should at least have them.

Features are acquired by sending a `GET` request to an instance's `/` route besides the other elements of the [info](#info) payload.

### Federation Implementation

All routes where other instances can request / submit data will have an
additional `/external` route (like `/external/this/channels/:channel_id/`).

For info about how IDs are created read [this](#ids).

`/external` routes will follow specific rules, these being:

A new instance (one the home instance has never seen before) will have to first send an `identify` payload, this payload is simple as it just provides a shared **private** key that both instances can identify each other with (in case an instance's domain get's compromised) and the instance's id.

`/external` routes will take both Oprish payloads and Pandemonium payloads in the form of HTTP requests (let's say an instance A has a community with a channel that has user's from other instances, one of which is B, when a user from instance B sends a message to `B's domain/external/A's ID/channels/:channel_id/messages`, B will send the Oprish message payload to `A's domain/external/this/channels/:channelid/message`, and when a user from instance A sends a message the opposite will happen with A sending a request to B's external route).

I'm sure this implementation has some edge cases which may cause some issues but I'm hoping to iron them out and document them here once we encounter them.

## Miscellaneous info

### IDs

A Eludris ID is a 64 bit (8 byte) number, structured like so.

```explaination-please-stfu-md
 12345678  12345678  12345678  12345678  12345678  12345678  12345678  12345678
 TTTTTTTT  TTTTTTTT  TTTTTTTT  TTTTTTTT  IIIIIIII  IIIIIIII  IIIIIIII  SSSSSSSS
╰──────────────────────────────────────╯╰────────────────────────────╯╰────────╯
                   │                                   │                   │
                   │                                   │ 8 bit (1 byte) sequence.
                   │                      24 bits (3 byte) Instance ID.
      32 bit (4 byte) Unix Timestamp.
```

T: A Unix timestamp with the Eludris epoch (1,650,000,000).

I: The id of the instance that generated this ID.

S: The sequence number of this ID

An instance ID is a 24 bit (3 byte) number, structured likes so.

```explaination-please-stfu-md
 12345678  12345678  12345678
 TTTTTTTT  TTTTTTTT  NNNNNNNN
╰──────────────────╯╰────────╯
         │              │
         │ 8 bit (1 byte) representing the first character in the instance's name.
16 bit (2 byte) partial unix timestamp.
```

T: The first 16 bits of the current Unix timestamp (also with the Eludris) epoch.

N: The 8 bits representing the first character in the instance name.

### Redis

Eludris uses a non persistent Redis instance to store data that should be really fast to fetch and is ephemeral like rate-limit info.

Here's the structure of currently available keys:

- rate_limit:\<user-id>:\<method>:\<route>

### Internal names

Some of the Eludris components have names that are used internally by the
Eludris team or are referenced directly in the source code.

Here are some of these names:

- Das Ding: The Eludris logo.
- Thang: The Eludris mascot.
- Oprish: The Eludris RESTful API.
- Pandemonium: The Eludris websocket based gateway.
- Effis: The Eludris file server, proxy and CDN.
- Todel: The Eludris model and shared logic crate.
- Carnage: The official Eludris frontend.

## How It Works

Now, Eludris is split into 4 main parts which if you have read this far in would have an idea of, they are:

- Oprish: The Eludris RESTful API.
- Pandemonium: The Eludris websocket based gateway.
- Effis: The Eludris file server, proxy and CDN.
- Todel: The Eludris model and shared logic crate.

These tiny micro-services are tied together using Apache Kafka and the docker-compose in eludris/eludris, how they actually work is as follows:

Oprish waits for HTTP requests from clients, does validation and authentication to make sure that everything is right, then dispatches an event to the Apache Kafka topic which Pandemonium listens to and determines who to dispatch the event to.

When a user connects to Pandemonium it fetches the relative data about the user such as what communities it's in or what roles and permissions it has which allows it to provide data when the client connects besides the End-To-End-Encryption keys and also allows it to filter what events each client gets delivered.

Effis is responsible for handling files and being a CDN besides being a media proxy which means that it stores files and attachments (and their related info), serves them and also can generate embed previews from links.

Todel is a lib crate that houses all the Eludris models and shared logic for Oprish, Pandemonium and Effis to reuse and rely on.

## API Spec

This section discusses how the Eludris v0.2.0 API will work from a client point of view.

### Models

#### Feature

A model representing a set of features an Eludris instance may have.

The point of this model is to make it so that different clients can work for different instances with different features without anything breaking.

Anything in the official implementation of Eludris (present at Eludris/eludris) is under the feature `base` and has an id of `0`

| field | type     | description                      |
| ----- | -------- | -------------------------------- |
| name  | `String` | The name of this set of features |
| id    | `u32`    | The ID of this set of features   |

#### Info

A model representing an Eludris instance's info.

| field         | type                   | description                    |
| ------------- | ---------------------- | ------------------------------ |
| instance_name | `String`               | The name of this instance      |
| features      | `HashMap<u32, String>` | The features this instance has |

#### Message

A model representing an Eludris message.

| field   | type     | description                                |
| ------- | -------- | ------------------------------------------ |
| author  | `String` | The name of the user who sent this message |
| content | `String` | The content of the message                 |

### Oprish HTTP Methods

#### Get Instance Info

Route:
`/`

Method:
`GET`

URL parameters:
None

Request JSON:
None

Response JSON:
An [info](#info) object

#### Send A Message

Route:
`/messages/`

Method:
`POST`

URL parameters:
None

Request JSON:
A [message](#message) object

Response JSON:
A [message](#message) object

#### Interacting With Pandemonium

To actually be able to properly use Eludris you will need to establish a Pandemonium connection which is essentially the programming equivalent of sending yourself straight to hell's boiler room.

Contrary to how it may seem, going to hell is actually very easy, all you need to do is connect to the websocket and send a Ping frame every 20 seconds and tada! Pandemonium will start sending you events.

The current payload you can get as of Eludris v0.2.0 is a [message](#message) payload.
