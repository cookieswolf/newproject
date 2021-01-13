use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
use frame_system::{EventRecord, Phase, RawOrigin};

//创建kitty成功
#[test]
fn owned_kitties_can_append_values() {
     
    new_test_ext().execute_with(|| {
        run_to_block(10);
        assert_eq!(Kitties::create(Origin::signed(1)), Ok(()));

        
        assert_eq!(
            System::events(),
            vec![EventRecord {
            phase:Phase::Initialization,
            event:TestEvent::simple_event(Event:<Test>::Created(1u64,0)),
            topics:vec![],
            }]
        );
    })
}

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
        assert_eq!(
                System::events(),
                vec![EventRecord {
                phase:Phase::Initialization,
                event:TestEvent::simple_event(Event:<Test>::Transferred(1u64,2,id-1)),
                topics:vec![],
                }]
        );
    })
}

 