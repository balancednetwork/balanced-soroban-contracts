#![no_std]

mod admin;
pub mod contract;
mod storage_types;
mod states;
mod test;
mod errors;
mod config;
mod messages;

pub use crate::contract::XcallManagerClient;