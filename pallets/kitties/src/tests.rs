#![cfg(test)]

use crate::{mock::*, pallet::Error,pallet::KittyCnt};
use frame_support::{assert_noop, assert_ok,assert_err};

#[test]
fn should_build_genesis_kitties() {
	new_test_ext().execute_with(|| {
		// Check we have 2 kitties, as specified
		assert_eq!(Kitties::kitty_cnt(), 2);

		// Check owners own the correct amount of kitties
		let kitties_owned_by_1 = Kitties::kitties_owned(0);
		assert_eq!(kitties_owned_by_1, Some(ALICE));

		let kitties_owned_by_2 = Kitties::kitties_owned(1);
		assert_eq!(kitties_owned_by_2, Some(BOB));

		// Check that kitties are owned correctly
		let kitty1 = Kitties::kitties(0).expect("Could have this kitty ID owned by acct 1");
		assert_eq!(kitty1.owner, ALICE);

		let kitty2 = Kitties::kitties(1).expect("Could have this kitty ID owned by acct 2");
		assert_eq!(kitty2.owner, BOB);
	});
}

#[test]
fn create_kitty_error_by_insufficient_balance() {
	new_test_ext_for_create().execute_with(|| {
		assert_noop!(
			Kitties::create_kitty(Origin::signed(COCO)),
			pallet_balances::Error::<Test>::InsufficientBalance
		);
	});
}


#[test]
fn create_kitty_error_by_kitty_cnt_overflow() {
	new_test_ext_for_create().execute_with(|| {

		KittyCnt::<Test>::put(u64::MAX);
		// assert_ok!(Kitties::create_kitty(Origin::signed(ALICE)));
		assert_noop!(
			Kitties::create_kitty(Origin::signed(ALICE)),
			Error::<Test>::KittyCntOverflow
		);
	});

}



#[test]
fn create_kitty_should_work() {
	new_test_ext_for_create().execute_with(|| {


		assert_ok!(Kitties::create_kitty(Origin::signed(ALICE)));


		System::assert_has_event(Event::Kitties(crate::Event::Created(ALICE, 0)));

		assert_eq!(Kitties::kitty_cnt(), 1);

		
		// check that account ALICE owns 1 kitty
		assert_eq!(Kitties::kitties_owned(0), Some(ALICE));

		// check that this kitty is specifically owned by account ALICE
		let kitty = Kitties::kitties(0).expect("should found the kitty");
		assert_eq!(kitty.owner, ALICE);
		assert_eq!(kitty.price, None);

		//check COCO reserved balanc after create new kitty
		assert_eq!(Pledge::get(), Balances::reserved_balance(ALICE));
	});
}

#[test]
fn transfer_kitty_error_by_kitty_not_exist() {
	new_test_ext().execute_with(|| {
		// should failed, kitty is not exist
		assert_noop!(
			Kitties::transfer(Origin::signed(ALICE), BOB, 3),
			Error::<Test>::KittyNotExist
		);

	});
}

#[test]
fn transfer_kitty_error_by_kitty_not_owner() {
	new_test_ext().execute_with(|| {
		// should failed, kitty is not exist
		assert_noop!(
			Kitties::transfer(Origin::signed(ALICE), BOB, 1),
			Error::<Test>::NotKittyOwner
		);

	});
}

#[test]
fn transfer_kitty_error_by_transfer_to_self() {
	new_test_ext().execute_with(|| {
		// should failed, can not transfer to self
		assert_noop!(
			Kitties::transfer(Origin::signed(ALICE), ALICE, 0),
			Error::<Test>::TransferToSelf
		);

	});
}




#[test]
fn transfer_kitty_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(Kitties::transfer(Origin::signed(ALICE), BOB, 0));

		System::assert_has_event(Event::Kitties(crate::Event::Transferred(ALICE, BOB, 0)));

		assert_eq!(Kitties::kitties_owned(0), Some(BOB));
		let kitty = Kitties::kitties(0).expect("should found the kitty");
		assert_eq!(kitty.owner, BOB);
	});
}


#[test]
fn sell_kitty_unit_error_by_kitty_not_exist() {
	new_test_ext().execute_with(|| {
		// should failed, kitty is not exist
		assert_noop!(
			Kitties::sell_kitty(Origin::signed(ALICE), 3, Some(500)),
			Error::<Test>::KittyNotExist
		);

	});
}

#[test]
fn sell_kitty_unit_error_by_kitty_not_owner() {
	new_test_ext().execute_with(|| {
		// should failed, kitty is not exist
		assert_noop!(
			Kitties::sell_kitty(Origin::signed(ALICE), 1, Some(500)),
			Error::<Test>::NotKittyOwner
		);

	});
}

#[test]
fn sell_kitty_should_work() {
	new_test_ext().execute_with(|| {

		assert_ok!(Kitties::sell_kitty(Origin::signed(ALICE), 0, Some(500)));

		System::assert_has_event(Event::Kitties(crate::Event::PriceSet(ALICE, 0, Some(500))));

		let kitty = Kitties::kitties(0).expect("should found the kitty");
		assert_eq!(kitty.price, Some(500));
	});
}

#[test]
fn buy_kitty_unit_error_by_buyer_is_kitty_owner() {
	new_test_ext().execute_with(|| {

		assert_noop!(
			Kitties::buy_kitty(Origin::signed(ALICE), 0, 500),
			Error::<Test>::BuyerIsKittyOwner
		);

	});
}

#[test]
fn buy_kitty_unit_error_by_kitty_not_for_sale() {
	new_test_ext().execute_with(|| {

		assert_noop!(
			Kitties::buy_kitty(Origin::signed(BOB), 0, 500),
			Error::<Test>::KittyNotForSale
		);

	});
}

#[test]
fn buy_kitty_unit_error_by_kitty_not_exist() {
	new_test_ext().execute_with(|| {
		assert_noop!(Kitties::buy_kitty(Origin::signed(BOB), 2, 500), Error::<Test>::KittyNotExist);

	});
}

#[test]
fn buy_kitty_unit_error_by_kitty_bid_price_too_low() {
	new_test_ext().execute_with(|| {

		assert_ok!(Kitties::sell_kitty(Origin::signed(ALICE), 0, Some(100)));

		assert_noop!(
			Kitties::buy_kitty(Origin::signed(BOB), 0, 1),
			Error::<Test>::KittyBidPriceTooLow
		);

	});
}


#[test]
fn buy_kitty_unit_error_by_not_enough_balance() {
	new_test_ext().execute_with(|| {

		assert_ok!(Kitties::sell_kitty(Origin::signed(ALICE), 0, Some(100)));

		assert_noop!(
			Kitties::buy_kitty(Origin::signed(BOB), 0, u64::MAX),
			Error::<Test>::NotEnoughBalance
		);


	});
}



#[test]
fn buy_kitty_should_work() {
	new_test_ext().execute_with(|| {

		// set price
		assert_ok!(Kitties::sell_kitty(Origin::signed(ALICE), 0, Some(100)));

		assert_ok!(Kitties::buy_kitty(Origin::signed(BOB), 0, 100));

		System::assert_has_event(Event::Kitties(crate::Event::Bought(BOB, ALICE, 0, 100)));

		assert_ok!(Kitties::sell_kitty(Origin::signed(BOB), 0, Some(100)));

		assert_ok!(Kitties::buy_kitty(Origin::signed(ALICE), 0, 1000));

		System::assert_has_event(Event::Kitties(crate::Event::Bought(ALICE, BOB, 0, 1000)));

		// check kitty information
		let kitty = Kitties::kitties(0).expect("should found the kitty");
		assert_eq!(kitty.owner, ALICE);
		assert_eq!(kitty.price, None);
	});
}



#[test]
fn breed_kitty_error_by_kitty_not_exist() {
	new_test_ext().execute_with(|| {

		assert_noop!(
			Kitties::breed_kitty(Origin::signed(ALICE), 0, 2),
			Error::<Test>::KittyNotExist
		);


	});
}

#[test]
fn breed_kitty_error_by_kitty_not_owner() {
	new_test_ext().execute_with(|| {

		assert_noop!(
			Kitties::breed_kitty(Origin::signed(ALICE), 0, 1),
			Error::<Test>::NotKittyOwner
		);

	});
}


#[test]
fn breed_kitty_should_work() {
	new_test_ext().execute_with(|| {

		assert_ok!(Kitties::transfer(Origin::signed(BOB), ALICE, 1));

		// should work
		assert_ok!(Kitties::breed_kitty(Origin::signed(ALICE), 0, 1));

		// Event log
		System::assert_has_event(Event::Kitties(crate::Event::BreedKitty(ALICE, 0, 1, 2)));

		// check new price of kitty
		let kitty = Kitties::kitties(2).expect("should found the kitty");
		assert_eq!(kitty.price, None);
		assert_eq!(kitty.owner, ALICE);
	});
}