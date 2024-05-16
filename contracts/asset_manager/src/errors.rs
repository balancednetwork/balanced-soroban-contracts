use soroban_sdk::contracterror;

#[contracterror]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContractError {
    InvalidRlpLength = 1,
    InvalidRollbackMessage = 2
}