#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::codec::{Codec, Decode, Encode};
use frame_support::traits::Vec;
use frame_support::{
    decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure, Parameter,
};
use frame_system::{self, ensure_signed};
use sp_runtime::traits::{AtLeast32BitUnsigned, CheckedAdd, CheckedSub, Member};

pub trait Trait: frame_system::Trait{
    type Event: From<Event<Self>>+Into<<Self as frame_system::Trait>::Event>;
    type TokenBalance:CheckedAdd + CheckedSub + Parameter + Member + Codec + Default + Copy + AtLeast32BitUnsigned;
}
#[derive(Encode,Decode,Default,Clone,PartialEq,Debug)]
pub struct Erc20Token<U> {
    name:Vec<u8>,
    ticker:Vec<u8>,
    total_supply:U,
}



decl_storage! {
    trait Store for Module<T: Trait> as Erc20{
        TokenId get(fn token_id): u32;
        Tokens get(fn token_details): map hasher(blake2_128_concat) u32 => Erc20Token<T::TokenBalance>;
        BalanceOf get(fn balance_of): map hasher(blake2_128_concat) (u32, T::AccountId) => T::TokenBalance;
        Allowance get(fn allowance): map hasher(blake2_128_concat) (u32, T::AccountId, T::AccountId) => T::TokenBalance;
    }
}
decl_event! (
    pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId, <T as Trait>::TokenBalance  {
        Transfer(u32,AccountId,AccountId,TokenBalance), 
        Approval(u32,AccountId,AccountId,TokenBalance),
    }
);
 
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        //type Error = Error<T>;
        fn deposit_event()=default;
        #[weight=0]
        fn init(origin, name: Vec<u8>, ticker: Vec<u8>, total_supply: T::TokenBalance)->DispatchResult {
            let sender = ensure_signed(origin)?; 
            ensure!(name.len()<=64,"token name cannot exceed 64 bytes"); 
            ensure!(ticker.len()<=32,"token ticker cannot exceed 32 bytes"); 
            let token_id = Self::token_id();
            let next_token_id = token_id.checked_add(1).ok_or("overflow in calculating next token id")?;
            TokenId::put(next_token_id);
            let token = Erc20Token {
              name,
              ticker,
              total_supply,
          };
          <Tokens<T>>::insert(token_id, token);
          <BalanceOf<T>>::insert((token_id, sender), total_supply);
          Ok(())
        }

        #[weight=0]
        fn transfer(_origin,token_id: u32, to: T::AccountId,value: T::TokenBalance)->DispatchResult{
            let sender = ensure_signed(_origin)?;
            Self::_transfer(token_id, sender,to,value)
        }
        #[weight=0]
        pub fn transfer_from(_origin,token_id: u32, from: T::AccountId,to: T::AccountId,value: T::TokenBalance)->DispatchResult{
            ensure!(<Allowance<T>>::contains_key((token_id, from.clone(), to.clone())), "Allowance does not exist.");
            let allowance = Self::allowance((token_id, from.clone(), to.clone()));
            ensure!(allowance >= value, "Not enough allowance.");
            let updated_allowance = allowance.checked_sub(&value).ok_or("overflow in calculating allowance")?;
            <Allowance<T>>::insert((token_id, from.clone(), to.clone()), updated_allowance);
            Self::deposit_event(RawEvent::Approval(token_id, from.clone(), to.clone(), value));
            Self::_transfer(token_id, from, to, value)
        }

        #[weight=0]
        fn approve(_origin,token_id: u32, spender: T::AccountId,value: T::TokenBalance)->DispatchResult{
          let sender = ensure_signed(_origin)?;
          ensure!(<BalanceOf<T>>::contains_key((token_id, sender.clone())), "Account does not own this token");
          let allowance = Self::allowance((token_id, sender.clone(), spender.clone()));
          let updated_allowance = allowance.checked_add(&value).ok_or("overflow in calculating allowance")?;
          <Allowance<T>>::insert((token_id, sender.clone(), spender.clone()), updated_allowance);
          Self::deposit_event(RawEvent::Approval(token_id, sender.clone(), spender.clone(), value));
          Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    fn _transfer(token_id: u32,from: T::AccountId,to: T::AccountId,value: T::TokenBalance,)->DispatchResult{
        ensure!(
            <BalanceOf<T>>::contains_key((token_id, from.clone())),
            "Account does not own this token"
        );
        let sender_balance = Self::balance_of((token_id, from.clone()));
        ensure!(sender_balance >= value, "Not enough balance.");
        let updated_from_balance = sender_balance
            .checked_sub(&value)
            .ok_or("overflow in calculating balance")?;
        let receiver_balance = Self::balance_of((token_id, to.clone()));
        let updated_to_balance = receiver_balance
            .checked_add(&value)
            .ok_or("overflow in calculating balance")?;
        <BalanceOf<T>>::insert((token_id, from.clone()), updated_from_balance);
        <BalanceOf<T>>::insert((token_id, to.clone()), updated_to_balance);
        Self::deposit_event(RawEvent::Transfer(token_id, from, to, value));
        Ok(())
    }
}