//! A module for writing Matrix bots
//!
//! This module maps some parts of the Matrix client-server protocoll to rust
//! objects.
//! In order to communicate with the Matrix homeserver, this library needs an
//! authentication token (`access_token`).
//! This authentication token can either be given directly, or can be generated
//! using username-password authentication, if the homeserver supports this.
//! With this authentication token, rooms can be joined or created.
//! After as room has been joined, messages can be sent to the room and new
//! messages can be fetched from from the room.

#[macro_use]
extern crate serde_derive;
extern crate reqwest;
extern crate serde_json;
#[macro_use]
extern crate url;

use std::rc::Rc;
use std::collections::HashMap;
use serde_json::{Value};

use url::percent_encoding::{utf8_percent_encode, USERINFO_ENCODE_SET,  PATH_SEGMENT_ENCODE_SET};
define_encode_set! {
    pub ACCESS_TOKEN_ENCODE_SET = [USERINFO_ENCODE_SET] | {
        '%', '&'
    }
}


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


/// The information needed to connect to a homeseverer with an access token
#[derive(Deserialize, Debug, Clone)]
struct ServerInfo {
    /// The homeserver URL without trailing slash, e. g. `https://matrix.org`
    server_name: String,
    /// The access token
    access_token: String,
}

/// Represents a Matrix homeserver to which an access token has been created
pub struct MatrixHomeserver {
    client : Rc<reqwest::Client>,
    info: ServerInfo,
}

/// Represents a Matrix room from which events can be fetched from
pub struct MatrixRoom {
    id: String,
    latest_since: Option<String>,
    client : Rc<reqwest::Client>,
    info : ServerInfo,
}

/// A message received from or to be sent to a room
pub enum Message {
    /// Text message. Should not be used to reply to messages!
    Text(String),
    /// Emote. Represents an action.
    Emote(String),
    /// Notice. Should be used for automatic replies. Should not be replied to!
    Notice(String),
    /// Image file. The URL should be created by uploading to the homesever.
    Image{body:String, url:String},
    /// File. The URL should be created by uploading to the homesever.
    File{body:String, url:String},
    /// Location. `geo_uri` should be a Geo URI.
    /// E. g. `geo:37.786971,-122.399677`.
    Location{body:String, geo_uri:String},
    /// Video file. The URL should be created by uploading to the homesever.
    Video{body:String, url:String},
    /// Audio file. The URL should be created by uploading to the homesever.
    Audio{body:String, url:String}
}

/// An event received from or to be sent to a room
pub enum RoomEvent {
    /// A message in the room.
    Message(Message),
    /// The name of the room.
    Name(String),
    /// The topice of the room.
    Topic(String),
    /// The avatar (an image) of the room.
    Avatar{url:String},
}

impl MatrixHomeserver {
    /// Create a new Homeserver object
    ///
    /// This does not create a stateful connection.
    /// It only constructs a `request` object and saves the URL of the Homeserver.
    ///
    /// * `server_name` – The homeserver URL without trailing slash, e. g. `https://matrix.org`
    /// * `access_token` – The access token
    pub fn new(server_name : &str, access_token : &str) -> Self {
        MatrixHomeserver {
            client : Rc::new(reqwest::Client::new()),
            info: ServerInfo { server_name: server_name.to_owned(), access_token: access_token.to_owned() }
        }
    }

    /// Create a new Homeserver object from username an password combination
    ///
    /// This does not create a stateful connection.
    /// It only constructs a `request` object and saves the URL of the Homeserver.
    ///
    /// * `server_name` – The homeserver URL without trailing slash, e. g. `https://matrix.org`
    /// * `username` – The username without homeserver part, e. g. `bot`
    /// * `password` – The password
    ///
    /// # Panics
    /// If the server does not support or allow simple username and password
    /// login, this function panics.
    pub fn login(server_name:&str, username:&str, password:&str) -> Self {
        let client = reqwest::Client::new();

        let mut res = client.get(
            &format!("{}/_matrix/client/r0/login", server_name))
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

        let mut map : HashMap<&str, &str> = HashMap::new();

        map.insert("type", "m.login.password");
        map.insert("user", &username);
        map.insert("password", &password);
    
        let mut res = client.post(
            &format!("{}/_matrix/client/r0/login",
                     server_name))
            .json(&map)
            .send()
            .unwrap();

        //println!("{}",res.text().unwrap());
        let at_info: AccesstokenInfo = res.json().unwrap();
        MatrixHomeserver {
            client : Rc::new(client),
            info: ServerInfo {
                server_name: server_name.to_owned(),
                access_token: at_info.access_token,
            }
        }
    }

    /// Returns the access token
    ///
    /// This is especially usefull, if you authenticated via username and
    /// password and want to retrieve the access token for later use.
    pub fn get_access_token(&self) -> String {
        return self.info.access_token.clone();
    }

    /// Creates a Matrix room object
    ///
    /// This joins the room.
    /// If the room has already been joined, this function can be called anyway
    /// to only create the room object.
    ///
    /// If the room cannot be joined, `None` is returned
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

    /// Creates a new Matrix room on the server and returns a Matrix room object
    ///
    /// The room will be created with the preset `public_chat`.
    /// Thus, everyone can join the room.
    ///
    /// If the room cannot be created or already exists, `None` is returned.
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
    /// Shortcut for `send_message(Message::Text(…))`
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
    /// Shortcut for `send_message(Message::Emote(…))`
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
    /// Shortcut for `send_message(Message::Notice(…))`
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


