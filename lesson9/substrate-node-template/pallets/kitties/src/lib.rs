#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use frame_support::{decl_module, decl_storage, decl_event, decl_error, ensure,Parameter,dispatch::DispatchResult,
                    traits::{Randomness,Currency, ExistenceRequirement::AllowDeath, ReservableCurrency},
};
use sp_io::hashing::blake2_128; 
use frame_system::{self as system, ensure_signed};
use sp_runtime::{DispatchError, traits::{AtLeast32Bit, Bounded, Member}};
use crate::doublelinkedlist::{LinkedList, LinkedItem};
//设计的数据结构双链表 
mod doublelinkedlist;

#[cfg(test)]
mod mock;

//type KittyIndex = u32;


#[derive(Encode, Decode)]
pub struct Kitty(pub [u8; 16]); 

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Randomness: Randomness<Self::Hash>;
    //KittyIndex
    type KittyIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    type Currency: Currency<Self::AccountId>+ReservableCurrency<Self::AccountId>;
}
type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

type KittyLinkedItem<T> = LinkedItem<<T as Trait>::KittyIndex>;
type OwnerKittiesList<T> = LinkedList<OwnerKitties<T>, <T as system::Trait>::AccountId, <T as Trait>::KittyIndex>;
decl_storage! {
	trait Store for Module<T: Trait> as Kitties {
        pub Kitties get(fn kitties): map hasher(blake2_128_concat) T::KittyIndex => Option<Kitty>;
        
		pub KittiesCount get(fn kitties_count): T::KittyIndex;
		pub KittyOwners get(fn kitty_owner): map hasher(blake2_128_concat) T::KittyIndex => Option<T::AccountId>;
        
        //扩展存储
        pub OwnerKitties get(fn owner_kitties): map hasher(blake2_128_concat) (T::AccountId, Option<T::KittyIndex>) => Option<KittyLinkedItem<T>>;
	}
}

decl_event!(
    pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId,
    KittyIndex = <T as Trait>::KittyIndex,
    Balance = BalanceOf<T>,
	BlockNumber = <T as system::Trait>::BlockNumber,
    {
		Created(AccountId, KittyIndex),
        Transferred(AccountId, AccountId, KittyIndex),
        LockFunds(AccountId, Balance, BlockNumber),
		UnlockFunds(AccountId, Balance, BlockNumber),
		// sender, dest, amount, block number
		TransferFunds(AccountId, AccountId, Balance, BlockNumber),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		KittiesCountOverFlow,
		InvalidKittyId,
        RequireDifferentParent,
        RequireOwner,
		NotVerifyKittyOwner,
		NoAccountKitties,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 0]
		pub fn create(origin) {  
			let sender = ensure_signed(origin)?;
			let kitty_id = Self::next_kitty_id()?; // 取id
			let dna = Self::random_value(&sender);
            let kitty = Kitty(dna);

            Self::insert_kitty(&sender, kitty_id, kitty);
			Self::deposit_event(RawEvent::Created(sender, kitty_id));
		}

		#[weight = 0]
		pub fn transfer(origin, to: T::AccountId, kitty_id: T::KittyIndex){
            let sender = ensure_signed(origin)?;
            // bug没有验证kitty的所有者是谁，可任意转移
            let accountid = Self::kitty_owner(kitty_id).ok_or(Error::<T>::InvalidKittyId)?; 
            ensure!(accountid != sender.clone(), Error::<T>::RequireDifferentParent);

            ensure!(<OwnerKitties<T>>::contains_key((&sender, Some(kitty_id))), Error::<T>::RequireOwner);

            Self::transfer_owner_kitty(&sender, &to, kitty_id);
            Self::deposit_event(RawEvent::Transferred(sender, to, kitty_id));
		}
        
		#[weight = 0]
		pub fn breed(origin, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex){
            let sender = ensure_signed(origin)?;
            let new_kitty_id = Self::do_breed(&sender, kitty_id_1, kitty_id_2)?;
			Self::deposit_event(RawEvent::Created(sender, new_kitty_id));
        }
        
        #[weight = 10_000]
		pub fn reserve_funds(origin, amount: BalanceOf<T>) -> DispatchResult {
			let locker = ensure_signed(origin)?;

			T::Currency::reserve(&locker, amount)
					.map_err(|_| "locker can't afford to lock the amount requested")?;

			let now = <system::Module<T>>::block_number();

			Self::deposit_event(RawEvent::LockFunds(locker, amount, now));
			Ok(())
		}

		/// Unreserves the specified amount of funds from the caller
		#[weight = 10_000]
		pub fn unreserve_funds(origin, amount: BalanceOf<T>) -> DispatchResult {
			let unlocker = ensure_signed(origin)?;

			T::Currency::unreserve(&unlocker, amount);
			// ReservableCurrency::unreserve does not fail (it will lock up as much as amount)

			let now = <system::Module<T>>::block_number();

			Self::deposit_event(RawEvent::UnlockFunds(unlocker, amount, now));
			Ok(())
		}

		/// Transfers funds. Essentially a wrapper around the Currency's own transfer method
		#[weight = 10_000]
		pub fn transfer_funds(origin, dest: T::AccountId, amount: BalanceOf<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			T::Currency::transfer(&sender, &dest, amount, AllowDeath)?;

			let now = <system::Module<T>>::block_number();

			Self::deposit_event(RawEvent::TransferFunds(sender, dest, amount, now));
			Ok(())
		}

		/// Atomically unreserves funds and and transfers them.
		/// might be useful in closed economic systems
		#[weight = 10_000]
		pub fn unreserve_and_transfer(
			origin,
			to_punish: T::AccountId,
			dest: T::AccountId,
			collateral: BalanceOf<T>
		) -> DispatchResult {
			let _ = ensure_signed(origin)?; // dangerous because can be called with any signature (so dont do this in practice ever!)

						// If collateral is bigger than to_punish's reserved_balance, store what's left in overdraft.
			let overdraft = T::Currency::unreserve(&to_punish, collateral);

			T::Currency::transfer(&to_punish, &dest, collateral - overdraft, AllowDeath)?;

			let now = <system::Module<T>>::block_number();
			Self::deposit_event(RawEvent::TransferFunds(to_punish, dest, collateral - overdraft, now));

			Ok(())
		}

	}
}
fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8 {
    (selector & dna1) | (!selector & dna2)
}

impl<T: Trait> Module<T> {
    fn transfer_owner_kitty(from: &T::AccountId, to: &T::AccountId, kitty_id: T::KittyIndex)  {
		<OwnerKittiesList<T>>::remove(&from, kitty_id);
		Self::insert_owner_kitty(&to, kitty_id);
	}
    fn insert_owner_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex) {
		<OwnerKittiesList<T>>::append(owner, kitty_id);
		<KittyOwners<T>>::insert(kitty_id, owner);
	}
    fn insert_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex, kitty: Kitty) {
        //Kitties::insert(kitty_id, kitty); // 插入kitty
        //KittiesCount::put(kitty_id+1.into()); // 下一个index
        Kitties::<T>::insert(kitty_id, kitty);
		KittiesCount::<T>::put(kitty_id + 1.into());

        Self::insert_owner_kitty(owner, kitty_id);
    }
    fn next_kitty_id() -> sp_std::result::Result<T::KittyIndex, DispatchError> {
        let kitty_id = Self::kitties_count(); 
        if kitty_id == T::KittyIndex::max_value() {
            return Err(Error::<T>::KittiesCountOverFlow.into());
        }
        Ok(kitty_id)
    }

    fn random_value(sender: &T::AccountId) -> [u8; 16] {
        let payload = ( // hash data
                        T::Randomness::random_seed(),
                        &sender,
                        <frame_system::Module<T>>::extrinsic_index(),
        );
        payload.using_encoded(blake2_128) // 128 bit
    }

     fn do_breed(sender: &T::AccountId, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> sp_std::result::Result<T::KittyIndex, DispatchError> {
       
        let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
        let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;
        
        ensure!(kitty_id_1 != kitty_id_2, Error::<T>::RequireDifferentParent);
       
        let kitty_id = Self::next_kitty_id()?;

        let kitty_1_dna = kitty1.0;
        let kitty_2_dna = kitty2.0;
        
        let selector = Self::random_value(&sender); 
        let mut new_dna = [0u8; 16];
        for i in 0..kitty_1_dna.len() {
            new_dna[i] = combine_dna(kitty_1_dna[i], kitty_2_dna[i], selector[i]);
        }
        Self::insert_kitty(sender, kitty_id, Kitty(new_dna)); 
        Ok(kitty_id) 
    }
    
}
 