crate::bitflag! {
    pub enum SpherePermissions {
        ADMINISTRATOR = 1,
        MANAGE_CHANNELS = 2,
        MANAGE_ROLES = 3,
        MANAGE_EMOJIS = 4,
        MANAGE_SERVER = 5,
        CHANGE_NICKNAME = 6,
        MANAGE_NICKNAMES = 7,
        KICK_MEMBERS = 8,
        VIEW_CHANNEL = 9,
        SEND_MESSAGES = 10,
        EMBED_LINKS = 11,
        ATTACH_FILES = 12,
        ADD_REACTIONS = 13,
        MANAGE_MESSAGES = 14,
        READ_MESSAGE_HISTORY = 15,
    }
}
