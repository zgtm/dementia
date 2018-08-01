# Dementia

Small rust library for the [Matrix protocol](https://matrix.org/)

## Status

Right now, only the following are supported:

  * joining rooms
  * sending text messages and
  * receiving text messages

Support for room creation and receiving other kinds of messages are planned for the near time.

## Usage

In order to connect to a Matrix homeserver and join a room, you need a user on that homeserver and an access token for that user.

```rust
    let server_url = "https://matrix.org"; // The Matrix homeserver
    let access_token = "DAx…3wo";          // The Matrix user access token
```

With that, you can create a Homeserver object

```rust
    let connection = Homeserver::new(server_url)
        .access_token(access_token)
        .connect();
```

or alternatively

```rust
    let connection = MatrixHomeserver::connect(server_url, access_token);
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

use dementia::{Homeserver, Room};
use std::{thread, time};

fn main() {
    let server_url = "https://matrix.org"; // The Matrix homeserver
    let access_token = "DAx…3wo";          // The Matrix user access token

    let conn = Homeserver::new(server_url)
        .access_token(access_token)
        .connect();
    // The room must already exist
    let mut room = conn.join("#bottest:https://matrix.org".to_owned()); 
        
    let five_sec = time::Duration::new(5, 0);
    loop {
        for message in conn.get_new_messages() {
            if message == "hi" {
                conn.send_message("ahoi!".to_owned());
            }
        }
        thread::sleep(five_sec);
    }
}
```

If you don't have an access token (yet) but the server supports password authentication, you can let the library generate its own access token:

```rust   
    let connection = Homeserver::new("https://matrix.org")
        .username("@example:matrix.org")
        .password("examplepassword")
        .login()
        .connect();
```

Subsequentlty, you can retrieve the access token for future connections using

```rust
    access_token = connection.get_access_token();
```
