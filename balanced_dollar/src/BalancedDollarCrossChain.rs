use soroban_sdk::{contractimpl, Address, Env, String};

pub mod xcall_manager {
    soroban_sdk::contractimport!(
        file = "../xcall_manager/target/wasm32-unknown-unknown/release/xcall_manager.wasm"
    );
}

pub mod xcall {
    soroban_sdk::contractimport!(
        file = "../xcall/target/wasm32-unknown-unknown/release/xcall.wasm"
    );
}

#[contractimpl]
impl BalancedDollar {

    pub fn cross_transfer(
        from: Address,
        amount: i128,
        to: String,
        value: i128
    ) {
        from.require_auth();
        _cross_transfer(from, amount, to, value, String::from_str(""));
    }

    pub fn cross_transfer(
        from: Address,
        amount: i128,
        to: String,
        value: i128,
        data: BytesN
    ) {
        from.require_auth();
        _crossTransfer(from, amount, to, value, data);
    }

     fn _crossTransfer(
        from: Address,
        amount: i128,
        to: String,
        value: i128,
        data: BytesN
    )  {
        if value <= 0 {
            panic!("Amount less than minimum amount");
        }
       Self::burn(from, value);

        // string memory from = nid.networkAddress(msg.sender.toString());
        // // Validate address
        // to.parseNetworkAddress();
        // Messages.XCrossTransfer memory xcallMessage = Messages.XCrossTransfer(
        //     from,
        //     to,
        //     value,
        //     data
        // );

        // Messages.XCrossTransferRevert memory rollback = Messages.XCrossTransferRevert(
        //     msg.sender,
        //     value
        // );

        let protocols: Vec<String> = xcall_manager::Client::getProtocols();

        // ICallService(xCall).sendCallMessage{value: msg.value}(
        //     iconBnUSD,
        //     xcallMessage.encodeCrossTransfer(),
        //     rollback.encodeCrossTransferRevert(),
        //     protocols.sources,
        //     protocols.destinations
        // );

    }

    pub fn handle_call_message(
        from: String,
        data: BytesN,
        protocols: Vec<String>
    ) {
        xCall.require_auth();
        let protocols: Vec<String> = xcall_manager::Client::getProtocols();

        // string memory method = data.getMethod();
        // if (method.compareTo(Messages.CROSS_TRANSFER)) {
        //     require(from.compareTo(iconBnUSD), "onlyICONBnUSD");
        //     Messages.XCrossTransfer memory message = data.decodeCrossTransfer();
        //     (,string memory to) = message.to.parseNetworkAddress();
        //     _mint(to.parseAddress("Invalid account"), message.value);
        // } else if (method.compareTo(Messages.CROSS_TRANSFER_REVERT)) {
        //     require(from.compareTo(xCallNetworkAddress), "onlyCallService");
        //     Messages.XCrossTransferRevert memory message = data.decodeCrossTransferRevert();
        //     _mint(message.to, message.value);
        // } else {
        //     revert("Unknown message type");
        // }
    }

    
    

}