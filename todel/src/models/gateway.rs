use serde::{Deserialize, Serialize};

use super::{
    Category, CategoryEdit, InstanceInfo, MemberEdit, Message, Sphere, SphereChannel,
    SphereChannelEdit, SphereEdit, Status, User,
};
use crate::conf::RateLimitConf;

/// Pandemonium websocket payloads sent by the server to the client.
#[autodoc(category = "Gateway")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(tag = "op", content = "d")]
pub enum ServerPayload {
    /// A [`ClientPayload`] `PING` payload response.
    ///
    /// -----
    ///
    /// ### Example
    ///
    /// ```json
    /// {
    ///   "op": "PONG"
    /// }
    /// ```
    Pong,
    /// The payload sent when the client gets gateway rate limited.
    ///
    /// The client is supposed to wait `wait` milliseconds before sending any more events,
    /// otherwise they are disconnected.
    ///
    /// -----
    ///
    /// ### Example
    ///
    /// ```json
    /// {
    ///   "op": "RATE_LIMIT",
    ///   "d": {
    ///     "wait": 1010 // 1.01 seconds
    ///   }
    /// }
    /// ```
    RateLimit {
        /// The amount of milliseconds you have to wait before the rate limit ends
        wait: u64,
    },
    /// The payload sent by the server when you initiate a new gateway connection.
    ///
    /// -----
    ///
    /// ### Example
    ///
    /// ```json
    /// {
    ///   "op": "HELLO",
    ///   "d": {
    ///     "heartbeat_interval": 45000,
    ///     "instance_info": {
    ///       "instance_name": "EmreLand",
    ///       "description": "More based than Oliver's instance (trust)",
    ///       "version": "0.3.3",
    ///       "message_limit": 2048,
    ///       "oprish_url": "https://example.com",
    ///       "pandemonium_url": "https://example.com",
    ///       "effis_url": "https://example.com",
    ///       "file_size": 20000000,
    ///       "attachment_file_size": 100000000
    ///     },
    ///     "rate_limit": {
    ///       "reset_after": 10,
    ///       "limit": 5
    ///     }
    ///   }
    /// }
    /// ```
    Hello {
        /// The amount of milliseconds your ping interval is supposed to be.
        heartbeat_interval: u64,
        /// The instance's info.
        ///
        /// This is the same payload you get from the [`get_instance_info`] payload without
        /// ratelimits
        instance_info: Box<InstanceInfo>,
        /// The pandemonium ratelimit info.
        rate_limit: RateLimitConf,
    },
    /// The payload sent when the client has successfully authenticated. This contains the data the
    /// user needs on startup.
    ///
    /// -----
    ///
    /// ### Example
    ///
    /// ```json
    /// {
    ///   "op": "AUTHENTICATED",
    ///   "user": {
    ///     "id": 48615849987334,
    ///     "username": "barbaz",
    ///     "social_credit": 3,
    ///     "badges": 0,
    ///     "permissions": 0
    ///   },
    ///   "spheres": [ ... ]
    /// }
    /// ```
    Authenticated {
        user: User,
        /// The spheres that the user is a part of.
        spheres: Vec<Sphere>,
    },
    /// The payload received when a user updates themselves. This includes both user updates from
    /// the [`edit_user`] endpoint and profile updates from the [`edit_profile`] endpoint.
    ///
    /// -----
    ///
    /// ### Example
    ///
    /// ```json
    /// {
    ///   "id": 48615849987333,
    ///   "username": "foobar",
    ///   "social_credit": 42,
    ///   "badges": 0,
    ///   "permissions": 0
    /// }
    /// ```
    UserUpdate(User),
    /// The payload sent when a user's presence is updated.
    ///
    /// This is mainly used for when a user goes offline or online.
    ///
    /// -----
    ///
    /// ### Example
    ///
    /// ```json
    /// {
    ///   "user_id": 48615849987333,
    ///   "status": {
    ///     "type": "IDLE",
    ///     "text": "BURY THE LIGHT DEEP WITHIN"
    ///   }
    /// }
    /// ```
    PresenceUpdate {
        user_id: u64,
        status: Status,
    },
    /// The payload sent when the client receives a [`Message`].
    ///
    /// -----
    ///
    /// ### Example
    ///
    /// ```json
    /// {
    ///   "op": "MESSAGE_CREATE",
    ///   "d": {
    ///     "author": "A Certain Woo",
    ///     "content": "Woo!"
    ///   }
    /// }
    /// ```
    MessageCreate(Message),
    /// The payload sent when a client joins a sphere.
    ///
    /// -----
    ///
    /// ### Example
    ///
    /// ```json
    /// {
    ///   "op": "SPHERE_JOIN",
    ///   "d": {
    ///     "id": 4080402038786,
    ///     "owner_id": 4080403808259,
    ///     "name": "Spehre",
    ///     "type": "HYBRID",
    ///     "description": "Truly the sphere of all time",
    ///     "icon": 4080412852228,
    ///     "badges": 0,
    ///     "channels": [ ... ],
    ///     "members": [ ... ]
    ///   }
    /// }
    /// ```
    SphereJoin(Sphere),
    /// The payload sent when another user joins a sphere the client is in.
    ///
    /// -----
    ///
    /// ### Example
    ///
    /// ```json
    /// {
    ///   "op": "SPHERE_MEMBER_JOIN",
    ///   "d": {
    ///     "user": {
    ///       "id": 48615849987333,
    ///       "username": "foobar",
    ///       "social_credit": 42,
    ///       "badges": 0,
    ///       "permissions": 0
    ///     },
    ///     "sphere_id": 48615849987337
    ///   }
    /// }
    /// ```
    SphereMemberJoin {
        user: User,
        sphere_id: u64,
    },
    /// The payload sent when a category is created in a sphere the client is in.
    ///
    /// -----
    ///
    /// ### Example
    ///
    /// ```json
    /// {
    ///   "op": "CATEGORY_CREATE",
    ///   "d": {
    ///     "category": {
    ///       "id": 5473905934337,
    ///       "name": "iberia",
    ///       "position": 4,
    ///       "channels": []
    ///     },
    ///     "sphere_id": 5461801828355
    ///   }
    /// }
    /// ```
    CategoryCreate {
        category: Category,
        sphere_id: u64,
    },
    /// The payload sent when a category is edited in a sphere the client is in.
    ///
    /// -----
    ///
    /// ### Example
    ///
    /// ```json
    /// {
    ///   "op": "CATEGORY_EDIT",
    ///   "d": {
    ///     "data": { "name": "hyperion-fumo", "position": null },
    ///     "category_id": 5462748233731,
    ///     "sphere_id": 5461801828355
    ///   }
    /// }
    /// ```
    CategoryUpdate {
        /// An object containing the validated changes to the category.
        data: CategoryEdit,
        /// The id of the category that was changed.
        category_id: u64,
        /// The id of the guild in which the category was changed.
        sphere_id: u64,
    },
    /// The payload sent when a category is deleted in a sphere the client is in.
    ///
    /// -----
    ///
    /// ### Example
    ///
    /// ```json
    /// {
    ///   "op": "CATEGORY_DELETE",
    ///   "d": {
    ///     "category_id": 5461814149129,
    ///     "sphere_id": 5461801828355
    ///   }
    /// }
    /// ```
    CategoryDelete {
        /// The id of the category that was deleted.
        category_id: u64,
        /// The id of the sphere from which the category was deleted.
        sphere_id: u64,
    },
    /// The payload sent when a channel is created in a sphere the client is in.
    ///
    /// -----
    ///
    /// ### Example
    ///
    /// ```json
    /// {
    ///   "op": "SPHERE_CHANNEL_CREATE",
    ///   "d": {
    ///     "channel": {
    ///       "type": "TEXT",
    ///       "id": 5473965965314,
    ///       "sphere_id": 5461801828355,
    ///       "name": "lungmen",
    ///       "topic": "uwoogh",
    ///       "position": 2,
    ///       "category_id": 5462748233731
    ///     },
    ///     "sphere_id": 5461801828355
    ///   }
    /// }
    /// ```
    SphereChannelCreate {
        /// The channel that was created.
        channel: SphereChannel,
        /// The id of the sphere in which the channel was created.
        sphere_id: u64,
    },
    /// The payload sent when a channel is edited in a sphere the client is in.
    ///
    /// -----
    ///
    /// ### Example
    ///
    /// ```json
    /// {
    ///   "op": "SPHERE_CHANNEL_EDIT",
    ///   "d": {
    ///     "data": {
    ///       "name": "wscat",
    ///       "topic": "dont forget to ping the websocket",
    ///       "position": 3,
    ///       "category_id": 5461801828355
    ///     },
    ///     "channel_id": 5461813690375,
    ///     "sphere_id": 5461801828355
    ///   }
    /// }
    /// ```
    SphereChannelUpdate {
        /// An object containing the validated changes to the channel.
        data: SphereChannelEdit,
        /// The id of the channel that was edited.
        channel_id: u64,
        /// The id of the sphere in which the channel was edited.
        sphere_id: u64,
    },
    /// The payload sent when a channel is deleted in a sphere the client is in.
    ///
    /// -----
    ///
    /// ### Example
    ///
    /// ```json
    /// {
    ///   "op": "SPHERE_CHANNEL_DELETE",
    ///   "d": {
    ///     "channel_id": 5461813690375,
    ///     "sphere_id": 5461801828355
    ///   }
    /// }
    /// ```
    SphereChannelDelete {
        /// The id of the channel that was deleted.
        channel_id: u64,
        /// The id of the guild from which the channel was deleted.
        sphere_id: u64,
    },
    SphereUpdate {
        data: SphereEdit,
        sphere_id: u64,
    },
    MemberUpdate {
        data: MemberEdit,
        user_id: u64,
        sphere_id: u64,
    },
}

/// Pandemonium websocket payloads sent by the client to the server.
#[autodoc(category = "Gateway")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(tag = "op", content = "d")]
pub enum ClientPayload {
    /// The payload the client is supposed to periodically send the server to not get disconnected.
    ///
    /// The interval where these pings are supposed to be sent can be found in the `HELLO` payload
    /// of the [`ServerPayload`] enum.
    ///
    /// -----
    ///
    /// > **Note**
    /// >
    /// > You are supposed to send your first ping in a connection after `RAND * heartbeat_interval`
    /// > seconds, `RAND` being a random floating number between 0 and 1.
    /// >
    /// > This is done to avoid immediately overloading Pandemonium by connecting if it ever has to go down.
    ///
    /// ### Example
    ///
    /// ```json
    /// {
    ///   "op": "PING"
    /// }
    /// ```
    Ping,
    /// The first payload the client is supposed to send. The data of this payload is expected to
    /// be a session token obtained from the [`create_session`] route.
    ///
    /// -----
    ///
    /// ### Example
    ///
    /// ```json
    /// {
    ///   "op": "AUTHENTICATE",
    ///   "d": "eyJhbGciOiJIUzI1NiJ9.eyJ1c2VyX2lkIjoyMzQxMDY1MjYxMDU3LCJzZXNzaW9uX2lkIjoyMzQxMDgyNDMxNDg5fQ.j-nMmVTLXplaC4opGdZH32DUSWt1yD9Tm9hgB9M6oi4" // You're not supposed to use this example token (eckd)
    /// }
    /// ```
    Authenticate(String),
}
