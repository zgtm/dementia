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
extern crate rand;

mod sync;

use serde_json::Value;
use std::collections::HashMap;
use std::rc::Rc;

use url::percent_encoding::{utf8_percent_encode, PATH_SEGMENT_ENCODE_SET, USERINFO_ENCODE_SET};
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

pub struct HomeserverBuilder<Username, Password, AccessToken> {
    server: String,
    username: Username,
    password: Password,
    access_token: AccessToken,
}

/// Represents a Matrix homeserver to which an access token has been created
pub struct Homeserver {
    client: Rc<reqwest::Client>,
    info: ServerInfo,
}

/// Represents a Matrix room from which events can be fetched from
pub struct Room {
    id: String,
    latest_since: Option<String>,
    client: Rc<reqwest::Client>,
    info: ServerInfo,
}

pub use sync::RoomEvent;
pub use sync::MessageContent;

impl HomeserverBuilder<(), (), ()> {
    /// Set the access token
    pub fn access_token(self, access_token: &str) -> HomeserverBuilder<(), (), String> {
        HomeserverBuilder {
            server: self.server,
            username: self.username,
            password: self.password,
            access_token: access_token.to_owned(),
        }
    }
}

impl<T> HomeserverBuilder<(), T, ()> {
    /// Set the username
    pub fn username(self, username: &str) -> HomeserverBuilder<String, T, ()> {
        HomeserverBuilder {
            server: self.server,
            username: username.to_owned(),
            password: self.password,
            access_token: (),
        }
    }
}

impl<T> HomeserverBuilder<T, (), ()> {
    /// Set the password
    pub fn password(self, password: &str) -> HomeserverBuilder<T, String, ()> {
        HomeserverBuilder {
            server: self.server,
            username: self.username,
            password: password.to_owned(),
            access_token: (),
        }
    }
}

impl HomeserverBuilder<String, String, ()> {
    /// Log in with the given credentials
    ///
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
    pub fn login(self) -> HomeserverBuilder<String, String, String> {
        let at_info: AccesstokenInfo = {
            let client = reqwest::Client::new();

            let mut res = client
                .get(&format!("{}/_matrix/client/r0/login", self.server))
                .send()
                .unwrap();

            let mut pwlogin: bool = false;

            let v: Value = serde_json::from_str(&(res.text().unwrap())).unwrap();
            match v["flows"].as_array() {
                Some(flowlist) => for flow in flowlist {
                    match flow["type"].as_str() {
                        Some("m.login.password") => {
                            println!("Found login option `m.login.password`");
                            pwlogin = true;
                        }
                        _ => (),
                    }
                },
                _ => (),
            }
            if !pwlogin {
                panic!("Server does not offer the login option `m.login.password`")
            }

            let mut map: HashMap<&str, &str> = HashMap::new();

            map.insert("type", "m.login.password");
            map.insert("user", &self.username);
            map.insert("password", &self.password);

            let mut res = client
                .post(&format!("{}/_matrix/client/r0/login", self.server))
                .json(&map)
                .send()
                .unwrap();

            //println!("{}",res.text().unwrap());
            res.json().unwrap()
        };

        HomeserverBuilder {
            server: self.server,
            username: self.username,
            password: self.password,
            access_token: at_info.access_token,
        }
    }
}

impl<T, R> HomeserverBuilder<T, R, String> {
    pub fn connect(self) -> Homeserver {
        Homeserver {
            client: Rc::new(reqwest::Client::new()),
            info: ServerInfo {
                server_name: self.server,
                access_token: self.access_token,
            },
        }
    }
}

impl Homeserver {
    /// Start creating a new Homeserver object
    ///
    /// This does not create a stateful connection.
    /// It only constructs a `request` object and saves the URL of the Homeserver.
    ///
    /// * `server_name` – The homeserver URL without trailing slash, e. g. `https://matrix.org`
    /// * `access_token` – The access token
    pub fn new(server_url: &str) -> HomeserverBuilder<(), (), ()> {
        HomeserverBuilder {
            server: server_url.to_owned(),
            username: (),
            password: (),
            access_token: (),
        }
    }

    /// Create a new Homeserver object
    ///
    /// This does not create a stateful connection.
    /// It only constructs a `request` object and saves the URL of the Homeserver.
    ///
    /// * `server_name` – The homeserver URL without trailing slash, e. g. `https://matrix.org`
    /// * `access_token` – The access token
    pub fn connect(server_url: &str, access_token: &str) -> Self {
        Self::new(server_url).access_token(access_token).connect()
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
    pub fn login_and_connect(server_url: &str, username: &str, password: &str) -> Self {
        Self::new(server_url)
            .username(username)
            .password(password)
            .login()
            .connect()
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
    pub fn join_room(&self, room_name: String) -> Option<Room> {
        let map: HashMap<String, String> = HashMap::new();

        let mut res = self
            .client
            .post(&format!(
                "{}{}{}{}{}",
                self.info.server_name,
                "/_matrix/client/r0/join/",
                utf8_percent_encode(&room_name, PATH_SEGMENT_ENCODE_SET).to_string(),
                "?access_token=",
                utf8_percent_encode(&self.info.access_token, ACCESS_TOKEN_ENCODE_SET).to_string()
            )).json(&map)
            .send()
            .unwrap();
        let info: Result<JoinInfo, _> = res.json();
        match info {
            Ok(info) => Some(Room {
                id: info.room_id,
                latest_since: None,
                client: self.client.clone(),
                info: self.info.clone(),
            }),
            _ => None,
        }
    }

    /// Creates a new Matrix room on the server and returns a Matrix room object
    ///
    /// The room will be created with the preset `public_chat`.
    /// Thus, everyone can join the room.
    ///
    /// If the room cannot be created or already exists, `None` is returned.
    pub fn create_room(&self, room_name: String) -> Option<Room> {
        let mut map: HashMap<String, String> = HashMap::new();

        map.insert("room_alias_name".to_owned(), room_name);
        map.insert("preset".to_owned(), "public_chat".to_owned());

        let mut res = self
            .client
            .post(&format!(
                "{}/_matrix/client/r0/createRoom?access_token={}",
                self.info.server_name,
                utf8_percent_encode(&self.info.access_token, ACCESS_TOKEN_ENCODE_SET).to_string()
            )).json(&map)
            .send()
            .unwrap();
        let info: Result<JoinInfo, _> = res.json();
        match info {
            Ok(info) => Some(Room {
                id: info.room_id,
                latest_since: None,
                client: self.client.clone(),
                info: self.info.clone(),
            }),
            _ => None,
        }
    }

    /// Get all current invites from the server
    ///
    /// Returns a list of all room, the bot has been invited to.
    pub fn get_invites(&mut self) -> Vec<String> {
        let res = self
            .client
            .get(&format!(
                "{}/_matrix/client/r0/sync?filter={{\"room\":{{\"rooms\":[]}}}}&access_token={}",
                self.info.server_name,
                utf8_percent_encode(&self.info.access_token, ACCESS_TOKEN_ENCODE_SET).to_string()
            )).send();

        // println!("{}",&res.unwrap().text().unwrap());
        // return

        match res {
            Ok(mut res) => {
                let mut vec = Vec::new();

                let v: Value = serde_json::from_str(&(res.text().unwrap())).unwrap();
                match v["rooms"]["invite"].as_object() {
                    Some(invitelist) => for (room, info) in invitelist {
                        for event in info["invite_state"]["events"].as_array().unwrap() {
                            if event["membership"].as_str() == Some("invite") {
                                vec.push(room.to_owned());
                            }
                        }
                    },
                    _ => (),
                }
                vec
            }
            _ => Vec::new(),
        }
    }
}

impl Room {
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
    pub fn get_new_messages(&mut self) -> Vec<sync::RoomEvent> {
        let res = match self.latest_since.clone() {
            None => self.client.get(
                &format!("{}/_matrix/client/r0/sync?filter={{\"room\":{{\"rooms\":[\"{}\"],\"timeline\":{{\"limit\":0}}}}}}&access_token={}",
                         self.info.server_name,
                         utf8_percent_encode(&self.id, PATH_SEGMENT_ENCODE_SET).to_string(),
                         utf8_percent_encode(&self.info.access_token, ACCESS_TOKEN_ENCODE_SET).to_string()))
                .send(),
            Some(since) => self.client.get(
                &format!("{}/_matrix/client/r0/sync?since={}&filter={{\"room\":{{\"rooms\":[\"{}\"]}}}}&access_token={}",
                         self.info.server_name,
                         since,
                         utf8_percent_encode(&self.id, PATH_SEGMENT_ENCODE_SET).to_string(),
                         utf8_percent_encode(&self.info.access_token, ACCESS_TOKEN_ENCODE_SET).to_string()))
                .send()
        };

        match res {
            Ok(mut res) => {
                let text = res.text().unwrap();
                let mut v: sync::SyncMsgJson = serde_json::from_str(&text).unwrap();
                self.latest_since = v.next_batch;
                v.rooms.join.remove(&self.id).map(|room|
                                                    room.timeline.events).unwrap_or_else(|| vec![])
            }
            _ => vec![],
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
    /// ```ignore
    /// let message = RoomEvent::Message::Notice("Hallo".to_owned());
    /// room.send_message(message);
    /// ```
    ///
    /// ```ignore
    /// let logo_url = String::from("https://www.rust-lang.org/logos/rust-logo-128x128.png");
    /// let message = Message::Image{body: "Rust Logo".to_owned(), url: logo_url};
    /// room.send_message(message);
    /// ```
    pub fn send_message(&self, message: MessageContent) {
        let mut map: HashMap<String, String> = HashMap::new();

        match message {
            MessageContent::Text{ body } => {
                map.insert("msgtype".to_owned(), "m.text".to_owned());
                map.insert("body".to_owned(), body);
            }
            MessageContent::Emote{ body } => {
                map.insert("msgtype".to_owned(), "m.emote".to_owned());
                map.insert("body".to_owned(), body);
            }
            MessageContent::Notice{ body } => {
                map.insert("msgtype".to_owned(), "m.notice".to_owned());
                map.insert("body".to_owned(), body);
            }
            MessageContent::Image { body, url } => {
                map.insert("msgtype".to_owned(), "m.image".to_owned());
                map.insert("body".to_owned(), body);
                map.insert("url".to_owned(), url);
            }
            MessageContent::File { body, url } => {
                map.insert("msgtype".to_owned(), "m.image".to_owned());
                map.insert("body".to_owned(), body);
                map.insert("url".to_owned(), url);
            }
            MessageContent::Location { body, geo_uri } => {
                map.insert("msgtype".to_owned(), "m.image".to_owned());
                map.insert("body".to_owned(), body);
                map.insert("geo_uri".to_owned(), geo_uri);
            }
            MessageContent::Audio { body, url } => {
                map.insert("msgtype".to_owned(), "m.image".to_owned());
                map.insert("body".to_owned(), body);
                map.insert("url".to_owned(), url);
            }
            MessageContent::Video { body, url } => {
                map.insert("msgtype".to_owned(), "m.image".to_owned());
                map.insert("body".to_owned(), body);
                map.insert("url".to_owned(), url);
            }
        }
        self.client
            .put(&format!(
                "{}/_matrix/client/r0/rooms/{}/send/m.room.message/{}?access_token={}",
                self.info.server_name,
                utf8_percent_encode(&self.id, PATH_SEGMENT_ENCODE_SET).to_string(),
                rand::random::<u64>(),
                utf8_percent_encode(&self.info.access_token, ACCESS_TOKEN_ENCODE_SET).to_string()
            )).json(&map)
            .send()
            .unwrap();
    }

    /// Invite someone to a room
    ///
    /// * `user_id` – The fully qualified user ID of the invitee.
    pub fn invite(&self, user_id: &str) {
        let mut map: HashMap<&str, &str> = HashMap::new();
        map.insert("user_id", user_id);
        self.client
            .put(&format!(
                "{}/_matrix/client/r0/rooms/{}/invite?access_token={}",
                self.info.server_name,
                utf8_percent_encode(&self.id, PATH_SEGMENT_ENCODE_SET).to_string(),
                utf8_percent_encode(&self.info.access_token, ACCESS_TOKEN_ENCODE_SET).to_string()
            )).json(&map)
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
    /// ```ignore
    /// room.send_text("Hallo".to_owned());
    /// ```
    pub fn send_text(&self, text: String) {
        self.send_message(MessageContent::Text{body: text});
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
    /// ```ignore
    /// room.send_emote("is having trouble".to_owned());
    /// ```
    pub fn send_emote(&self, text: String) {
        self.send_message(MessageContent::Emote{body: text});
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
    /// ```ignore
    /// room.send_notice("Hallo".to_owned());
    /// ```
    pub fn send_notice(&self, text: String) {
        self.send_message(MessageContent::Notice{body: text});
    }
}

#[cfg(test)]
mod tests;


