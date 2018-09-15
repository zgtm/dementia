
use serde_json;


use super::*;

const SYNC_EXAMPLE_VALUE : &'static str = include_str!("sync_example.json");

#[test]
fn deserialize_example() {
    let msg : sync::SyncMsg = serde_json::from_str(SYNC_EXAMPLE_VALUE).unwrap();

    assert_eq!(msg.json.next_batch, Some("s72595_4483_1934".into()));
}
