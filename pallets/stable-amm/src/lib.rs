#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod mock;

pub mod types;
pub mod weights;

use frame_support::{
	dispatch::DispatchResult,
	pallet_prelude::*,
	traits::UnixTime,
	transactional, PalletId,
};

use sp_runtime::traits::{AccountIdConversion, StaticLookup};
use sp_std::{ops::Sub, vec, vec::Vec};

pub use pallet::*;
use types::*;

pub use weights::WeightInfo;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use super::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
        /// The overarching runtime event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// This pallet ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {

	}

	#[pallet::error]
	pub enum Error<T> {
	}

    /// 
    /// About transactional: https://paritytech.github.io/polkadot-sdk/master/frame_support/storage/transactional/index.html
    ///
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::default())]
		#[transactional]
		pub fn create_base_pool(
			origin: OriginFor<T>
        ) -> DispatchResult {
            Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::default())]
		#[transactional]
		pub fn create_meta_pool(
			origin: OriginFor<T>
		) -> DispatchResult {
            Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::default())]
		#[transactional]
		pub fn add_liquidity(
			origin: OriginFor<T>,
		) -> DispatchResult {
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::default())]
		#[transactional]
		pub fn swap(
			origin: OriginFor<T>,
		) -> DispatchResult {
			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::default())]
		#[transactional]
		pub fn remove_liquidity(
			origin: OriginFor<T>
		) -> DispatchResult {
			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::default())]
		#[transactional]
		pub fn remove_liquidity_one_currency(
			origin: OriginFor<T>,
		) -> DispatchResult {
			Ok(())
		}

		#[pallet::call_index(6)]
		#[pallet::weight(T::WeightInfo::default())]
		#[transactional]
		pub fn remove_liquidity_imbalance(
			origin: OriginFor<T>,
		) -> DispatchResult {
			Ok(())
		}

		#[pallet::call_index(7)]
		#[pallet::weight(T::WeightInfo::default())]
		#[transactional]
		pub fn add_pool_and_base_pool_liquidity(
			origin: OriginFor<T>,
		) -> DispatchResult {
			Ok(())
		}

		#[pallet::call_index(8)]
		#[pallet::weight(T::WeightInfo::default())]
		#[transactional]
		pub fn remove_pool_and_base_pool_liquidity(
			origin: OriginFor<T>,
		) -> DispatchResult {
			Ok(())
		}

		#[pallet::call_index(9)]
		#[pallet::weight(T::WeightInfo::default())]
		#[transactional]
		pub fn remove_pool_and_base_pool_liquidity_one_currency(
			origin: OriginFor<T>,
		) -> DispatchResult {
			Ok(())
		}

		#[pallet::call_index(10)]
		#[pallet::weight(T::WeightInfo::default())]
		#[transactional]
		pub fn swap_pool_from_base(
			origin: OriginFor<T>,
		) -> DispatchResult {
			Ok(())
		}

        #[pallet::call_index(11)]
		#[pallet::weight(T::WeightInfo::default())]
		#[transactional]
		pub fn swap_pool_to_base(
			origin: OriginFor<T>,
		) -> DispatchResult {
			Ok(())
		}

		#[pallet::call_index(12)]
		#[pallet::weight(T::WeightInfo::default())]
		#[transactional]
		pub fn swap_meta_pool_underlying(
			origin: OriginFor<T>,
		) -> DispatchResult {
			Ok(())
		}

        #[pallet::call_index(13)]
		#[pallet::weight(T::WeightInfo::default())]
		#[transactional]
		pub fn update_fee_receiver(
			origin: OriginFor<T>,
		) -> DispatchResult {
            Ok(())
		}

		#[pallet::call_index(14)]
		#[pallet::weight(T::WeightInfo::default())]
		#[transactional]
		pub fn set_swap_fee(
			origin: OriginFor<T>,
		) -> DispatchResult {
	        Ok(())
        }
		#[pallet::call_index(15)]
		#[pallet::weight(T::WeightInfo::default())]
		#[transactional]
		pub fn set_admin_fee(
			origin: OriginFor<T>,
		) -> DispatchResult {
            Ok(())
		}

        #[pallet::call_index(18)]
		#[pallet::weight(T::WeightInfo::default())]
		#[transactional]
		pub fn withdraw_admin_fee(origin: OriginFor<T>) -> DispatchResult {
            Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
}
