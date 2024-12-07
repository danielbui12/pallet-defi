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

   #[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::default())]
		#[transactional]
		pub fn do_something(
			origin: OriginFor<T>
        ) -> DispatchResult {
            Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
    pub fn inner_do_something(origin: OriginFor<T>) -> DispatchResult {
        Ok(())
    }
}
