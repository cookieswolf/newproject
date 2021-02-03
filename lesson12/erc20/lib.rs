#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod erc20 {

    use ink_storage::collections::HashMap as StorageHashMap;

    #[ink(storage)]
    pub struct Erc20 {
        /// Stores a single `bool` value on the storage.
        total_supply: Balance,
        balances: StorageHashMap<AccountId, Balance>,
        allowances: StorageHashMap<(AccountId, AccountId), Balance>,
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        to: AccountId,
        value: Balance,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InsufficientBalance,
        InsufficientAllowance,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl Erc20 {
        /// Constructor that initializes the `bool` value to the given `init_value`.
        #[ink(constructor)]
        pub fn new(total_supply: Balance) -> Self {
            let caller = Self::env().caller();
            let mut balances = StorageHashMap::new();
            balances.insert(caller, total_supply);
            let instance = Self {
                total_supply: total_supply,
                balances: balances,
                allowances: StorageHashMap::new(),
            };

            instance
        }

        /// 公共方法：2种 1.读 2.写
        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            *self.total_supply
        }

        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            *self.balances.get(&owner).unwrap_or(&0)
        }

        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            *self.allowances.get(&(owner, spender)).unwrap_or(&0)
        }

        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
            let who = Self::env().caller();
            self.transfer_help(who, to, value)
        }

        #[ink(message)]
        pub fn transfer_from(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<()> {
            let caller = self.env().caller();
            let allowance = self.allowance(from, caller);
            if allowance < value {
                return Err(Error::InsufficientAllowance)
            }
            self.transfer_help(from, to, value);
            self.allowances.insert((from, caller), allowance - value);
            Ok(())
        }
        #[ink(message)]
        pub fn burn(&mut self, value: Balance) {
            let who = Self::env().caller();
            let balance = self.balance_of(who);
            if balance < value {
                self.balances.insert(who, 0);
            } else {
                self.balances.insert(who, balance - value);
            }
        }
        #[ink(message)]
        pub fn issue(&mut self, to: AccountId, value: Balance) -> Result<()> {
            let current_balance = self.balance_of(to);
            self.balances.insert(to, current_balance + value);
            Ok(())
        }
        fn transfer_help(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<()> {
            let from_balance = self.balance_of(from);
            if from_balance < value {
                return Err(Error::InSufficientBalance);
            }
            self.balances.insert(from, from_balance - value);
            let to_balance = self.balance_of(to);
            self.balances.insert(to, to_balance + value);

            self.env().emit_event(Transfer{
                from: from,
                to: to,
                value: value,
            });

            Ok(())
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// We test if the default constructor does its job.
        #[test]
        fn create_contract_works() {
            let erc20 = Erc20::new(1000);
            assert_eq!(erc20.total_supply(), 1000);
        }
    }
}
