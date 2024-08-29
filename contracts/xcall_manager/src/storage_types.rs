use soroban_sdk::contracttype;

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Registry,
    Admin,
    ProposedProtocolToRemove,
    Config,
    Sources,
    Destinations,
    WhiteListedActions
}
