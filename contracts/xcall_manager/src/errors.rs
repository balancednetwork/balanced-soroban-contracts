use soroban_sdk::contracterror;

#[contracterror]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContractError {
    InvalidRlpLength = 1,
    ContractAlreadyInitialized = 3,
    AmountIsLessThanMinimumAmount = 6,
    ProtocolMismatch = 7,
    OnlyICONGovernance = 8,
    OnlyCallService = 9,
    UnknownMessageType = 10,
    AdminRequired = 11,
    NoProposalForRemovalExists = 12,
    NotWhiteListed = 13,
}
