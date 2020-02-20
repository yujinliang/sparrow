#![allow(dead_code)]
mod node {
const MAX_CONN_COUNT_LIMIT:u64 = 10000;
const MIN_CONN_COUNT_LIMIT:u16 = 30;
const GROW_COUNT:u16 = 15;
const SHRINK_COUNT:u16 = 8;
const IDLE_TIME_TO_SHRINK_THRESHOLD:u64 = 1800; //time unit: second 
const TO_CHECK_TIME_INTERVAL:u64 = 60;//time unit: second 
const PING_RETRY_COUNT: u8 = 3;
const PING_RETRY_MIN_INTERVAL: u16 = 5; //time unit: second 
const RECONNECT_RETRY_COUNT: u8 = 3; 
const RECONNECT_RETRY_MIN_INTERVAL:u16 = 10;//time unit: second 
}
