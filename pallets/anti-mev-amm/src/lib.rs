#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod mock;

pub mod constant_product;
pub mod anti_mev;
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


/// The log target of this pallet.
pub const LOG_TARGET: &str = "[💳 Anti MEV AMM]";

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
        type MinInitialToken: Get<AssetBalanceOf<Self>>;

        #[pallet::constant]
        type Fragment: Get<u32>;
        
        /// Maximum queue amount
        type MinQueueAmount: Get<u32>;
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
            // net = denominator - numerator
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
                assert!(*currency_amount >=  T::MinInitialCurrency::get(), "Initial currency amount is less than the minimum required");
                assert!(*token_amount >=  T::MinInitialToken::get(), "Initial token amount is less than the minimum required");

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

    /// The pair storage. maps asset id to the pair.
    #[pallet::storage]
    #[pallet::getter(fn pairs)]
    pub(super) type Pairs<T: Config> =
        StorageMap<_, Twox64Concat, AssetIdOf<T>, PairOf<T>, OptionQuery>;

    /// The queue for currency to asset swaps.
    /// Maps asset id to account.
    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn currency_to_asset_queue)]
    pub(super) type CurrencyToAssetQueue<T: Config> =
        StorageMap<_, Twox64Concat, AssetIdOf<T>, Vec<T::AccountId>, OptionQuery>;

    /// The queue for asset to currency swaps.
    /// Maps asset id to account
    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn asset_to_currency_queue)]
    pub(super) type AssetToCurrencyQueue<T: Config> =
        StorageMap<_, Twox64Concat, AssetIdOf<T>, Vec<T::AccountId>, OptionQuery>;


    /// The cumulative for currency
    /// Maps asset id to (maps account to amount)
    #[pallet::storage]
    #[pallet::getter(fn currency_cumulative)]
    pub(super) type CurrencyToAssetCumulative<T: Config> =
        StorageDoubleMap<
            _,
            Twox64Concat,
            AssetIdOf<T>,
            Twox64Concat,
            AccountIdOf<T>,
            BalanceOf<T>,
            OptionQuery
        >;

    /// The cumulative for asset
    /// Maps asset id to (maps account to amount)
    #[pallet::storage]
    #[pallet::getter(fn asset_cumulative)]
    pub(super) type AssetToCurrencyCumulative<T: Config> =
        StorageDoubleMap<
            _,
            Twox64Concat,
            AssetIdOf<T>,
            Twox64Concat,
            AccountIdOf<T>,
            AssetBalanceOf<T>,
            OptionQuery
        >;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
        /// A new pair was created (asset_id, liquidity_token_id)
        PairCreated(AssetIdOf<T>, AssetIdOf<T>),
        /// Liquidity was added to a pair (provider_id, asset_id, currency_amount, token_amount, liquidity_minted)
        LiquidityAdded(
            T::AccountId,
            AssetIdOf<T>,
            BalanceOf<T>,
            AssetBalanceOf<T>,
            AssetBalanceOf<T>,
        ),
        /// Liquidity was removed from a pair (provider_id, asset_id, currency_amount, token_amount, liquidity_amount)
        LiquidityRemoved(
            T::AccountId,
            AssetIdOf<T>,
            BalanceOf<T>,
            AssetBalanceOf<T>,
            AssetBalanceOf<T>,
        ),
        /// Currency was traded for an asset (asset_id, buyer_id, currency_amount, token_amount)
        SwappedCurrencyForAsset(
            AssetIdOf<T>,
            T::AccountId,
            BalanceOf<T>,
            AssetBalanceOf<T>,
        ),
        /// An asset was traded for currency (asset_id, buyer_id, currency_amount, token_amount)
        SwappedAssetForCurrency(
            AssetIdOf<T>,
            T::AccountId,
            BalanceOf<T>,
            AssetBalanceOf<T>,
        ),
        /// Add swap currency for asset tx to queue (asset_id, buyer_id, amount_in)
        AddedSwapCurrencyForAsset(
            AssetIdOf<T>,
            T::AccountId,
            BalanceOf<T>,
        ),
        /// Add swap asset for currency tx to queue (asset_id, buyer_id, amount_in)
        AddedSwapAssetForCurrency(
            AssetIdOf<T>,
            T::AccountId,
            AssetBalanceOf<T>,
        ),
        /// Settlement performed (asset_id, currency_out, asset_out)
        DistributeSettlement(
            AssetIdOf<T>,
            BalanceOf<T>,
            AssetBalanceOf<T>,
        ),
	}

	#[pallet::error]
	pub enum Error<T> {
         /// Asset with the specified ID does not exist
        AssetNotFound,
        /// Pair already exists
        PairAlreadyExists,
        /// Provided liquidity token ID is already in use
        TokenIdAlreadyInUse,
        /// Not enough free balance to add liquidity or perform trade
        BalanceTooLow,
        /// Not enough tokens to add liquidity or perform trade
        NotEnoughTokens,
        /// Specified account doesn't own enough liquidity in the pair
        ProviderLiquidityTooLow,
        /// No pair found for the given `asset_id`
        PairNotFound,
        /// Zero value provided for trade amount parameter
        TradeAmountIsZero,
        /// Zero value provided for `max_tokens` parameter
        MaxTokensIsZero,
        /// Zero value provided for `currency_amount` parameter
        CurrencyAmountIsZero,
        /// Value provided for `token_amount` parameter is too low
        TokenAmountTooLow,
        /// Value provided for `currency_amount` parameter is too high
        CurrencyAmountTooHigh,
        /// Value provided for `currency_amount` parameter is too low
        CurrencyAmountTooLow,
        /// Zero value provided for `min_liquidity` parameter
        MinLiquidityIsZero,
        /// Value provided for `max_tokens` parameter is too low
        MaxTokensTooLow,
        /// Value provided for `min_liquidity` parameter is too high
        MinLiquidityTooHigh,
        /// Zero value provided for `liquidity_amount` parameter
        LiquidityAmountIsZero,
        /// Zero value provided for `min_currency` parameter
        MinCurrencyIsZero,
        /// Zero value provided for `min_tokens` parameter
        MinTokensIsZero,
        /// Slippage exceeded
        SlippageExceeded,
        /// Value provided for `max_currency` parameter is too low
        MaxCurrencyTooLow,
        /// Value provided for `min_bought_tokens` parameter is too high
        MinBoughtTokensTooHigh,
        /// Value provided for `max_sold_tokens` parameter is too low
        MaxSoldTokensTooLow,
        /// There is not enough liquidity in the pair to perform trade
        OverLiquidityBalance,
        /// Overflow occurred
        Overflow,
        /// Underflow occurred
        Underflow,
        /// Deadline specified for the operation has passed
        DeadlinePassed,
        /// Queue too small
        QueueTooSmall,
        /// Currency overflow
        CurrencyOverflow,
        /// Asset overflow
        AssetOverflow,
        /// Currency leak
        CurrencyLeak,
        /// Asset leak
        AssetLeak,
	}

   #[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::default())]
		#[transactional]
		pub fn create_pair(
			origin: OriginFor<T>,
            asset_id: AssetIdOf<T>,
            liquidity_token_id: AssetIdOf<T>,
            currency_amount: BalanceOf<T>,
            token_amount: AssetBalanceOf<T>,
        ) -> DispatchResult {
            let caller = ensure_signed(origin)?;
            // validate the input
            ensure!(currency_amount >= T::MinInitialCurrency::get(), Error::<T>::CurrencyAmountTooLow);
            ensure!(token_amount >= T::MinInitialToken::get(), Error::<T>::TokenAmountTooLow);
            if T::Assets::total_issuance(asset_id.clone()).is_zero() {
                Err(Error::<T>::AssetNotFound)?
            }
            if <Pairs<T>>::contains_key(asset_id.clone()) {
                Err(Error::<T>::PairAlreadyExists)?
            }

            // register the liquidity token
            T::AssetRegistry::create(
                liquidity_token_id.clone(),
                T::pallet_account(),
                false,
                <AssetBalanceOf<T>>::one(),
            )
            .map_err(|_| Error::<T>::TokenIdAlreadyInUse)?;

            // create the pair
            let pair = Pair {
                asset_id: asset_id.clone(),
                currency_reserve: <BalanceOf<T>>::zero(),
                token_reserve: <AssetBalanceOf<T>>::zero(),
                liquidity_token_id: liquidity_token_id.clone(),
            };
            let liquidity_minted = T::currency_to_asset(currency_amount);
            Self::inner_add_liquidity(
                pair,
                currency_amount,
                token_amount,
                liquidity_minted,
                caller,
            )?;

            // create default queue
            <CurrencyToAssetQueue<T>>::insert(asset_id.clone(), Vec::<T::AccountId>::new());
            <AssetToCurrencyQueue<T>>::insert(asset_id.clone(), Vec::<T::AccountId>::new());

            Ok(())
		}

        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::default())]
        pub fn add_liquidity(
            origin: OriginFor<T>,
            asset_id: AssetIdOf<T>,
            currency_amount: BalanceOf<T>,
            min_liquidity: AssetBalanceOf<T>,
            max_tokens: AssetBalanceOf<T>,
            deadline: BlockNumberFor<T>,
        ) -> DispatchResult {
            // validate the input
            let caller = ensure_signed(origin)?;
            Self::check_deadline(&deadline)?;
            ensure!(currency_amount > Zero::zero(), Error::<T>::CurrencyAmountIsZero);
            ensure!(max_tokens > Zero::zero(), Error::<T>::MaxTokensIsZero);
            ensure!(min_liquidity > Zero::zero(), Error::<T>::MinLiquidityIsZero);
            Self::check_enough_currency(&caller, &currency_amount)?;
            Self::check_enough_tokens(&asset_id, &caller, &max_tokens)?;
            let pair = Self::get_pair(&asset_id)?;

            // compute the amount of tokens to mint
            let total_liquidity = T::Assets::total_issuance(pair.liquidity_token_id.clone());
            debug_assert!(total_liquidity > Zero::zero());
            let currency_amount = T::currency_to_asset(currency_amount);
            let currency_reserve = T::currency_to_asset(pair.currency_reserve);
            let token_amount =
                FixedU128::saturating_from_rational(currency_amount, currency_reserve)
                    .saturating_mul_int(pair.token_reserve)
                    .saturating_add(One::one());
            let liquidity_minted =
                FixedU128::saturating_from_rational(currency_amount, currency_reserve)
                    .saturating_mul_int(total_liquidity);
            ensure!(token_amount <= max_tokens, Error::<T>::MaxTokensTooLow);
            ensure!(liquidity_minted >= min_liquidity, Error::<T>::MinLiquidityTooHigh);

            // perform the operation
            Self::inner_add_liquidity(
                pair,
                T::asset_to_currency(currency_amount),
                token_amount,
                liquidity_minted,
                caller,
            )
        }

        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::default())]
        pub fn cp_swap_currency_for_asset(
            origin: OriginFor<T>,
            asset_id: AssetIdOf<T>,
            swap: CpSwap<BalanceOf<T>, AssetBalanceOf<T>>,
            deadline: BlockNumberFor<T>,
        ) -> DispatchResult {
            // validate the input
            let caller = ensure_signed(origin)?;
            Self::check_deadline(&deadline)?;
            Self::cp_check_trade_amount(&swap)?;
            let pair = Self::get_pair(&asset_id)?;

            // compute price
            let (currency_amount, token_amount) =
                Self::cp_compute_currency_to_asset(&pair, swap)?;
            Self::check_enough_currency(&caller, &currency_amount)?;

            // perform the trade
            Self::do_cp_swap_currency_for_asset(
                pair,
                currency_amount,
                token_amount,
                caller,
            )
        }

        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::default())]
        pub fn add_swap_currency_for_asset(
            origin: OriginFor<T>,
            asset_id: AssetIdOf<T>,
            amount_in: BalanceOf<T>,
            deadline: BlockNumberFor<T>,
        ) -> DispatchResult {
            // validate the input
            let caller = ensure_signed(origin)?;
            Self::check_deadline(&deadline)?;
            ensure!(!amount_in.is_zero(), Error::<T>::TradeAmountIsZero);
            Self::check_enough_currency(&caller, &amount_in)?;

            let pair = Self::get_pair(&asset_id)?;
            let pair_currency_cumulative = Self::get_pair_currency_cumulative(&asset_id, &caller);
            let pair_currency_queue = Self::get_pair_currency_queue(&asset_id)?;


            log::debug!(
				target: LOG_TARGET,
				"Current cumulative asset {:?} of {:?} is {:?}",
				asset_id,
				caller,
                pair_currency_cumulative
			);

            // pre compute to make sure the trade is possible
            Self::cp_get_output_amount(
                &amount_in,
                &pair.currency_reserve,
                &T::asset_to_currency(pair.token_reserve),
            )?;

            // transfer to pallet account
            let asset_id = pair.asset_id.clone();
            let pallet_account = T::pallet_account();
            <T as pallet::Config>::Currency::transfer(
                &caller,
                &pallet_account,
                amount_in.clone(),
                ExistenceRequirement::KeepAlive,
            )?;

            // add tx to queue
            Self::add_currency_to_asset_tx(
                asset_id,
                amount_in,
                caller,
                pair_currency_cumulative,
                pair_currency_queue,
            )?;


            Ok(())
        }

        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::default())]
        pub fn add_swap_asset_for_currency(
            origin: OriginFor<T>,
            asset_id: AssetIdOf<T>,
            amount_in: AssetBalanceOf<T>,
            deadline: BlockNumberFor<T>,
        ) -> DispatchResult {
            // validate the input
            let caller = ensure_signed(origin)?;
            Self::check_deadline(&deadline)?;
            ensure!(!amount_in.is_zero(), Error::<T>::TradeAmountIsZero);
            Self::check_enough_tokens(&asset_id, &caller, &amount_in)?;

            let pair = Self::get_pair(&asset_id)?;
            let pair_asset_cumulative = Self::get_pair_asset_cumulative(&asset_id, &caller);
            let pair_asset_queue = Self::get_pair_asset_queue(&asset_id)?;

            log::debug!(
				target: LOG_TARGET,
				"Current cumulative asset {:?} of {:?} is {:?}",
				asset_id,
				caller,
                pair_asset_cumulative
			);

            // pre compute to make sure the trade is possible
            Self::cp_get_output_amount(
                &T::asset_to_currency(amount_in),
                &T::asset_to_currency(pair.token_reserve),
                &pair.currency_reserve,
            )?;

            // transfer to pallet account
            let asset_id = pair.asset_id.clone();
            let pallet_account = T::pallet_account();
            T::Assets::transfer(
                asset_id.clone(),
                &caller,
                &pallet_account,
                amount_in.clone(),
                Preservation::Preserve,
            )?;

            // add tx to queue
            Self::add_asset_to_currency_tx(
                asset_id,
                amount_in,
                caller,
                pair_asset_cumulative,
                pair_asset_queue,
            )?;


            Ok(())
        }


        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::default())]
        pub fn settle_and_distribute(
            origin: OriginFor<T>,
            asset_id: AssetIdOf<T>,
        ) -> DispatchResult {
            let _executor = ensure_signed(origin)?;
            let currency_queue = Self::get_pair_currency_queue(&asset_id)?;
            let asset_queue = Self::get_pair_asset_queue(&asset_id)?;
            ensure!(
                currency_queue.len() >= T::MinQueueAmount::get() as usize,
                Error::<T>::QueueTooSmall
            );
            ensure!(
                asset_queue.len() >= T::MinQueueAmount::get() as usize,
                Error::<T>::QueueTooSmall
            );
            
            // Sum of all currency and asset in queue
            let total_cumulative_currency: BalanceOf<T> = Self::calculate_cumulative_currency(&asset_id, &currency_queue);
            let total_cumulative_asset: BalanceOf<T> = T::asset_to_currency(
                Self::calculate_cumulative_asset(&asset_id, &asset_queue)
            );
            
            let modified_cumulative_currency =
                total_cumulative_currency.clone() * T::ProviderFeeNumerator::get().into() / T::ProviderFeeDenominator::get().into();
            let modified_cumulative_asset = 
                total_cumulative_asset.clone() * T::ProviderFeeNumerator::get().into() / T::ProviderFeeDenominator::get().into();

            // Temporary reserves to save gas
            let pair = Self::get_pair(&asset_id)?;
            let mut temporary_currency_reserve = pair.currency_reserve;
            let mut temporary_asset_reserve = T::asset_to_currency(pair.token_reserve);
            let constant_product = temporary_currency_reserve.clone() * temporary_asset_reserve.clone();


            let fragment = T::Fragment::get();
            for _i in 1..=fragment {
                // Base currency increase and quote currency decrease
                temporary_currency_reserve += modified_cumulative_currency / fragment.into();
                temporary_asset_reserve = constant_product / temporary_currency_reserve;

                // Quote currency increase and base currency decrease
                temporary_asset_reserve += modified_cumulative_asset / fragment.into();
                temporary_currency_reserve = constant_product / temporary_asset_reserve;

                // NOTE: This algorithm is only asymptotically unbiased,
                // because the increase of base currency always goes first.
                // If we want to make it completely unbiased, we should
                // simulate in another direction and calculate the mean value.
            }

            // Calculate the output
            if temporary_currency_reserve > pair.currency_reserve + total_cumulative_currency {
                log::error!(
                    target: LOG_TARGET,
                    "Currency overflow: {:?} > {:?} + {:?}",
                    temporary_currency_reserve,
                    pair.currency_reserve,
                    total_cumulative_currency
                );
                return Err(Error::<T>::CurrencyOverflow.into());
            }
            if temporary_asset_reserve > T::asset_to_currency(pair.token_reserve) + total_cumulative_asset {
                log::error!(
                    target: LOG_TARGET,
                    "Asset overflow: {:?} > {:?} + {:?}",
                    temporary_asset_reserve,
                    pair.token_reserve,
                    total_cumulative_asset
                );
                return Err(Error::<T>::AssetOverflow.into());
            }

            let currency_out = 
                pair.currency_reserve + total_cumulative_currency - temporary_currency_reserve;
            let asset_out = 
                T::asset_to_currency(pair.token_reserve) + total_cumulative_asset - temporary_asset_reserve;

            // Distribute
            for i in 0..currency_queue.len() {
                Self::do_anti_mev_swap_currency_for_asset(
                    &asset_id,
                    &currency_queue[i],
                    &asset_out,
                    &total_cumulative_currency
                )?;
            }

            for i in 0..asset_queue.len() {
                Self::do_anti_mev_swap_asset_for_currency(
                    &asset_id,
                    &asset_queue[i],
                    &T::currency_to_asset(currency_out),
                    &T::currency_to_asset(total_cumulative_asset),
                )?;
            }

            // Update the reserves
            let pallet_account = T::pallet_account();
            ensure!(
                <T as Config>::Currency::free_balance(&pallet_account) < temporary_currency_reserve,
                Error::<T>::CurrencyLeak
            );
            ensure!(
                T::Assets::balance(asset_id.clone(), &pallet_account) < T::currency_to_asset(temporary_asset_reserve),
                Error::<T>::AssetLeak
            );

            // Reset for the next settlement period
            for i in 0..currency_queue.len() {
                <AssetToCurrencyCumulative<T>>::remove(asset_id.clone(), currency_queue[i].clone());
            }
            for i in 0..asset_queue.len() {
                <AssetToCurrencyCumulative<T>>::remove(asset_id.clone(), asset_queue[i].clone());
            }
            <CurrencyToAssetQueue<T>>::insert(asset_id.clone(), Vec::<T::AccountId>::new());
            <AssetToCurrencyQueue<T>>::insert(asset_id.clone(), Vec::<T::AccountId>::new());

            // TODO: distribute the rewards
            // For example:
            // - transfer native currency
            // - mint/transfer reward asset

            // emit SettlementPerformed(BaseCurrencyOut, QuoteCurrencyOut, block.timestamp);
            Self::deposit_event(Event::DistributeSettlement(
                asset_id,
                currency_out,
                T::currency_to_asset(asset_out),
            ));

            Ok(())
        }
	}



    impl<T: Config> Pallet<T> {
        /// Perform currency and asset transfers, mint liquidity token,
        /// update pair balances, emit event
        #[transactional]
        fn inner_add_liquidity(
            mut pair: PairOf<T>,
            currency_amount: BalanceOf<T>,
            token_amount: AssetBalanceOf<T>,
            liquidity_minted: AssetBalanceOf<T>,
            provider: AccountIdOf<T>,
        ) -> DispatchResult {
            // transfer currency and asset tokens to liquidity
            let asset_id = pair.asset_id.clone();
            let pallet_account = T::pallet_account();
            <T as pallet::Config>::Currency::transfer(
                &provider,
                &pallet_account,
                currency_amount,
                ExistenceRequirement::KeepAlive,
            )?;
            T::Assets::transfer(
                asset_id.clone(),
                &provider,
                &pallet_account,
                token_amount,
                Preservation::Preserve,
            )?;
            // mint liquidity tokens to the provider
            T::AssetRegistry::mint_into(
                pair.liquidity_token_id.clone(),
                &provider,
                liquidity_minted,
            )?;
    
            // accrue liquidity reserves
            pair.currency_reserve.saturating_accrue(currency_amount);
            pair.token_reserve.saturating_accrue(token_amount);
            <Pairs<T>>::insert(asset_id.clone(), pair);
    
            // emit event
            Self::deposit_event(Event::LiquidityAdded(
                provider,
                asset_id,
                currency_amount,
                token_amount,
                liquidity_minted,
            ));
            Ok(())
        }

        pub(crate) fn get_pair(asset_id: &AssetIdOf<T>) -> Result<PairOf<T>, Error<T>> {
            <Pairs<T>>::get(asset_id.clone()).ok_or(Error::<T>::PairNotFound)
        }
    
        pub(crate) fn check_deadline(deadline: &BlockNumberFor<T>) -> Result<(), Error<T>> {
            ensure!(deadline >= &<frame_system::Pallet<T>>::block_number(), Error::DeadlinePassed);
            Ok(())
        }

        pub(crate) fn check_enough_currency(
            account_id: &AccountIdOf<T>,
            amount: &BalanceOf<T>,
        ) -> Result<(), Error<T>> {
            ensure!(
                &<T as Config>::Currency::free_balance(account_id) >= amount,
                Error::<T>::BalanceTooLow
            );
            Ok(())
        }
    
        pub(crate) fn check_enough_tokens(
            asset_id: &AssetIdOf<T>,
            account_id: &AccountIdOf<T>,
            amount: &AssetBalanceOf<T>,
        ) -> Result<(), Error<T>> {
            match T::Assets::can_withdraw(asset_id.clone(), account_id, *amount) {
                WithdrawConsequence::Success => Ok(()),
                WithdrawConsequence::ReducedToZero(_) => Ok(()),
                WithdrawConsequence::UnknownAsset => Err(Error::<T>::AssetNotFound),
                _ => Err(Error::<T>::NotEnoughTokens),
            }
        }

        pub(crate) fn calculate_cumulative_currency(
            asset_id: &AssetIdOf<T>,
            queue: &Vec<T::AccountId>,
        ) -> BalanceOf<T> {
            let mut total_cumulative: BalanceOf<T> = Zero::zero();
            queue.iter()
                .for_each(|account| {
                    total_cumulative += Self::get_pair_currency_cumulative(&asset_id, &account);
                });
            total_cumulative
        }

        pub(crate) fn calculate_cumulative_asset(
            asset_id: &AssetIdOf<T>,
            queue: &Vec<T::AccountId>,
        ) -> AssetBalanceOf<T> {
            let mut total_cumulative: AssetBalanceOf<T> = Zero::zero();
            queue.iter()
                .for_each(|account| {
                    total_cumulative += Self::get_pair_asset_cumulative(&asset_id, &account);
                });
            total_cumulative
        }

    }
}

