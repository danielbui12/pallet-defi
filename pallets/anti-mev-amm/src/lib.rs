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
    pallet_prelude::*,
    sp_runtime::{
        traits::{
            AccountIdConversion, CheckedAdd, CheckedMul, CheckedSub, Convert, One, Saturating,
            Zero,
        },
        FixedPointNumber, FixedPointOperand, FixedU128,
    },
    traits::{
        fungibles::{Create, Destroy, Inspect, Mutate},
        tokens::{Balance, Fortitude, Precision, Preservation, WithdrawConsequence},
        ExistenceRequirement, Currency,
    },
    transactional, PalletId,
};
use codec::EncodeLike;
use sp_std::{
    ops::Sub, vec, vec::Vec,
    fmt::Debug,
};

pub use pallet::*;
use types::*;

pub use weights::WeightInfo;

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
pub type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
pub type AssetIdOf<T> = <T as Config>::AssetId;
pub type AssetBalanceOf<T> = <T as Config>::AssetBalance;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use super::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
        /// Pallet ID.
        #[pallet::constant]
        type PalletId: Get<PalletId>;

        /// The overarching runtime event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

        /// The currency trait.
        type Currency: Currency<Self::AccountId>;

        /// The balance type for assets (i.e. tokens).
        type AssetBalance: Balance
            + FixedPointOperand
            + MaxEncodedLen
            + MaybeSerializeDeserialize
            + TypeInfo;

        // Two-way conversion between asset and currency balances
        type AssetToCurrencyBalance: Convert<Self::AssetBalance, BalanceOf<Self>>;
        type CurrencyToAssetBalance: Convert<BalanceOf<Self>, Self::AssetBalance>;

        /// The asset ID type.
        type AssetId: MaybeSerializeDeserialize
            + MaxEncodedLen
            + TypeInfo
            + Clone
            + Debug
            + PartialEq
            + EncodeLike
            + Decode;
        
        /// The type for tradable assets.
        type Assets: Inspect<Self::AccountId, AssetId = Self::AssetId, Balance = Self::AssetBalance>
            + Mutate<Self::AccountId>;

        /// The type for liquidity tokens.
        type AssetRegistry: Inspect<Self::AccountId, AssetId = Self::AssetId, Balance = Self::AssetBalance>
            + Mutate<Self::AccountId>
            + Create<Self::AccountId>
            + Destroy<Self::AccountId>;

        /// Provider fee numerator.
        #[pallet::constant]
        type ProviderFeeNumerator: Get<BalanceOf<Self>>;

        /// Provider fee denominator.
        #[pallet::constant]
        type ProviderFeeDenominator: Get<BalanceOf<Self>>;

        /// Minimum initial currency deposit amount
        #[pallet::constant]
        type MinInitialCurrency: Get<BalanceOf<Self>>;

        /// Minimum initial token deposit amount
        #[pallet::constant]
        type MinInitialToken: Get<BalanceOf<Self>>;
	}

    pub trait ConfigHelper: Config {
        fn pallet_account() -> AccountIdOf<Self>;
        fn currency_to_asset(currency_balance: BalanceOf<Self>) -> AssetBalanceOf<Self>;
        fn asset_to_currency(asset_balance: AssetBalanceOf<Self>) -> BalanceOf<Self>;
        fn net_amount_numerator() -> BalanceOf<Self>;
    }

    impl<T: Config> ConfigHelper for T {
        #[inline(always)]
        fn pallet_account() -> AccountIdOf<Self> {
            Self::PalletId::get().into_account_truncating()
        }

        #[inline(always)]
        fn currency_to_asset(currency_balance: BalanceOf<Self>) -> AssetBalanceOf<Self> {
            Self::CurrencyToAssetBalance::convert(currency_balance)
        }

        #[inline(always)]
        fn asset_to_currency(asset_balance: AssetBalanceOf<Self>) -> BalanceOf<Self> {
            Self::AssetToCurrencyBalance::convert(asset_balance)
        }

        #[inline(always)]
        fn net_amount_numerator() -> BalanceOf<Self> {
            Self::ProviderFeeDenominator::get()
                .checked_sub(&Self::ProviderFeeNumerator::get())
                .expect("Provider fee shouldn't be greater than 100%")
        }
    }

    type GenesisPairInfo<T> = (
        // provider
        AccountIdOf<T>,
        // asset_id
        AssetIdOf<T>,
        // liquidity token
        AssetIdOf<T>,
        // currency balance
        BalanceOf<T>,
        // asset balance
        AssetBalanceOf<T>
    );

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub pairs: Vec<GenesisPairInfo<T>>,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> GenesisConfig<T> {
            GenesisConfig { pairs: vec![] }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            let pallet_account = T::pallet_account();
            for (provider, asset_id, liquidity_token_id, currency_amount, token_amount) in
                &self.pairs
            {
                assert!(!<Pairs<T>>::contains_key(asset_id), "Existed pair");
                assert!(
                    T::AssetRegistry::create(
                        liquidity_token_id.clone(),
                        pallet_account.clone(),
                        false,
                        <AssetBalanceOf<T>>::one(),
                    )
                    .is_ok(),
                    "Liquidity token id already in use"
                );
                // assert!(*currency_amount >= Self::MinInitialCurrency::get().into(), "Initial currency amount is less than the minimum required");
                // assert!(*token_amount >= Self::MinInitialToken::get().into(), "Initial token amount is less than the minimum required");

                let mut pair = Pair {
                    asset_id: asset_id.clone(),
                    currency_reserve: <BalanceOf<T>>::zero(),
                    token_reserve: <AssetBalanceOf<T>>::zero(),
                    liquidity_token_id: liquidity_token_id.clone(),
                };
                let liquidity_minted = T::currency_to_asset(*currency_amount);

                // Transfer the initial liquidity to the pair
                assert!(
                    <T as pallet::Config>::Currency::transfer(
                        provider,
                        &pallet_account,
                        *currency_amount,
                        ExistenceRequirement::KeepAlive,
                    )
                    .is_ok(),
                    "Provider does not have enough amount of currency"
                );

                assert!(
                    T::Assets::transfer(
                        asset_id.clone(),
                        provider,
                        &pallet_account,
                        *token_amount,
                        Preservation::Preserve,
                    )
                    .is_ok(),
                    "Provider does not have enough amount of asset tokens"
                );

                // Mint liquidity tokens for the provider
                assert!(
                    T::AssetRegistry::mint_into(
                        liquidity_token_id.clone(),
                        provider,
                        liquidity_minted
                    )
                    .is_ok(),
                    "Unexpected error while minting liquidity tokens for Provider"
                );

                // Balances update --------------------------
                pair
                    .currency_reserve
                    .saturating_accrue(*currency_amount);
                pair.token_reserve.saturating_accrue(*token_amount);
                <Pairs<T>>::insert(asset_id.clone(), pair);
            }
        }
    }

    /// The pair storage.
    /// It maps asset id to the pair.
    #[pallet::storage]
    #[pallet::getter(fn pairs)]
    pub(super) type Pairs<T: Config> =
        StorageMap<_, Twox64Concat, AssetIdOf<T>, PairOf<T>, OptionQuery>;

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

    impl<T: Config> Pallet<T> {
        pub fn inner_do_something(origin: OriginFor<T>) -> DispatchResult {
            Ok(())
        }
    }
}
