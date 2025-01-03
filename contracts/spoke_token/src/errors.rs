use soroban_sdk::contracterror;

#[contracterror]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContractError {
    InvalidRlpLength = 1,
    ContractAlreadyInitialized = 2,
    DecimalMustFitInAu8 = 3,
    ProtocolMismatch = 4,
    OnlyIconHubToken = 5,
    OnlyCallService = 6,
    UnknownMessageType = 7,
    InvalidAddress = 8,
    InvalidNetworkAddressLength = 9,
    InvalidNetworkAddress = 10,
    InvalidAmount = 11,
    Uninitialized = 12
}
