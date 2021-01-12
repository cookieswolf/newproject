#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use frame_support::{decl_module, decl_storage, decl_event, decl_error, ensure,Parameter,
                    traits::{Randomness,Currency,ReservableCurrency},
};
use sp_io::hashing::blake2_128; 
use frame_system::{self as system, ensure_signed};
use sp_runtime::{DispatchError, traits::{AtLeast32Bit, Bounded, Member}};
use crate::doublelinkedlist::{LinkedList, LinkedItem};
//设计的数据结构双链表 
mod doublelinkedlist;

 

//type KittyIndex = u32;


#[derive(Encode, Decode)]
pub struct Kitty(pub [u8; 16]); 

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Randomness: Randomness<Self::Hash>;
    //KittyIndex
    type KittyIndex: Parameter + Member + AtLeast32Bit + Bounded + Default + Copy;
    //type Currency: Currency<Self::AccountId>+ReservableCurrency<Self::AccountId>;
}
//type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
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
    KittyIndex = <T as Trait>::KittyIndex
    {
		Created(AccountId, KittyIndex),
		Transferred(AccountId, AccountId, KittyIndex),
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

#[cfg(test)]
mod tests {
    use super::*;
    //use pallet_balances;
    use sp_core::H256;
    use frame_support::{impl_outer_origin, parameter_types, weights::Weight, assert_ok, assert_noop,
                        traits::{OnFinalize, OnInitialize},
    };
     
    use sp_runtime::{
        traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,
    };
    
    use frame_system as system;

    impl_outer_origin! {
	    pub enum Origin for Test {}
    }

    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const MaximumBlockWeight: Weight = 1024;
        pub const MaximumBlockLength: u32 = 2 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);

        pub const ExistentialDeposit: u64 = 1;
    }

    impl system::Trait for Test {
        type BaseCallFilter = ();
        type Origin = Origin;
        type Call = ();
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type DbWeight = ();
        type BlockExecutionWeight = ();
        type ExtrinsicBaseWeight = ();
        type MaximumExtrinsicWeight = MaximumBlockWeight;
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type Version = ();
        type PalletInfo = ();
        type AccountData = ();
        type OnNewAccount = ();
        type OnKilledAccount = ();
        type SystemWeightInfo = ();
    }
     
    type Randomness = pallet_randomness_collective_flip::Module<Test>;

    impl Trait for Test {
        type Event = ();
        type Randomness = Randomness;
        type KittyIndex = u32;
       // type Currency = pallet_balances::Module<Self>;
        //type Currency = Module<Test>; 

    }
 
    pub type Kitties = Module<Test>;
    pub type System = frame_system::Module<Test>;

    fn run_to_block(n: u64) {
        while System::block_number() < n {
            Kitties::on_finalize(System::block_number());
            System::on_finalize(System::block_number());
            System::set_block_number(System::block_number() + 1);
            System::on_initialize(System::block_number());
            Kitties::on_initialize(System::block_number());
        }
    }

    pub fn new_test_ext() -> sp_io::TestExternalities {
        system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
    }

    /// 创建kitty
    #[test]
    fn owned_kitties_can_append_values() {
        new_test_ext().execute_with(|| {
            run_to_block(10);
            assert_eq!(Kitties::create(Origin::signed(1)), Ok(()))
        })
    }

    /// transfer kitty
    #[test]
    fn transfer_kitties() {
        new_test_ext().execute_with(|| {
            run_to_block(10);
            assert_ok!(Kitties::create(Origin::signed(1)));
            let id = Kitties::kitties_count();
            assert_ok!(Kitties::transfer(Origin::signed(1), 2 , id-1));
            assert_noop!(
                Kitties::transfer(Origin::signed(1), 2, id-1),
                Error::<Test>::NotVerifyKittyOwner
                );
        })
    }
}