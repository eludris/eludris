---
title: 'How to Use the Autodoc Inventory'
description: 'How to implement the autodoc inventory into your Eludris API wrapper.'
order: 4
---

The autodoc inventory is a tool that allows you to automatically generate types and route code programmatically.

## Guide

To get started, have a look at the [index](/autodoc/index.json) to see the contents of autodoc. The type of this is [Index](#index), and it contains a list of paths to other items in the inventory.

Each item, when fetched with `/autodoc/<path>`, will be an [ItemInfo](#item-info). The important information here is the `name` key - containing the name of the item, and the `item` key, with info about the actual item's contents.

Items can be an [object](#object), [enum](#enum), or [route](#route).

### Primitive Types

The following are types that do not refer to another item in the inventory.

- `String`
- `u8`, `u16`, `u32`, `u64`, `u128`, `usize`, `i8`, `i16`, `i32`, `i64`, `i128`, `isize` - these are just integers in languages without sizes
- `Boolean`
- `T[]` - an array of `T`
- `IpAddr` - a string representation of an IPv4/IPv6 address. This is given as it is useful to convert to a helper type in your language for a smaller memory footprint (integer representation).

### Objects

An object is simply a collection of fields with names and types. The `fields` key contains a list of [FieldInfo](#field-info) objects, which contain information about the field. An object correlates to a "struct"/"interface"/"class" in most languages.

Each field has a `name` key, which is the name of the field, and a `field_type` key, which is the type of the field. The `field_type` key can be a [primitive type](#primitive-types) (such as `String`), or another item in the inventory (such as `Message`).

There is a `nullable` key, which is a boolean indicating whether the field can be `null` or not. There is also an `ommitable` key, which is a boolean indicating whether the field can be omitted from the object or not.

A field may also be `flattened`, which means that the field is flattened into the object. This means that all the fields in the flattened object are moved into the parent object. This is used when multiple objects have the same fields.

```json
// MessageCreate
{
  "type": "object",
  "fields": [
    {
      "name": "content",
      "doc": "The message's content.",
      "field_type": "String",
      "flattened": false,
      "nullable": false,
      "ommitable": false
    }
  ]
}
// Message
{
  "type": "object",
  "fields": [
    {
      "name": "author",
      "doc": "The message's author.",
      "field_type": "User",
      "flattened": false,
      "nullable": false,
      "ommitable": false
    },
    {
      "name": "message",
      "doc": "The message's data.",
      "field_type": "MessageCreate",
      "flattened": true,
      "nullable": false,
      "ommitable": false
    }
  ]
}
// UpdateUserProfile
{
  "type": "object",
  "fields": [
    {
      "name": "status",
      "doc": "The user's new status. This field cannot be more than 150 characters long.",
      "field_type": "String",
      "flattened": false,
      "nullable": true,
      "ommitable": true
    },
    {
      "name": "status_type",
      "doc": "The user's new status type. This must be one of `ONLINE`, `OFFLINE`, `IDLE` and `BUSY`.",
      "field_type": "StatusType",
      "flattened": false,
      "nullable": false,
      "ommitable": true
    }
  ]
}
```

For example, the above objects relate to the following typescript:

```typescript
interface Message {
  author: User;
  content: string;
}

interface UpdateUserProfile {
  status?: string | null;
  status_type?: StatusType;
}
```

### Enums

An enum has multiple [EnumVariant](#enum-variant)s representing the possible values of this type. There are more things to consider than enums in most languages.

An enum usually has a `tag` which is a key to which variant is present. It is a string pointing to the key containing the tag.

If an enum is `untagged`, the variant would have to be figured out by finding which variant will deserialize correctly. This is only used right now for if each variant is a `unit`, so the correct variant can be figured out by what the value is.

If an enum is not `untagged`, but also does not have a `tag`, this means it is "externally tagged". An external tag means that the tag is in the key, and the variant is in the value.

An example of this in JSON:

```json
{ "image": { "width": 5, "height": 10 } }
```

The enum's `content`, if any, points to a field which contains the enum's content. This may either be the fields of an `object` or the value of a `tuple` variant.

An example of this in JSON:

```json
{ "op": "AUTHENTICATE", "d": "token" }
```

#### Variants

A `unit` variant is a variant without any content. This nicely maps to a normal `enum` in most languages.

```json
{
  "type": "enum",
  "tag": null,
  "untagged": false,
  "content": null,
  "variants": [
    {
      "type": "unit",
      "name": "ONLINE",
      "doc": null
    },
    {
      "type": "unit",
      "name": "OFFLINE",
      "doc": null
    }
  ]
}
```

For example, the above enum results in the following typescript:

```typescript
enum StatusType {
  Online = 'ONLINE',
  Offline = 'OFFLINE'
}
```

A `tuple` variant is a variant where the `content` of the enum is that of `field_type`. This means that if `field_type` is a `String`, the contents of the enum is also a `String`, and if `field_type` is `User`, the contents are the same fields as `User`. For example, in the `AUTHENTICATE` example above, `field_type` is `String`, so the content (`d`) is a string.

An `object` variant is a variant with the contents of `fields`. This is just like an `object` type, except within a variant of an enum. For example, in the `image` example above, the `fields` are `width` and `height`, making the metadata of an image file.

### Routes

A route corresponds to a HTTP `method` and `route`.

The `method` of a route corresponds to the [HTTP method](https://developer.mozilla.org/docs/Web/HTTP/Methods) of the route, and the `route` corresponds to the path.

The `route` may contain `<parameters>` formatted with `<>` around them. The types to fill these in are in the `path_params` field. This is a list of [ParamInfo](#param-info)s that have the `name` and the `param_type` of each named parameter.

The `query_params` field contains the `?query&parameters`. These of course are represented in the URL as strings, but are simply string representations of the parameters. For example, `?intparam=1&boolparam=true`.

The [`body`](#body) field contains information about how to format your request body. `body.type` can refer to a [primitive type](#primitive-types) or a type in the inventory. `body.format` is a [MIME type](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types/Common_types) such as `application/json` or `multipart/form-data`.

The [`response`](#response) field contains information about what the response body is like. `response.type` can refer to a [primitive type](#primitive-types) or a type in the inventory. `response.format` is a [MIME type](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types/Common_types) such as `application/json` or `multipart/form-data`. `response.format` may also be `"raw"` if it is the raw content of a file (such as `Get File` and `Get Attachment`). `response.status_code` refers to the [HTTP status code](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status) of the response. `response.rate_limit` ddefines whether the response will contain rate limit headers.

#### Example

```json
{
  "type": "route",
  "method": "GET",
  "route": "/<id>",
  "path_params": [
    {
      "name": "id",
      "param_type": "u64"
    }
  ],
  "query_params": [],
  "body": null,
  "response": {
    "type": "FetchResponse",
    "format": "raw",
    "status_code": 200,
    "rate_limit": true
  }
}
```

## Types

### Index

| Field   | Type     | Description                                                 |
| ------- | -------- | ----------------------------------------------------------- |
| version | String   | The Eludris version this refers to.                         |
| items   | String[] | The items in the inventory. These are paths to other items. |

#### Example

```json
{
  "version": "0.3.3",
  "items": ["oprish/create_message.json", "todel/Message.json"]
}
```

### Item Info

`ItemInfo` contains metadata about all kinda of items. `category` is used in the public documentation for headers, it has no real meaning otherwise.

| Field    | Type          | Description                                                                          |
| -------- | ------------- | ------------------------------------------------------------------------------------ |
| name     | String        | The name of the item.                                                                |
| doc      | String?       | The documentation of the item.                                                       |
| category | String        | The category of the item.                                                            |
| hidden   | Boolean       | Whether the item is hidden from the public documentation (such as flattened models). |
| package  | String        | The package the item belongs to (oprish, todel, effis)                               |
| item     | [Item](#item) | The item this info refers to.                                                        |

#### Example

```json
{
  "name": "Message",
  "doc": "The message payload. [...]",
  "category": "Messaging",
  "hidden": false,
  "package": "todel",
  "item": [...]
}
```

### Item

An item represents one type or route in the Eludris API.

#### Object

An object is simply a collection of fields with names and types.

| Field  | Type                       | Description          |
| ------ | -------------------------- | -------------------- |
| type   | "object"                   |                      |
| fields | [FieldInfo](#field-info)[] | The object's fields. |

##### Example

```json
{
  "type": "object",
  "fields": [
    {
      "name": "content",
      "doc": "...",
      "field_type": "String",
      "nullable": false,
      "ommitable": false,
      "flattened": false
    }
  ]
}
```

#### Enum

An enum contains multiple variants. It usually has a "tag" to identify which variant it is.

| Field    | Type                           | Description                                                            |
| -------- | ------------------------------ | ---------------------------------------------------------------------- |
| type     | "enum"                         |                                                                        |
| tag      | String?                        | The field which contains the identifying tag.                          |
| untagged | Boolean                        | Whether the enum is untagged.                                          |
| content  | String?                        | The field containing the inner content of the enum, flattened if null. |
| variants | [EnumVariant](#enum-variant)[] | This enum's variants.                                                  |

##### Example

```json
{
  "type": "enum",
  "tag": "op",
  "untagged": false,
  "content": "d",
  "rename_all": "SCREAMING_SNAKE_CASE",
  "variants": [
    {
      "type": "unit",
      "name": "PONG",
      "doc": "A [`ClientPayload`] `PING` payload response."
    }
  ]
}
```

#### Route

| Field          | Type                       | Description                                                                                                                                           |
| -------------- | -------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| type           | "route"                    |                                                                                                                                                       |
| method         | String                     | This route's HTTP method.                                                                                                                             |
| route          | String                     | This route's path. Formatted with `<param>` for path parameters.                                                                                      |
| path_params    | [ParamInfo](#param-info)[] | This route's path parameters.                                                                                                                         |
| query_params   | [ParamInfo](#param-info)[] | This route's query parameters.                                                                                                                        |
| body           | [Body](#body)              | Info relating to the body of the request.                                                                                                             |
| response       | [Response](#response)      | Info relating to the response of the request.                                                                                                         |
| requires_auth? | Boolean                    | Whether this route requires `Authorization`. `false` means it is preferred (higher rate limits) but not required. Not present means no authorization. |

##### Example

```json
{
  "type": "route",
  "method": "POST",
  "route": "/messages",
  "path_params": [],
  "query_params": [],
  "body": {
    "type": "Message",
    "format": "application/json"
  },
  "response": {
    "type": "Message",
    "format": "application/json",
    "status_code": 201,
    "rate_limit": true
  },
  "requires_auth": true
}
```

### Field Info

| Field      | Type    | Description                                                              |
| ---------- | ------- | ------------------------------------------------------------------------ |
| name       | String  | The field's name (key).                                                  |
| doc        | String? | The field's documentation.                                               |
| field_type | String  | The field's type.                                                        |
| nullable   | Boolean | Whether the field is nullable.                                           |
| ommitable  | Boolean | Whether the field is ommitable (may not exist in the payload).           |
| flattened  | Boolean | Whether the field's contents are flattened onto the encompassing object. |

#### Example

```json
{
  "name": "shared",
  "doc": null,
  "field_type": "SharedErrorData",
  "nullable": false,
  "ommitable": false,
  "flattened": true
}
```

### Enum Variant

A variant is one of the possible values of an enum. They can be of the following kinds:

#### Unit

A unit variant is a variant without any content.

| Field | Type    | Description                  |
| ----- | ------- | ---------------------------- |
| name  | String  | The variant's name.          |
| doc   | String? | The variant's documentation. |

##### Example

```json
{
  "type": "unit",
  "name": "PONG",
  "doc": "A [`ClientPayload`] `PING` payload response."
}
```

#### Tuple

A tuple variant is a variant where the content is the fields of the `field_type`.

| Field      | Type    | Description                  |
| ---------- | ------- | ---------------------------- |
| name       | String  | The variant's name.          |
| doc        | String? | The variant's documentation. |
| field_type | String  | The type of the tuple.       |

#### Example

```json
{
  "type": "tuple",
  "name": "MESSAGE_CREATE",
  "doc": "The event sent when the client receives a [`Message`]",
  "field_type": "Message"
}
```

#### Object

An object variant contains other fields as the content, just like an object item.

| Field  | Type                       | Description                  |
| ------ | -------------------------- | ---------------------------- |
| name   | String                     | The variant's name.          |
| doc    | String?                    | The variant's documentation. |
| fields | [FieldInfo](#field-info)[] | The variant's fields.        |

##### Example

```json
{
  "type": "object",
  "name": "RATE_LIMIT",
  "doc": "The event sent when the client gets gateway rate limited.",
  "fields": [
    {
      "name": "wait",
      "doc": "The amount of milliseconds you have to wait before the rate limit ends",
      "field_type": "u64",
      "nullable": false,
      "ommitable": false,
      "flattened": false
    }
  ]
}
```

### Param Info

| Field      | Type   | Description           |
| ---------- | ------ | --------------------- |
| name       | String | The parameter's name. |
| param_type | String | The parameter's type. |

#### Example

```json
{
  "name": "rate_limits",
  "param_type": "bool"
}
```

### Body

| Field  | Type   | Description                            |
| ------ | ------ | -------------------------------------- |
| type   | String | The body's type.                       |
| format | String | The body's format (mime-type or `raw`) |

#### Example

```json
{
  "type": "Message",
  "format": "application/json"
}
```

### Response

| Field       | Type    | Description                                       |
| ----------- | ------- | ------------------------------------------------- |
| type        | String  | The response's type.                              |
| format      | String  | The response's format (mime-type or `raw`)        |
| status_code | Integer | The response's HTTP status code.                  |
| rate_limit  | Boolean | Whether the response contains rate limit headers. |

#### Example

```json
{
  "type": "Message",
  "format": "application/json",
  "status_code": 201,
  "rate_limit": true
}
```
