extern crate dementia;

use dementia::{Homeserver, MessageContent, RoomEvent};
use std::{thread, time};

fn main() {
    let argv: Vec<String> = std::env::args().collect();
    if argv.len() != 4 {
        println!(
            "Usage: {} <homeserver_address> <access_token> <room_id>",
            argv[0]
        );
        println!("Got: {:?}", argv);
        println!("(Don't forget to escape the '#' in your shell!)");
        return;
    }

    let mut server = Homeserver::new(&argv[1]).access_token(&argv[2]).connect();

    let mut room = match server.join_room(argv[3].clone()) {
        Some(r) => r,
        _ => {
            println!("Joining room '{}' failed!", argv[3]);
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
                RoomEvent::Message{
                    content,
                    sender
                } => match content {
                    MessageContent::Text { body} => {
                        println!("{}", body);
                        if body == "hi" {
                            room.send_notice(format!("ahoi, {}!", sender));
                        }
                    }
                    _ => (),
                }
                _ => (),
            }
            
        }
        thread::sleep(time::Duration::new(10, 0));
    }
}
