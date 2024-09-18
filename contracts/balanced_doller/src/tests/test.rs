// #![cfg(test)]
// extern crate std;

// use crate::contract::BalancedDollarClient;


// use soroban_sdk::{
//     symbol_short,
//     testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
//     Address, IntoVal, Symbol,
// };

// use super::setup::*;

// #[test]
// fn test() {
//     let ctx = TestContext::default();
//     let client = BalancedDollarClient::new(&ctx.env, &ctx.registry);
//     let e = &ctx.env;
//     e.mock_all_auths();
//     ctx.init_context(&client);
    

//     let admin2 = Address::generate(&e);
//     let user1 = Address::generate(&e);
//     let user2 = Address::generate(&e);
//     let user3 = Address::generate(&e);

//     client.mint(&ctx.admin, &1000);
//     assert_eq!(
//         e.auths(),
//         std::vec![(
//             ctx.admin.clone(),
//             AuthorizedInvocation {
//                 function: AuthorizedFunction::Contract((
//                     client.address.clone(),
//                     symbol_short!("mint"),
//                     (&ctx.admin, 1000_i128).into_val(e),
//                 )),
//                 sub_invocations: std::vec![]
//             }
//         )]
//     );
//     assert_eq!(client.balance(&ctx.admin), 1000);

//     client.approve(&ctx.admin, &user3, &500, &200);
//     assert_eq!(
//         e.auths(),
//         std::vec![(
//             ctx.admin.clone(),
//             AuthorizedInvocation {
//                 function: AuthorizedFunction::Contract((
//                     client.address.clone(),
//                     symbol_short!("approve"),
//                     (&ctx.admin, &user3, 500_i128, 200_u32).into_val(e),
//                 )),
//                 sub_invocations: std::vec![]
//             }
//         )]
//     );
//     assert_eq!(client.allowance(&ctx.admin, &user3), 500);

//     client.transfer(&ctx.admin, &user2, &600);
//     assert_eq!(
//         e.auths(),
//         std::vec![(
//             ctx.admin.clone(),
//             AuthorizedInvocation {
//                 function: AuthorizedFunction::Contract((
//                     client.address.clone(),
//                     symbol_short!("transfer"),
//                     (&ctx.admin, &user2, 600_i128).into_val(e),
//                 )),
//                 sub_invocations: std::vec![]
//             }
//         )]
//     );
//     assert_eq!(client.balance(&ctx.admin), 400);
//     assert_eq!(client.balance(&user2), 600);

//     client.transfer_from(&user3, &ctx.admin, &user1, &400);
//     assert_eq!(
//         e.auths(),
//         std::vec![(
//             user3.clone(),
//             AuthorizedInvocation {
//                 function: AuthorizedFunction::Contract((
//                     client.address.clone(),
//                     Symbol::new(&e, "transfer_from"),
//                     (&user3, &ctx.admin, &user1, 400_i128).into_val(e),
//                 )),
//                 sub_invocations: std::vec![]
//             }
//         )]
//     );
//     assert_eq!(client.balance(&user1), 400);
//     assert_eq!(client.balance(&user2), 600);

//     client.transfer(&user1, &user3, &300);
//     assert_eq!(client.balance(&user1), 100);
//     assert_eq!(client.balance(&user3), 300);

//     client.set_admin(&admin2);
//     assert_eq!(
//         e.auths(),
//         std::vec![(
//             ctx.admin.clone(),
//             AuthorizedInvocation {
//                 function: AuthorizedFunction::Contract((
//                     client.address.clone(),
//                     symbol_short!("set_admin"),
//                     (&admin2,).into_val(e),
//                 )),
//                 sub_invocations: std::vec![]
//             }
//         )]
//     );

//     // Increase to 500
//     client.approve(&user2, &user3, &500, &200);
//     assert_eq!(client.allowance(&user2, &user3), 500);
//     client.approve(&user2, &user3, &0, &200);
//     assert_eq!(
//         e.auths(),
//         std::vec![(
//             user2.clone(),
//             AuthorizedInvocation {
//                 function: AuthorizedFunction::Contract((
//                     client.address.clone(),
//                     symbol_short!("approve"),
//                     (&user2, &user3, 0_i128, 200_u32).into_val(e),
//                 )),
//                 sub_invocations: std::vec![]
//             }
//         )]
//     );
//     assert_eq!(client.allowance(&user2, &user3), 0);
// }

// #[test]
// fn test_burn() {
//     let ctx = TestContext::default();
//     let client = BalancedDollarClient::new(&ctx.env, &ctx.registry);
//     let e = &ctx.env;
//     e.mock_all_auths();
//     ctx.init_context(&client);

    
//     let user2 = Address::generate(&e);

//     client.mint(&ctx.admin, &1000);
//     assert_eq!(client.balance(&ctx.admin), 1000);

//     client.approve(&ctx.admin, &user2, &500, &200);
//     assert_eq!(client.allowance(&ctx.admin, &user2), 500);

//     client.burn_from(&user2, &ctx.admin, &500);
//     assert_eq!(
//         e.auths(),
//         std::vec![(
//             user2.clone(),
//             AuthorizedInvocation {
//                 function: AuthorizedFunction::Contract((
//                     client.address.clone(),
//                     symbol_short!("burn_from"),
//                     (&user2, &ctx.admin, 500_i128).into_val(e),
//                 )),
//                 sub_invocations: std::vec![]
//             }
//         )]
//     );

//     assert_eq!(client.allowance(&ctx.admin, &user2), 0);
//     assert_eq!(client.balance(&ctx.admin), 500);
//     assert_eq!(client.balance(&user2), 0);

//     client.burn(&ctx.admin, &500);

//      assert_eq!(client.balance(&ctx.admin), 0);
//      assert_eq!(client.balance(&user2), 0);
// }

// #[test]
// #[should_panic(expected = "insufficient balance")]
// fn transfer_insufficient_balance() {
//     let ctx = TestContext::default();
//     let client = BalancedDollarClient::new(&ctx.env, &ctx.registry);
//     let e = &ctx.env;
//     e.mock_all_auths();
//     ctx.init_context(&client);
//     e.mock_all_auths();
//     let user2 = Address::generate(&e);

//     client.mint(&ctx.admin, &1000);
//     assert_eq!(client.balance(&ctx.admin), 1000);

//     client.transfer(&ctx.admin, &user2, &1001);
// }

// #[test]
// #[should_panic(expected = "insufficient allowance")]
// fn transfer_from_insufficient_allowance() {
//     let ctx = TestContext::default();
//     let client = BalancedDollarClient::new(&ctx.env, &ctx.registry);
//     let e = &ctx.env;
//     e.mock_all_auths();
//     ctx.init_context(&client);
//     let user2 = Address::generate(&e);
//     let user3 = Address::generate(&e);

//     client.mint(&ctx.admin, &1000);
//     assert_eq!(client.balance(&ctx.admin), 1000);

//     client.approve(&ctx.admin, &user3, &100, &200);
//     assert_eq!(client.allowance(&ctx.admin, &user3), 100);

//     client.transfer_from(&user3, &ctx.admin, &user2, &101);
// }

