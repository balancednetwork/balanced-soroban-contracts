#![no_std]

pub mod admin;
pub mod contract;
pub mod storage_types;
mod test;
pub mod states;
mod config;
mod errors;
mod messages;
mod xcall_manager_interface;

pub use crate::contract::AssetManagerClient;