use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

// Test accounts used
pub const ALICE: <Test as frame_system::Config>::AccountId = 0;

/// Genesis tests
#[test]
fn genesis_config() {
    new_test_ext(false).execute_with(||{
        assert!(Artists::get_artist(0) == 
            Some(ArtistInfos {
                id: 0,
                account: ALICE,
                name: b"Genesis Artist".to_vec().try_into().unwrap(),
                age: 0, // Genesis block is 0
            }
        ));
    });
}

#[test]
fn create_artist_root() {
    new_test_ext(true).execute_with(||{
        assert_ok!(Artists::force_create(
            Origin::root(),
            1,
            ALICE,
            b"Test Artist".to_vec(),
            b"Test Artist Asset".to_vec(),
            b"TAA".to_vec(),
        ));
        assert_noop!(Artists::force_create(
            Origin::root(),
            1,
            ALICE,
            b"Test Artist 2".to_vec(),
            b"Test Artist Asset 2".to_vec(),
            b"TAA2".to_vec(),
        ), Error::<Test>::AlreadyExist);
    });
}