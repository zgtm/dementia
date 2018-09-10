extern crate dementia;

use dementia::{Homeserver, Message, RoomEvent};
use std::{thread, time};

fn main() {
    let argv: Vec<String> = std::env::args().collect();
    if argv.len() != 5 {
        println!("Usage: {} <homeserver_address> <user_id> <password> <room_id>", argv[0]);
        println!("Got: {:?}", argv);
        println!("(Don't forget to escape the '#' in your shell!)");
        return;            
    }
    
    let mut server = Homeserver::new(&argv[1])
        .username(&argv[2])
        .password(&argv[3])
        .login().connect();
    
    let mut room = match server.join_room(argv[4].clone()) {
        Some(r) => r,
        _ => {
            println!("Joining room '{}' failed!", argv[4]);
            return;
        }
    };
    
    loop {
        // Follow all invites
        for invite in server.get_invites() {
            server.join_room(invite);
        }
        
        // Reply to message "hi" in room given as parameter
        for event in room.get_new_messages() {
            match event {
                RoomEvent::Message(Message::Text(text)) => {
                    println!("{}", text);
                    if text == "hi" {
                        room.send_notice("ahoi!".to_owned());
                    }
                },
                _ => ()
            }
        }
        thread::sleep(time::Duration::new(10, 0));
    }
}
