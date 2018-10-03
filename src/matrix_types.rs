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
pub struct Invite {}

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
    Message(EventMessage),
    #[serde(rename = "m.room.member")]
    Member,
}

#[derive(Deserialize, Clone)]
pub struct EventMessage {
    sender: String,
    content: Content,
}

#[derive(Deserialize, Clone)]
#[serde(tag = "msgtype")]
pub enum Content{
    #[serde(rename = "m.text")]
    Message(MessageText),
}

#[derive(Deserialize, Clone)]
pub struct MessageText {
    body: String,
}





#[test]
fn deserialize_example() {
    use serde_json;
    //    use super::*;

    const SYNC_EXAMPLE_VALUE : &'static str = include_str!("../test/sync_example.json");

     
    let msg : Sync = serde_json::from_str(SYNC_EXAMPLE_VALUE).unwrap();


    assert_eq!(msg.next_batch, "s72595_4483_1934");
}
