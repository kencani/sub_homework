use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
#[test]
fn create_claim_should_works() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		let claim = vec![0, 1, 2, 3];
		assert_ok!(PoeModule::create_claim(Origin::signed(ALICE), claim.clone()));
		assert_eq!(
			Proofs::<Test>::get(&claim),
			Some((ALICE, frame_system::Pallet::<Test>::block_number())),
		);
	});
}

#[test]
fn create_claim_error_by_already_exist() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		let claim = vec![0, 1, 2, 3];

		assert_ok!(PoeModule::create_claim(Origin::signed(ALICE), claim.clone()));

		assert_noop!(
			PoeModule::create_claim(Origin::signed(ALICE), claim.clone()),
			Error::<Test>::ProofAlreadyExist
		);
	});
}

#[test]
fn trans_claim_should_works() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		let claim = vec![0, 1, 2, 3];
		assert_ok!(PoeModule::create_claim(Origin::signed(ALICE), claim.clone()));
		assert_ok!(PoeModule::trans_claim(Origin::signed(ALICE), claim.clone(),BOB));
		assert_eq!(
			Proofs::<Test>::get(&claim),
			Some((BOB, frame_system::Pallet::<Test>::block_number())),
		);
	});
}



#[test]
fn trans_claim_error_by_not_exist() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		let claim = vec![0, 1, 2, 3];
		let others = vec![0, 1];
		assert_ok!(PoeModule::create_claim(Origin::signed(ALICE), claim.clone()));
		assert_noop!(PoeModule::trans_claim(Origin::signed(ALICE), others.clone(),BOB),Error::<Test>::ClaimNotExist);

	});
}



#[test]
fn trans_claim_error_by_not_claim_owner() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		let claim = vec![0, 1, 2, 3];
		assert_ok!(PoeModule::create_claim(Origin::signed(ALICE), claim.clone()));
		assert_noop!(PoeModule::trans_claim(Origin::signed(BOB), claim.clone(),ALICE),Error::<Test>::NotClaimOwner);

	});
}


#[test]
fn revoked_claim_should_works() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		let claim = vec![0, 1, 2, 3];
		assert_ok!(PoeModule::create_claim(Origin::signed(ALICE), claim.clone()));
		assert_ok!(PoeModule::revoked_claim(Origin::signed(ALICE), claim.clone()));
		assert_eq!(
			Proofs::<Test>::get(&claim),
			None,
		);

	});
}


#[test]
fn revoked_claim_error_by_not_exist() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		let claim = vec![0, 1, 2, 3];
		let others = vec![0, 1];
		assert_ok!(PoeModule::create_claim(Origin::signed(ALICE), claim.clone()));
		assert_noop!(PoeModule::revoked_claim(Origin::signed(ALICE), others.clone()),Error::<Test>::ClaimNotExist);

	});
}




#[test]
fn revoked_claim_error_by_not_claim_owner() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		let claim = vec![0, 1, 2, 3];
		assert_ok!(PoeModule::create_claim(Origin::signed(ALICE), claim.clone()));
		assert_noop!(PoeModule::revoked_claim(Origin::signed(BOB), claim.clone()),Error::<Test>::NotClaimOwner);

	});
}



#[test]
fn create_claim_error_by_claim_over_length() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		let claim = vec![0, 1, 2, 3,4,5,6];
		assert_noop!(PoeModule::create_claim(Origin::signed(ALICE), claim.clone()),Error::<Test>::ClaimOverLength);

	});
}
