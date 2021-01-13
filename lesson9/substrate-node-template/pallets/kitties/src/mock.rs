use super::*;
   
    use balances;
    use sp_core::H256;
    use frame_support::{impl_outer_event,impl_outer_origin, parameter_types, weights::Weight, assert_ok, assert_noop,
                        traits::{OnFinalize, OnInitialize},
    };
     
    use sp_runtime::{
        traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,
    };
    
    //use frame_system as system;
    use frame_system::{self as system, EventRecord, Phase};
    impl_outer_origin! {
	    pub enum Origin for Test {}
    }
    mod simple_event {
        pub use crate::Event;
    }
    impl_outer_event! {
        pub enum TestEvent for Test {
            simple_event<T>,
            frame_system<T>,
            balances<T>,
        }
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
        type AccountData = balances::AccountData<u64>;
        type OnNewAccount = ();
        type OnKilledAccount = ();
        type SystemWeightInfo = ();
    }
    impl balances::Trait for Test {
        type Balance = u64;
        type MaxLocks = ();
        type Event = ();
        type DustRemoval = ();
        type ExistentialDeposit = ExistentialDeposit;
        type AccountStore = System;
        type WeightInfo = ();
    }
    
    type Randomness = pallet_randomness_collective_flip::Module<Test>;

    impl Trait for Test {
        type Event = ();
        type Randomness = Randomness;
        type KittyIndex = u32;
        type Currency = balances::Module<Self>;
        //type Currency = Module<Test>; 

    }
    //pub type Balances = balances::Module<Test>;
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

 
 

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}