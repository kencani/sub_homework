#![cfg_attr(not(feature = "std"), no_std)]

use codec::HasCompact;
use frame_support::{
	pallet_prelude::*,
	traits::{
		tokens::ExistenceRequirement, BalanceStatus, Currency, Randomness, ReservableCurrency,
	},
	transactional,
};
use frame_system::pallet_prelude::*;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.io/docs/en/knowledgebase/runtime/frame>
pub use pallet::*;
use scale_info::TypeInfo;
use sp_io::hashing::blake2_128;
use sp_std::{borrow::ToOwned, convert::From, prelude::*};

mod mock;
mod tests;

#[frame_support::pallet]
pub mod pallet {

	use super::*;

	#[cfg(feature = "std")]
	use frame_support::serde::{Deserialize, Serialize};
	use sp_runtime::traits::{AtLeast32BitUnsigned, CheckedAdd, One};

	type AccountOf<T> = <T as frame_system::Config>::AccountId;
	type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	// Struct for holding Kitty information.
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T: Config> {
		pub dna: [u8; 16], // Using 16 bytes to represent a kitty DNA
		pub price: Option<BalanceOf<T>>,
		pub gender: Gender,
		pub owner: AccountOf<T>,
		pub deposit: BalanceOf<T>,
	}

	// Set Gender type in Kitty struct.
	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum Gender {
		Male,
		Female,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The type of Randomness we want to specify for this pallet.
		type KittyRandomness: Randomness<Self::Hash, Self::BlockNumber>;


		/// The maximum amount of Kitties a single account can own.
		#[pallet::constant]
		type MaxKittyOwned: Get<u32>;

		/// 作业 2
		type KittyIndex: Parameter + Member + AtLeast32BitUnsigned + Default + Copy + HasCompact;

		/// 作业 4
		type Currency: ReservableCurrency<Self::AccountId>;

		/// 作业 4
		#[pallet::constant]
		type Pledge: Get<BalanceOf<Self>>;
	}

	// Errors.
	#[pallet::error]
	pub enum Error<T> {
		/// An account cannot own more Kitties than `MaxKittyCount`.
		ExceedMaxKittyOwned,
		/// Buyer cannot be the owner.
		BuyerIsKittyOwner,
		/// Cannot transfer a kitty to its owner.
		TransferToSelf,
		/// Handles checking whether the Kitty exists.
		KittyNotExist,
		/// Handles checking that the Kitty is owned by the account transferring, buying or setting
		/// a price for it.
		NotKittyOwner,
		/// Ensures the Kitty is for sale.
		KittyNotForSale,
		/// Ensures that the buying price is greater than the asking price.
		KittyBidPriceTooLow,
		/// Ensures that an account has enough funds to purchase a Kitty.
		NotEnoughBalance,
		/// Handles arithemtic overflow when incrementing the Kitty counter.
		KittyCntOverflow,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new Kitty was sucessfully created. \[sender, kitty_id\]
		Created(T::AccountId, T::KittyIndex),
		/// Kitty price was sucessfully set. \[sender, kitty_id, new_price\]
		PriceSet(T::AccountId, T::KittyIndex, Option<BalanceOf<T>>),
		/// A Kitty was sucessfully transferred. \[from, to, kitty_id\]
		Transferred(T::AccountId, T::AccountId, T::KittyIndex),
		/// A Kitty was sucessfully bought. \[buyer, seller, kitty_id, bid_price\]
		Bought(T::AccountId, T::AccountId, T::KittyIndex, BalanceOf<T>),
		/// A new Kitty was sucessfully breed. \[sender, kitty_one, kitty_two, new_kitty\]
		BreedKitty(T::AccountId, T::KittyIndex, T::KittyIndex, T::KittyIndex),
	}

	// Storage items.

	#[pallet::storage]
	#[pallet::getter(fn kitty_cnt)]
	/// Keeps track of the number of Kitties in existence.
	pub(super) type KittyCnt<T: Config> = StorageValue<_, T::KittyIndex, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	/// Stores a Kitty's unique traits, owner and price.
	pub(super) type Kitties<T: Config> =
		StorageMap<_, Twox64Concat, T::KittyIndex, Kitty<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties_owned)]
	/// Keeps track of what accounts own what Kitty.
	pub(super) type KittiesOwned<T: Config> =
		StorageMap<_, Twox64Concat, T::KittyIndex, T::AccountId, OptionQuery>;

	// Our pallet's genesis configuration.
	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub kitties: Vec<(T::AccountId, [u8; 16], Gender)>,
	}

	// Required to implement default for GenesisConfig.
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> GenesisConfig<T> {
			GenesisConfig { kitties: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			// When building a kitty from genesis config, we require the dna and gender to be
			// supplied.
			for (acct, dna, gender) in &self.kitties {
				let _ = <Pallet<T>>::mint(acct, Some(dna.clone()), Some(gender.clone()));
			}
		}
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new unique kitty.
		///
		/// The actual kitty creation is done in the `mint()` function.		
		#[transactional]
		#[pallet::weight(100)]
		pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let kitty_id = Self::mint(&sender, None, None)?;

			// Logging to the console
			log::info!("🎈😺 A kitty is born with ID ➡ {:?}.", kitty_id);
			// Deposit our "Created" event.
			Self::deposit_event(Event::Created(sender, kitty_id));
			Ok(())
		}

		/// 重构代码
		#[pallet::weight(100)]
		pub fn sell_kitty(
			origin: OriginFor<T>,
			kitty_id: T::KittyIndex,
			new_price: Option<BalanceOf<T>>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;
			ensure!(Self::is_kitty_owner(&kitty_id, &sender)?, Error::<T>::NotKittyOwner);
			Self::exchange(&kitty_id, &sender, None, new_price)
		}

	
		/// 重构代码
		#[transactional]
		#[pallet::weight(100)]
		pub fn buy_kitty(
			origin: OriginFor<T>,
			kitty_id: T::KittyIndex,
			bid_price: BalanceOf<T>,
		) -> DispatchResult {
			let buyer = ensure_signed(origin)?;
			ensure!(!Self::is_kitty_owner(&kitty_id, &buyer)?, Error::<T>::BuyerIsKittyOwner);
			Self::exchange(&kitty_id, &buyer, None, Some(bid_price))
		}


		/// Directly transfer a kitty to another recipient.
		///
		/// Any account that holds a kitty can send it to another Account. This will reset the
		/// asking price of the kitty, marking it not for sale.
		#[pallet::weight(100)]
		pub fn transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			kitty_id: T::KittyIndex,
		) -> DispatchResult {
			let from = ensure_signed(origin)?;
			// Verify the kitty is not transferring back to its owner.
			ensure!(&from != &to, Error::<T>::TransferToSelf);
			Self::exchange(&kitty_id, &from, Some(to), None)
		}


		/// Breed a Kitty.
		///
		/// Breed two kitties to create a new generation
		/// of Kitties.
		#[pallet::weight(100)]
		pub fn breed_kitty(
			origin: OriginFor<T>,
			kid1: T::KittyIndex,
			kid2: T::KittyIndex,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// Check: Verify `sender` owns both kitties (and both kitties exist).
			ensure!(Self::is_kitty_owner(&kid1, &sender)?, Error::<T>::NotKittyOwner);
			ensure!(Self::is_kitty_owner(&kid2, &sender)?, Error::<T>::NotKittyOwner);

			let new_dna = Self::breed_dna(&kid1, &kid2)?;
			let kitty_id = Self::mint(&sender, Some(new_dna), None)?;
			// Deposit our "Breed" event.
			Self::deposit_event(Event::BreedKitty(sender, kid1, kid2, kitty_id));
			Ok(())
		}
	}

	//** Our helper functions.**//

	impl<T: Config> Pallet<T> {
		fn gen_gender() -> Gender {
			let random = T::KittyRandomness::random(&b"gender"[..]).0;
			match random.as_ref()[0] % 2 {
				0 => Gender::Male,
				_ => Gender::Female,
			}
		}

		fn gen_dna() -> [u8; 16] {
			let payload = (
				T::KittyRandomness::random(&b"dna"[..]).0,
				<frame_system::Pallet<T>>::block_number(),
			);
			payload.using_encoded(blake2_128)
		}

		pub fn breed_dna(kid1: &T::KittyIndex, kid2: &T::KittyIndex) -> Result<[u8; 16], Error<T>> {
			let dna1 = Self::kitties(kid1).ok_or(Error::<T>::KittyNotExist)?.dna;
			let dna2 = Self::kitties(kid2).ok_or(Error::<T>::KittyNotExist)?.dna;

			let mut new_dna = Self::gen_dna();
			for i in 0..new_dna.len() {
				new_dna[i] = (new_dna[i] & dna1[i]) | (!new_dna[i] & dna2[i]);
			}
			Ok(new_dna)
		}

		// Helper to mint a Kitty.
		pub fn mint(
			owner: &T::AccountId,
			dna: Option<[u8; 16]>,
			gender: Option<Gender>,
		) -> Result<T::KittyIndex, DispatchError> {
			let deposit = T::Pledge::get();
			T::Currency::reserve(&owner, deposit)?;

			let kitty = Kitty::<T> {
				dna: dna.unwrap_or_else(Self::gen_dna),
				price: None,
				gender: gender.unwrap_or_else(Self::gen_gender),
				owner: owner.clone(),
				deposit,
			};

			// Performs this operation first as it may fail
			let kitty_id =
				KittyCnt::<T>::try_mutate(|id| -> Result<T::KittyIndex, DispatchError> {
					let current_id = *id;
					*id = id.checked_add(&One::one()).ok_or(Error::<T>::KittyCntOverflow)?;
					Ok(current_id)
				})?;

			<KittiesOwned<T>>::insert(kitty_id, owner);

			Kitties::<T>::insert(kitty_id, kitty);

			Ok(kitty_id)
		}

		pub fn is_kitty_owner(
			kitty_id: &T::KittyIndex,
			acct: &T::AccountId,
		) -> Result<bool, Error<T>> {
			match Self::kitties(kitty_id) {
				Some(kitty) => Ok(kitty.owner == *acct),
				None => Err(Error::<T>::KittyNotExist),
			}
		}

		#[transactional]
		pub fn exchange(
			kitty_id: &T::KittyIndex,
			who: &T::AccountId,
			to: Option<T::AccountId>,
			price: Option<BalanceOf<T>>,
		) -> DispatchResult {
			Kitties::<T>::try_mutate(kitty_id, |maybe| {

				let k = maybe.as_mut().ok_or(Error::<T>::KittyNotExist)?;

				if &k.owner == who {
					if let Some(new_owner) = to {

						ensure!(who != &new_owner, Error::<T>::TransferToSelf);

						T::Currency::repatriate_reserved(
							&k.owner,
							&new_owner,
							k.deposit,
							BalanceStatus::Reserved,
						)?;

						k.owner = new_owner.clone();
						k.price = None;

						<KittiesOwned<T>>::insert(kitty_id, new_owner.clone());

						Self::deposit_event(Event::Transferred(
							who.to_owned(),
							new_owner,
							kitty_id.to_owned(),
						));

						// just change the price if to is None
					} else {  

						k.price = price;
						Self::deposit_event(Event::PriceSet(
							who.to_owned(),
							kitty_id.to_owned(),
							price,
						));
					}

					Ok(())
				} else {

					let bid_price = price.ok_or(Error::<T>::NotKittyOwner)?;
					if let Some(ask_price) = k.price {
						ensure!(ask_price <= bid_price, Error::<T>::KittyBidPriceTooLow);
					} else {
						Err(Error::<T>::KittyNotForSale)?;
					}

					T::Currency::repatriate_reserved(
						&k.owner,
						who,
						k.deposit,
						BalanceStatus::Reserved,
					)?;

					ensure!(
						T::Currency::free_balance(who) >= bid_price,
						Error::<T>::NotEnoughBalance
					);

					let seller = k.owner.clone();

					T::Currency::transfer(
						who,
						&seller,
						bid_price,
						ExistenceRequirement::KeepAlive,
					)?;

					k.owner = who.to_owned();
					k.price = None;

					KittiesOwned::<T>::insert(kitty_id, who.to_owned());

					Self::deposit_event(Event::Bought(
						who.to_owned(),
						seller,
						kitty_id.to_owned(),
						bid_price,
					));

					Ok(())
				}
			})
		}
	}
}