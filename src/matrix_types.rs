use std::collections::HashMap;

pub type RoomId = String;

// Not used in the current version of the client-server API (r0)
// might be used in future versions (unstable)
/*#[derive(Deserialize, Clone)]
pub struct SyncMsg {
    #[serde(rename = "application/json")]
    pub json: Sync,
}*/

#[derive(Deserialize, Clone)]
pub struct Sync {
    pub next_batch: String,
    pub rooms: Rooms,
    // other interesting fields include
    // * presence
    // * account_data
}

#[derive(Deserialize, Clone)]
pub struct Rooms {
    pub invite: HashMap<RoomId, Invite>,
    pub join: HashMap<RoomId, Join>,
}

#[derive(Deserialize, Clone)]
pub struct Invite {
    pub invite_state: InviteState,
}

#[derive(Deserialize, Clone)]
pub struct InviteState {
    pub events: Vec<StrippedEvent>,
}

#[derive(Deserialize, Clone)]
pub struct Join {
    timeline: Timeline,
}

#[derive(Deserialize, Clone)]
pub struct Timeline {
    prev_batch: String,
    events: Vec<Event>,
}

#[derive(Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Event {
    #[serde(rename = "m.room.message")]
    Message(MessageEvent),
    #[serde(rename = "m.room.member")]
    Member(MemberEvent),
}

#[derive(Deserialize, Clone)]
#[serde(tag = "type")]
pub enum StrippedEvent {
    #[serde(rename = "m.room.member")]
    Member(MemberEvent),
    #[serde(rename = "m.room.name")]
    Name(RoomNameEvent),
}


#[derive(Deserialize, Clone)]
pub struct MessageEvent {
    sender: String,
    content: MessageContent,
    event_id: String,
    origin_server_ts: u64, //?
}

#[derive(Deserialize, Clone)]
#[serde(tag = "msgtype")]
pub enum MessageContent{
    #[serde(rename = "m.text")]
    Text(TextMessageBody),
    #[serde(rename = "m.emote")]
    Emote(TextMessageBody),
    #[serde(rename = "m.notice")]
    Notice(TextMessageBody),
    #[serde(rename = "m.image")]
    Image(MultimediaMessageBody),
    #[serde(rename = "m.file")]
    File(FileMessageBody),
    #[serde(rename = "m.video")]
    Video(MultimediaMessageBody),
    #[serde(rename = "m.audio")]
    Audio(MultimediaMessageBody),
    #[serde(rename = "m.location")]
    Location(LocationMessageBody),
}

#[derive(Deserialize, Clone)]
pub struct TextMessageBody {
    body: String,
}

#[derive(Deserialize, Clone)]
pub struct FileMessageBody {
    body: String,
    filename: String,
    url: Option<String>,
    file: EncryptedFile,
}

#[derive(Deserialize, Clone)]
pub struct MultimediaMessageBody {
    body: String,
    url: Option<String>,
    file: EncryptedFile,
}

#[derive(Deserialize, Clone)]
pub struct LocationMessageBody {
    body: String,
    geo_uri: String,
}

#[derive(Deserialize, Clone)]
pub struct MemberEvent {
    sender: String,
    content: MemberContent,
}

#[derive(Deserialize, Clone)]
pub struct RoomNameEvent {
    sender: String,
    content: RoomNameContent,
}

#[derive(Deserialize, Clone)]
#[serde(tag = "membership")]
pub enum MemberContent {
    #[serde(rename = "invite")]
    Invite,
    #[serde(rename = "join")]
    Join,
    #[serde(rename = "leave")]
    Leave,
    #[serde(rename = "ban")]
    Ban,
    #[serde(rename = "knock")]
    Knock,
}

#[derive(Deserialize, Clone)]
pub struct RoomNameContent {
    name: String,
}


#[derive(Deserialize, Clone)]
pub struct EncryptedFile {
    url: String,
    key: Jwk,
    iv: String,
    hashes: HashMap<String, String>,
    v: String,
}

#[derive(Deserialize, Clone)]
pub struct Jwk {
    key: String,
    key_opts: Vec<String>,
    alg: String,
    k: String,
    ext: bool,
}
        


#[test]
fn deserialize_example() {
    use serde_json;
    //    use super::*;

    const SYNC_EXAMPLE_VALUE : &'static str = include_str!("../test/sync_example.json");

     
    let msg : Sync = serde_json::from_str(SYNC_EXAMPLE_VALUE).unwrap();


    assert_eq!(msg.next_batch, "s72595_4483_1934");
}
