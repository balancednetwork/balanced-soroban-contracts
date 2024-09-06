
**Balanced Stellar spoke contracts**

This document outlines the major changes in the implementation of Balanced in the Stellar blockchain and the rationale behind these changes. Key updates includes the merge of `deposit native` and `deposit` method on AssetManager contract, introduction of `extend_ttl` method on all contracts, introduction of `upgrade` method to upgrade the contract etc

For more details on each spoke contracts see: [Balanced Crosschain Docs](https://github.com/balancednetwork/balanced-java-contracts/blob/420-balanced-docs/docs/crosschain.md)
  

For more details on the Balanced Protocol see [Balanced Docs](https://github.com/balancednetwork/balanced-java-contracts/blob/420-balanced-docs/docs/docs.md) or [Balanced Network](https://balanced.network/)

  

1.  **Merge of `deposit native` and `deposit` method**
**Change:**  The methods deposit and deposit native are commonly available on AssetManager contract in other languages, but in stellar(also in SUI) both are merge and deposit method has been used as merged method.

**Rationale:**
* The stellar native token can be accessed by Stellar Asset Contract (SAC) and the SAC contract address can be used for the token activities, the same way the Stellar Token Contract (STC) is used for the token activities
  
2.  **Introduction of `extend_ttl` method**
**Change:** New method extend_ttl has been introduced on stellar contracts, which will be used to extend ttl of the storages by the contract admin by paying the required rent periodically (however there is not authentication, anyone can extend the ttl)

**Rationale:** There are three types of the storages in stellar, Temporary Storage, Instance Storage and Persistence Storage. Balanced has used Instance Storage and Persistence Storage, Temporary Storage has not been required in balanced. The rent paying by user does not seem logical as the applicability of rent is not per transaction but for specified period, In which many users can make transactions. For the reason, it is designed in the balanced such that rent will be paid by the admin periodically 

3.  **Introduction of `upgrade` method**
**Change:**  `upgrade` method has been introduced in the balanced stellar contracts

**Rationale:**  The upgrade mechanism of stellar contract is different from upgrade mechanism of contracts written on other languages. Stellar contracts can be upgraded simply sending the reference of the hash of the newly installed WASM to the `upgrade` method of the stellar contract. 