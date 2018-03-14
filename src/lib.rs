#[macro_use]
extern crate serde_derive;
extern crate reqwest;
extern crate serde_json;
#[macro_use]
extern crate url;

use serde_json::{Value};

use std::collections::HashMap;

#[derive(Deserialize, Debug)]
struct JoinInfo {
    room_id: String,
}

#[derive(Deserialize, Debug)]
struct SinceInfo {
    next_batch: String,
}

#[derive(Deserialize, Debug)]
struct AccesstokenInfo {
    access_token: String,
}

#[derive(Deserialize, Debug)]
pub struct LoginInfo {
    pub server_name: String,
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ServerInfo {
    pub server_name: String,
    pub access_token: String,
}


use std::rc::Rc;

pub struct MatrixRoom {
    id: String,
    latest_since: Option<String>,
    client : Rc<reqwest::Client>,
    info : ServerInfo,
}

pub struct MatrixHomeserver {
    client : Rc<reqwest::Client>,
    info: ServerInfo,
}

use url::percent_encoding::{utf8_percent_encode, USERINFO_ENCODE_SET,  PATH_SEGMENT_ENCODE_SET};

define_encode_set! {
    pub ACCESS_TOKEN_ENCODE_SET = [USERINFO_ENCODE_SET] | {
        '%', '&'
    }
}

pub enum Message {
    Text(String),
    Emote(String),
    Notice(String),
    Image{ body: String, url: String},
    File{ body: String, url: String},
    Location{ body: String, geo_uri: String},
    Video{ body: String, url: String},
    Audio{ body: String, url: String}
}

pub enum RoomEvent {
    Message(Message),
    Name(String),
    Topic(String),
    Avatar{ url: String},
}

impl MatrixHomeserver {
    pub fn new(info : ServerInfo) -> Self {
        MatrixHomeserver {
            client : Rc::new(reqwest::Client::new()),
            info: info
        }
    }
    pub fn login(info : LoginInfo) -> Self {
        let client = reqwest::Client::new();

        let mut res = client.get(
            &format!("{}/_matrix/client/r0/login",
                     info.server_name))
            .send()
            .unwrap();

        let mut pwlogin : bool = false;
        
        let v: Value = serde_json::from_str(&(res.text().unwrap())).unwrap();
        match v["flows"].as_array() {
             Some(flowlist) =>               
                for flow in flowlist {
                    match flow["type"].as_str() {
                        Some("m.login.password") => {println!("Found login option `m.login.password`"); pwlogin = true;}
                        _ => ()
                    }
                },
            _ => ()
        }
        if !pwlogin {panic!("Passwort-Login nicht möglich")}

        let mut map : HashMap<String,String> = HashMap::new();

        map.insert("type".to_owned() , "m.login.password".to_owned());
        map.insert("user".to_owned() , info.username);
        map.insert("password".to_owned() , info.password);
    
        let mut res = client.post(
            &format!("{}/_matrix/client/r0/login",
                     info.server_name))
            .json(&map)
            .send()
            .unwrap();

        //println!("{}",res.text().unwrap());
        let at_info: AccesstokenInfo = res.json().unwrap();
        MatrixHomeserver {
            client : Rc::new(client),
            info: ServerInfo {
                server_name: info.server_name,
                access_token: at_info.access_token,
            }
        }
    }
    pub fn join_room(&self, room_name : String) -> Option<MatrixRoom> {
        let map : HashMap<String,String> = HashMap::new();
        
        let mut res = self.client.post(
            &format!("{}{}{}{}{}",
                    self.info.server_name,
                    "/_matrix/client/r0/join/",
                    utf8_percent_encode(&room_name, PATH_SEGMENT_ENCODE_SET).to_string(),
                    "?access_token=",
                    utf8_percent_encode(&self.info.access_token, ACCESS_TOKEN_ENCODE_SET).to_string()))
            .json(&map)
            .send()
            .unwrap();
        let info: Result<JoinInfo, _> = res.json();
        match info {
            Ok(info) => Some(MatrixRoom {
                id: info.room_id,
                latest_since: None,
                client : self.client.clone(),
                info : self.info.clone(),
            }),
            _ => None
        }        
    }
    pub fn create_room(&self, room_name : String) -> Option<MatrixRoom> {
        let mut map : HashMap<String,String> = HashMap::new();
        
        map.insert("room_alias_name".to_owned() , room_name);
        map.insert("preset".to_owned() , "public_chat".to_owned());
        
        let mut res = self.client.post(
            &format!("{}/_matrix/client/r0/createRoom?access_token={}",
                     self.info.server_name,
                     utf8_percent_encode(&self.info.access_token, ACCESS_TOKEN_ENCODE_SET).to_string()))
            .json(&map)
            .send()
            .unwrap();
        let info: Result<JoinInfo, _> = res.json();
        match info {
            Ok(info) => Some(MatrixRoom {
                id: info.room_id,
                latest_since: None,
                client : self.client.clone(),
                info : self.info.clone(),
            }),
            _ => None
        }
    }
}

extern crate rand;

impl MatrixRoom {
    /// Receive all new events in a room since the last time this function has
    /// been called
    ///
    /// The first time this function is called, since there is no last time,
    /// only general information about the room is encoded in the events.
    /// In all later calls, the new messages are included.
    ///
    /// Note: To prevent prevent infinite-loop situations between bots, a bot
    /// should never reply to a message of type `notice` and should only reply
    /// with messages of type `notice`.
    ///
    /// # Examples
    ///
    pub fn get_new_messages(&mut self) -> Vec<RoomEvent> {
        match self.latest_since.clone() {
            None => {    
                let mut res = self.client.get(
                    &format!("{}/_matrix/client/r0/sync?filter={{\"room\":{{\"rooms\":[\"{}\"],\"timeline\":{{\"limit\":1}}}}}}&access_token={}",
                             self.info.server_name,
                             utf8_percent_encode(&self.id, PATH_SEGMENT_ENCODE_SET).to_string(),
                             utf8_percent_encode(&self.info.access_token, ACCESS_TOKEN_ENCODE_SET).to_string()))
                    .send();
                match res {
                    Ok(mut res) => {
                        let info: SinceInfo = res.json().unwrap();
                        self.latest_since = Some(info.next_batch);
                    },
                    _ => ()
                }
                Vec::new()
                //println!("{}",res.text().unwrap());
            },
            Some(since) => {
                let mut res = self.client.get(
                    &format!("{}/_matrix/client/r0/sync?since={}&filter={{\"room\":{{\"rooms\":[\"{}\"]}}}}&access_token={}",
                             self.info.server_name,
                             since,
                             utf8_percent_encode(&self.id, PATH_SEGMENT_ENCODE_SET).to_string(),
                             utf8_percent_encode(&self.info.access_token, ACCESS_TOKEN_ENCODE_SET).to_string()))
                    .send();
                match res {
                    Ok(mut res) => {
                        let mut vec = Vec::new();
                        
                        let v: Value = serde_json::from_str(&(res.text().unwrap())).unwrap();
                        self.latest_since = Some(v["next_batch"].as_str().unwrap().to_owned());
                        match v["rooms"]["join"][&self.id]["timeline"]["events"].as_array() {
                            Some(eventlist) =>               
                                for event in eventlist {
                                    match event["content"]["msgtype"].as_str() {
                                        Some("m.text") => vec.push(RoomEvent::Message(Message::Text(event["content"]["body"].as_str().unwrap().to_owned()))),
                                        Some("m.emote") => vec.push(RoomEvent::Message(Message::Emote(event["content"]["body"].as_str().unwrap().to_owned()))),
                                        Some("m.notice") => vec.push(RoomEvent::Message(Message::Notice(event["content"]["body"].as_str().unwrap().to_owned()))),
                                        Some("m.image") => vec.push(RoomEvent::Message(Message::Image
                                                                                       { body: event["content"]["body"].as_str().unwrap().to_owned(),
                                      url: event["content"]["url"].as_str().unwrap().to_owned() })),
                                        Some("m.file") => vec.push(RoomEvent::Message(Message::File
                                                                                      { body: event["content"]["body"].as_str().unwrap().to_owned(),
                                                                                        url: event["content"]["url"].as_str().unwrap().to_owned() })),
                                        Some("m.location") => vec.push(RoomEvent::Message(Message::Location
                                                                                          { body: event["content"]["body"].as_str().unwrap().to_owned(),
                                                                                            geo_uri: event["content"]["geo_uri"].as_str().unwrap().to_owned() })),
                                        Some("m.video") => vec.push(RoomEvent::Message(Message::Audio
                                                                                       { body: event["content"]["body"].as_str().unwrap().to_owned(),
                                                                                         url: event["content"]["url"].as_str().unwrap().to_owned() })),
                                        Some("m.audio") => vec.push(RoomEvent::Message(Message::Audio
                                                                                       { body: event["content"]["body"].as_str().unwrap().to_owned(),
                                                                                         url: event["content"]["url"].as_str().unwrap().to_owned() })),
                                        _ => ()
                                    }
                                },
                            _ => ()
                        }
                        
                        vec
                    },
                    _ => Vec::new()
                }
            }
        }
            
    }

    /// Send a message to a room
    ///
    /// Note: To prevent prevent infinite-loop situations between bots, a bot
    /// should never reply to messages with a message of type `text`.
    /// Instead, a message of type `notice` should be used.
    /// Thus, a bot should also never reply to a message of type `notice`.
    ///
    /// # Examples
    ///
    /// ```
    /// let message = Message::Notice("Hallo".to_owned());
    /// room.send_message(message);
    /// ```
    ///
    /// ```
    /// let logo_url = String::from("https://www.rust-lang.org/logos/rust-logo-128x128.png");
    /// let message = Message::Image{body: "Rust Logo".to_owned(), url: logo_url};
    /// room.send_message(message);
    /// ```
    pub fn send_message(&self, message: Message) {
        let mut map : HashMap<String,String> = HashMap::new();

        match message {
            Message::Text(text) => {
                map.insert("msgtype".to_owned() , "m.text".to_owned());
                map.insert("body".to_owned() , text);
            },
            Message::Emote(text) => {
                map.insert("msgtype".to_owned() , "m.emote".to_owned());
                map.insert("body".to_owned() , text);
            },
            Message::Notice(text) => {
                map.insert("msgtype".to_owned() , "m.notice".to_owned());
                map.insert("body".to_owned() , text);
            },
            Message::Image{body, url} => {
                map.insert("msgtype".to_owned() , "m.image".to_owned());
                map.insert("body".to_owned() , body);
                map.insert("url".to_owned() , url);
            },
            Message::File{body, url} => {
                map.insert("msgtype".to_owned() , "m.image".to_owned());
                map.insert("body".to_owned() , body);
                map.insert("url".to_owned() , url);
            },
            Message::Location{body, geo_uri} => {
                map.insert("msgtype".to_owned() , "m.image".to_owned());
                map.insert("body".to_owned() , body);
                map.insert("geo_uri".to_owned() , geo_uri);
            },
            Message::Audio{body, url} => {
                map.insert("msgtype".to_owned() , "m.image".to_owned());
                map.insert("body".to_owned() , body);
                map.insert("url".to_owned() , url);
            },
            Message::Video{body, url} => {
                map.insert("msgtype".to_owned() , "m.image".to_owned());
                map.insert("body".to_owned() , body);
                map.insert("url".to_owned() , url);
            },
        }    
        self.client.put(
            &format!("{}/_matrix/client/r0/rooms/{}/send/m.room.message/{}?access_token={}",
                     self.info.server_name,                     
                     utf8_percent_encode(&self.id, PATH_SEGMENT_ENCODE_SET).to_string(),
                     rand::random::<u64>(),
                     utf8_percent_encode(&self.info.access_token, ACCESS_TOKEN_ENCODE_SET).to_string())) 
            .json(&map)
            .send()
            .unwrap();
    }

    /// Send a message of type `text` to a room
    ///
    /// Shortcut for `send_message(Message::Text(…))``
    ///
    /// A bot should never reply to messages with a message of type `text`.
    /// Instead, a message of type `notice` should be used.
    ///
    /// # Examples
    ///
    /// ```
    /// room.send_text("Hallo".to_owned());
    /// ```
    pub fn send_text(&self, text: String) {
        self.send_message(Message::Text(text));
    }
    /// Send a message of type `emote` to a room
    ///
    /// Shortcut for `send_message(Message::Emote(…))``
    ///
    /// An emote describes an action that is being performed.
    /// This corresponds to the IRC CTCP ACTION command and is usually induced
    /// by the prefix `/me`.
    /// 
    ///
    /// # Examples
    ///
    /// ```
    /// room.send_emote("is having trouble".to_owned());
    /// ```
    pub fn send_emote(&self, text: String) {
        self.send_message(Message::Emote(text));
    }
    
    /// Send a message of type `notice` to a room
    ///
    /// Shortcut for `send_message(Message::Notice(…))``
    ///
    /// A bot should always use a message of type `notice`, when replying to
    /// messages.
    ///
    /// # Examples
    ///
    /// ```
    /// room.send_notice("Hallo".to_owned());
    /// ```
    pub fn send_notice(&self, text: String) {
        self.send_message(Message::Notice(text));
    }
}


