---
title: 'What the Heck is an Eludris?'
description: 'Ever wondered what the heck an Eludris was? This page is just the thing for you then!'
order: 0
---

<div align="center">
  <img width="300em" src="https://github.com/eludris/.github/blob/main/assets/das_ding.svg?raw=true" />
  <br>
  <br>
  <a style="border: unset;" href="https://discord.gg/vV6v2DhWQB"><img src="https://shields.io/discord/980412957060137001?style=for-the-badge&logo=discord&labelColor=363934&label=Discord%20Server&color=363934" /></a>
  <a style="border: unset;" href="https://reddit.com/r/eludris"><img src="https://img.shields.io/static/v1?&label=%20&style=for-the-badge&message=Reddit&logo=reddit&logoColor=ff5700&color=121212&labelColor=121212" /></a>
  <a style="border: unset;" href="https://twitter.com/eludris"><img src="https://img.shields.io/static/v1?&label=%20&style=for-the-badge&message=Twitter&logo=twitter&color=15202b&labelColor=15202b" /></a>
</div>

# What the Heck is an Eludris

Eludris is a user-centric Discord x Reddit mesh-like social media platform that
values privacy through [End-To-End Encryption](https://en.wikipedia.org/wiki/End-to-end_encryption)
where you can build a community that feels like your own.

To use Eludris, you can use the [official web client](https://app.eludris.com).

One advantage that Eludris offers is that it allows and encourages people to make
their own clients, toolings, bots, API wrappers and so on. Eludris facilitates
this through the existence of the [Eludris Community organisation](https://github.com/eludris-community)
where people can add their own creations and the [Eludris Awesome repository](https://github.com/eludris/awesome),
in addition to these docs.

Consider joining our [Discord Server](https://discord.gg/amMHHjzwb9) since that
is currently mainly where we hang out, joke around and work on Eludris and Eludris
related stuff.

## Reading the Docs

The documentation is straightforward, however here are some additional clarifications
on some of the points where you can get confused.

### Omittability and Nullability

While reading the docs you might notice the use of the question marks next to field
names or types, here's an example of that.

| Field | Type    |
| ----- | ------- |
| Foo?  | Integer |
| Bar   | String? |

Now, you may be wondering what these question marks are doing there. It depends
on where the question mark is, if the question mark is next to the field name --
like in `Foo?` -- that field is omittable, meaning that it is not guaranteed to
exist. Otherwise in a case where it exists it is **guaranteed** to be an Integer.

On the other hand if the question mark is next to the type -- like in `String?` -- it means
that, while the field will always be present, its value can be `null`.
