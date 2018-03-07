# Dementia

Small rust library for the [Matrix protocol](https://matrix.org/)

## Status

Right now, only joining rooms, sending text messages and receiving text messages are supported. Support for room creation and receiving other kinds of messages are planned for near time.

## Usage

In order to connect to a Matrix homeserver and join a room, you need a user on that homeserver and an access token for that user.

```rust
    let serverinfo = ServerInfo {
        server_name: "https://matrix.org".to_owned(), // The Matrix homeserver
        access_token : "DAx…3wo".to_owned()           // The Matrix user access token
    };
```

With that, you can create a Homeserver object

```rust
    let connection = MatrixHomeserver::new(serverinfo);
```

and use this object to join rooms: 

```rust
    let mut room = connection.join("#bottest:https://matrix.org".to_owned());
```
(You need to join a room you want to interact with even if you are already joined. This is for the library to obtain the room id.)

You receive new messages with `connection.get_new_messages()` (which returns a `Vector<String>` of all messages since last called) and send messages with `connection.send_message()` (which takes a `String`).


## Example

```rust
extern crate dementia;

use dementia::{ServerInfo, LoginInfo, MatrixHomeserver, MatrixRoom};
use std::{thread, time};

fn main() {
    let serverinfo = ServerInfo {
        server_name: "https://matrix.org".to_owned(), // The Matrix homeserver
        access_token : "DAx…3wo".to_owned()           // The Matrix user access token
    };

    let connection = MatrixHomeserver::new(serverinfo);
    // The room must already exist
    let mut room = connection.join("#bottest:https://matrix.org".to_owned()); 
        
    let five_sec = time::Duration::new(5, 0);
    loop {
        for message in connection.get_new_messages() {
            if message == "hi" {
                connection.send_message("ahoi!".to_owned());
            }
        }
        thread::sleep(five_sec);
    }
}
```

If you don't have an access token (yet) but the server supports password authentication, you can let the library generate its own access token:

```rust
    let logininfo = LoginInfo {
        server_name: "https://matrix.org".to_owned(),
        username : "@username:matrix.org".to_owned(),
        password : "password".to_owned()
    };
    
    let connection = MatrixHomeserver::login(logininfo);
```