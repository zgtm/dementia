use std::collections::HashMap;

pub type RoomId = String;

#[derive(Deserialize, Clone)]
pub struct SyncMsg {
    #[serde(rename = "application/json")]
    pub json: SyncMsgJson,
}

#[derive(Deserialize, Clone)]
pub struct SyncMsgJson {
    pub next_batch: Option<String>,
    pub rooms: SyncRooms
}

#[derive(Deserialize, Clone)]
pub struct SyncRooms {
    pub invite: HashMap<RoomId, SyncRoomsInvite>,
    pub join: HashMap<RoomId, SyncRoomsJoin>,
}



#[derive(Deserialize, Clone)]
pub struct SyncRoomsInvite { }


#[derive(Deserialize, Clone)]
pub struct SyncRoomsJoin {
    pub timeline: SyncRoomsJoinTimeline,
}


#[derive(Deserialize, Clone)]
pub struct SyncRoomsJoinTimeline {
    pub prev_batch: String,
    pub events: Vec<RoomEvent>,
}

#[derive(Deserialize, Clone)]
#[serde(tag = "type")]
pub enum RoomEvent {
    #[serde(rename = "m.room.message")]
    Message{
        sender: String,
        content: MessageContent,
    },
    #[serde(rename = "m.room.member")]
    Member,
    #[serde(rename = "m.room.redaction")]
    Redaction,
}



#[derive(Deserialize, Clone)]
#[serde(tag = "msgtype")]
pub enum MessageContent {
    #[serde(rename = "m.text")]
    Text{ body: String },
    #[serde(rename = "m.emote")]
    Emote { body: String },
    #[serde(rename = "m.notice")]
    Notice { body: String },
    #[serde(rename = "m.file")]
    File { body: String, url: String },
    #[serde(rename = "m.image")]
    Image { body: String, url: String },
    #[serde(rename = "m.video")]
    Video { body: String, url: String },
    #[serde(rename = "m.audio")]
    Audio { body: String, url: String },
    #[serde(rename = "m.location")]
    Location { body: String, geo_uri: String },
}
