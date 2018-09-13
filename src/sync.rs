use std::collections::HashMap;

pub type RoomId = String;

#[derive(Deserialize, Clone)]
pub struct SyncMsg {
    #[serde(rename = "application/json")]
    pub json: SyncMsgJson,
}

#[derive(Deserialize, Clone)]
pub struct SyncMsgJson {
    pub next_batch: String,
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
    timeline: SyncRoomsJoinTimeline,
}


#[derive(Deserialize, Clone)]
pub struct SyncRoomsJoinTimeline {
    prev_batch: String,
    events: Vec<SyncRoomsJoinTimelineEvent>,
}

#[derive(Deserialize, Clone)]
#[serde(tag = "type")]
pub enum SyncRoomsJoinTimelineEvent {
    #[serde(rename = "m.room.message")]
    Message(SyncRoomsJoinTimelineEventMessage),
    #[serde(rename = "m.room.member")]
    Member,
}

#[derive(Deserialize, Clone)]
pub struct SyncRoomsJoinTimelineEventMessage {
    sender: String,
    content: SyncRoomsJoinTimelineEventMessageContent,
}

#[derive(Deserialize, Clone)]
#[serde(tag = "msgtype")]
pub enum SyncRoomsJoinTimelineEventMessageContent{
    #[serde(rename = "m.text")]
    Message(SyncRoomsJoinTimelineEventMessageContentText),
}

#[derive(Deserialize, Clone)]
pub struct SyncRoomsJoinTimelineEventMessageContentText {
    body: String,
}
