//! This contract performs a batch of atomic token swaps between multiple
//! parties and does a simple price matching.
//! Parties don't need to know each other and also don't need to know their
//! signatures are used in this contract; they sign the `AtomicSwap` contract
//! invocation that guarantees that their token will be swapped with someone
//! while following the price limit.
//! This example demonstrates how authorized calls can be batched together.
#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Vec};

mod atomic_swap {
    use soroban_sdk::{contract, contracterror, contractimpl, token, Address, Env, IntoVal};

    #[contracterror]
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    #[repr(u32)]
    pub enum AtomicSwapError {
        MinRecvNotMet = 1,
    }

    #[contract]
    pub struct AtomicSwapContract;

    #[contractimpl]
    impl AtomicSwapContract {
        /// Atomic swap between 2 accounts of 2 tokens.
        ///
        /// Both parties must authorize this call with their expected amounts.
        /// The implementation transfers each party's full offered amounts
        /// into the swap contract, then returns the leftovers.
        pub fn swap(
            env: Env,
            acc_a: Address,
            acc_b: Address,
            token_a: Address,
            token_b: Address,
            amount_a: i128,
            min_recv_a: i128,
            amount_b: i128,
            min_recv_b: i128,
        ) -> Result<(), AtomicSwapError> {
            // Require auth for each side with deterministic, contract-specific args.
            acc_a.require_auth_for_args(
                (token_a.clone(), token_b.clone(), amount_a, min_recv_a).into_val(&env),
            );
            acc_b.require_auth_for_args(
                (token_b.clone(), token_a.clone(), amount_b, min_recv_b).into_val(&env),
            );

            // Safety checks: ensure both minimum receive constraints are met.
            if amount_a < min_recv_b || amount_b < min_recv_a {
                return Err(AtomicSwapError::MinRecvNotMet);
            }

            // Actual swap amounts are constrained by the opposite party's min expectations.
            let actual_token_a = min_recv_b; // sent to acc_b
            let actual_token_b = min_recv_a; // sent to acc_a

            let swap_contract_id = env.current_contract_address();
            let token_a_client = token::Client::new(&env, &token_a);
            let token_b_client = token::Client::new(&env, &token_b);

            // Pull full amounts into the swap contract.
            token_a_client.transfer(&acc_a, &swap_contract_id, &amount_a);
            token_b_client.transfer(&acc_b, &swap_contract_id, &amount_b);

            // Pay swap proceeds.
            token_b_client.transfer(&swap_contract_id, &acc_a, &actual_token_b);
            token_a_client.transfer(&swap_contract_id, &acc_b, &actual_token_a);

            // Refund leftovers to each party.
            let refund_a = amount_a - actual_token_a;
            let refund_b = amount_b - actual_token_b;
            if refund_a > 0 {
                token_a_client.transfer(&swap_contract_id, &acc_a, &refund_a);
            }
            if refund_b > 0 {
                token_b_client.transfer(&swap_contract_id, &acc_b, &refund_b);
            }

            Ok(())
        }
    }
}

#[derive(Clone)]
#[contracttype]
pub struct SwapSpec {
    pub address: Address,
    pub amount: i128,
    pub min_recv: i128,
}

#[contract]
pub struct AtomicMultiSwapContract;

#[contractimpl]
impl AtomicMultiSwapContract {
    // Swap token A for token B atomically between the parties that want to
    // swap A->B and parties that want to swap B->A.
    // All the parties should have authorized the `swap` via `swap_contract`,
    // but they don't need to authorize `multi_swap` itself.
    pub fn multi_swap(
        env: Env,
        swap_contract: Address,
        token_a: Address,
        token_b: Address,
        swaps_a: Vec<SwapSpec>,
        swaps_b: Vec<SwapSpec>,
    ) {
        let mut remaining_b = swaps_b;
        let swap_client = atomic_swap::AtomicSwapContractClient::new(&env, &swap_contract);

        fn matches(a: &SwapSpec, b: &SwapSpec) -> bool {
            a.amount >= b.min_recv && a.min_recv <= b.amount
        }

        for acc_a in swaps_a.iter() {
            let mut i = 0;
            while i < remaining_b.len() {
                let acc_b = remaining_b.get(i).unwrap();

                if matches(&acc_a, &acc_b)
                    && swap_client
                        .try_swap(
                            &acc_a.address,
                            &acc_b.address,
                            &token_a,
                            &token_b,
                            &acc_a.amount,
                            &acc_a.min_recv,
                            &acc_b.amount,
                            &acc_b.min_recv,
                        )
                        .is_ok()
                {
                    // If it succeeds, clear this B swap and move to the next A.
                    remaining_b.remove(i);
                    break;
                }

                i += 1;
            }
        }
    }
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod test;