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
    pub fn join(&self, room_name : String) -> MatrixRoom {
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
        let info: JoinInfo = res.json().unwrap();
        MatrixRoom {
            id: info.room_id,
            latest_since: None,
            client : self.client.clone(),
            info : self.info.clone(),
        }            
    }
}

extern crate rand;

impl MatrixRoom {
    pub fn get_new_messages(&mut self) -> Vec<String> {
        match self.latest_since.clone() {
            None => {    
                let mut res = self.client.get(
                    &format!("{}{}{}{}{}",
                             self.info.server_name,
                             "/_matrix/client/r0/sync?filter={\"room\":{\"rooms\":[\"",
                             utf8_percent_encode(&self.id, PATH_SEGMENT_ENCODE_SET).to_string(),
                             "\"],\"timeline\":{\"limit\":1}}}&access_token=",
                             utf8_percent_encode(&self.info.access_token, ACCESS_TOKEN_ENCODE_SET).to_string()))
                    .send()
                    .unwrap();
                //println!("{}",res.text().unwrap());
                let info: SinceInfo = res.json().unwrap();
                self.latest_since = Some(info.next_batch);
                Vec::new()
            },
            Some(since) => {
                let mut res = self.client.get(
                    &format!("{}{}{}{}{}{}{}",
                             self.info.server_name,
                             "/_matrix/client/r0/sync?since=",
                             since,
                             "&filter={\"room\":{\"rooms\":[\"",
                             utf8_percent_encode(&self.id, PATH_SEGMENT_ENCODE_SET).to_string(),
                             "\"]}}&access_token=",
                             utf8_percent_encode(&self.info.access_token, ACCESS_TOKEN_ENCODE_SET).to_string()))
                    .send()
                    .unwrap();
                let mut vec = Vec::new();

                let v: Value = serde_json::from_str(&(res.text().unwrap())).unwrap();
                self.latest_since = Some(v["next_batch"].as_str().unwrap().to_owned());
                match v["rooms"]["join"][&self.id]["timeline"]["events"].as_array() {
                    Some(eventlist) =>               
                        for event in eventlist {
                            match event["content"]["msgtype"].as_str() {
                                Some("m.text") => vec.push(event["content"]["body"].as_str().unwrap().to_owned()),
                                _ => ()
                            }
                        },
                    _ => ()
                }

                vec
            }
        }
            
    }
    pub fn send_message(&self, message: String ) {
        let mut map : HashMap<String,String> = HashMap::new();
        
        map.insert("msgtype".to_owned() , "m.text".to_owned());
        map.insert("body".to_owned() , message);
    
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
}


