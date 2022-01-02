#![cfg_attr(not(feature = "std"), no_std)]

/// Homeworks of lesson one
/// <https://docs.substrate.io/v3/runtime/frame>
///
pub use pallet::*;
use sp_std::prelude::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*, ensure};
	use frame_system::pallet_prelude::*;
	use sp_std::prelude::*;
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		#[pallet::constant]
		type Length: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;


	#[pallet::storage]
	#[pallet::getter(fn proofs)]
	pub(super) type Proofs<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		Vec<u8>,
		(T::AccountId,T::BlockNumber),
		OptionQuery,
	>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// ClaimCreate
		/// - id T::AccountId 来源
		/// - claim Vec<u8> 存证明
		ClaimCreate( T::AccountId,Vec<u8>),


		/// ClaimTrans
		/// - id T::AccountId 来源
		/// - dest T::AccountId 转义目标
		/// - claim Vec<u8> 存证明
		ClaimTrans( T::AccountId,T::AccountId,Vec<u8>),

		/// ClaimRevoked
		/// - id T::AccountId 来源
		/// - claim Vec<u8> 存证明
		ClaimRevoked( T::AccountId,Vec<u8>),

	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {

		ProofAlreadyExist,

		ClaimNotExist,

		NotClaimOwner,

		ClaimOverLength,
	}


	#[pallet::call]
	impl<T: Config> Pallet<T> {

		/// storage and claim.
		#[pallet::weight(0)]
		pub fn create_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResultWithPostInfo {
			let max_length = T::Length::get();

			ensure!(claim.len()  <= max_length as usize , Error::<T>::ClaimOverLength);

			let who = ensure_signed(origin)?;
			//check proofs if it's exist.
			ensure!(!Proofs::<T>::contains_key(&claim),Error::<T>::ProofAlreadyExist);

			//insert the claim
			Proofs::<T>::insert(&claim,(who.clone(),frame_system::Pallet::<T>::block_number()));

			Self::deposit_event(Event::ClaimCreate(who, claim));

			Ok(().into())
		}



		#[pallet::weight(0)]
		pub fn trans_claim(origin: OriginFor<T>, claim: Vec<u8> ,dest: T::AccountId) -> DispatchResultWithPostInfo {

			let who = ensure_signed(origin)?;

			//check proofs if it's exist.
			let (owner,_) = Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?;

			//check owner
			ensure!(who == owner,Error::<T>::NotClaimOwner);

			//insert the claim
			Proofs::<T>::insert(&claim,(dest.clone(),frame_system::Pallet::<T>::block_number()));

			Self::deposit_event(Event::ClaimTrans(who,dest, claim));

			Ok(().into())
		}


		#[pallet::weight(0)]
		pub fn revoked_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResultWithPostInfo {

			let who = ensure_signed(origin)?;

			//check proofs if it's exist.
			let (owner,_) = Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?;

			//check owner
			ensure!(who == owner,Error::<T>::NotClaimOwner);

			//remove the key name claim
			Proofs::<T>::remove(&claim);

			Ok(().into())
		}


	}
}
