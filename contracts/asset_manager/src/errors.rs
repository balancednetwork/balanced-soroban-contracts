use soroban_sdk::contracterror;

#[contracterror]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContractError {
    InvalidRlpLength = 1,
    InvalidRollbackMessage = 2,
    ContractAlreadyInitialized = 3,
    PercentageShouldBeLessThanOrEqualToPOINTS = 4,
    ExceedsWithdrawLimit = 5,
    AmountIsLessThanMinimumAmount = 6,
    ProtocolMismatch = 7,
    OnlyICONAssetManager = 8,
    OnlyCallService = 9,
    UnknownMessageType = 10,
    AdminRequired = 11,
    TokenExists = 12,
    InvalidAddress = 13,
}
