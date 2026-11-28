#![cfg_attr(not(feature = "use-std"), no_std)]

use postcard_rpc::{TopicDirection, endpoints, topics};
use postcard_schema::Schema;
use serde::{Deserialize, Serialize};

endpoints! {
    list = ENDPOINTS;
    | EndpointTy   | RequestTy   | ResponseTy   | Path   |
    | ------------ | ----------- | ------------ | ------ |
    | WakeEndpoint | WakeRequest | WakeResponse | "wake" |
}

#[derive(Serialize, Deserialize, Schema)]
pub struct WakeRequest;

#[derive(Serialize, Deserialize, Schema)]
pub struct WakeResponse;

topics! {
    list = UPLINK_TOPICS;
    direction = TopicDirection::ToClient;
    | TopicTy     | MessageTy   | Path     | Cfg |
    | ----------- | ----------- | -------- | --- |
    | HeightTopic | HeightEvent | "height" |     |
}

#[derive(Serialize, Deserialize, Schema)]
pub struct HeightEvent {
    pub height: u8,
}

topics! {
    list = DOWNLINK_TOPICS;
    direction = TopicDirection::ToServer;
    | TopicTy     | MessageTy   | Path     | Cfg |
    | ----------- | ----------- | -------- | --- |
}
