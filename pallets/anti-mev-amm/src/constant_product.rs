use super::*;

impl<T: Config> Pallet<T> {
    #[transactional]
    pub(crate) fn do_cp_swap_currency_for_asset(
        mut pair: PairOf<T>,
        currency_amount: BalanceOf<T>,
        token_amount: AssetBalanceOf<T>,
        buyer: AccountIdOf<T>,
    ) -> DispatchResult {
        let asset_id = pair.asset_id.clone();
        let pallet_account = T::pallet_account();
        if buyer != pallet_account {
            <T as pallet::Config>::Currency::transfer(
                &buyer,
                &pallet_account,
                currency_amount,
                ExistenceRequirement::AllowDeath,
            )?;
        }
        T::Assets::transfer(
            asset_id.clone(),
            &pallet_account,
            &buyer,
            token_amount,
            Preservation::Expendable,
        )?;

        // update pair balances
        pair.currency_reserve.saturating_accrue(currency_amount);
        pair.token_reserve.saturating_reduce(token_amount);
        <Pairs<T>>::insert(asset_id.clone(), pair);

        // emit event
        Self::deposit_event(Event::SwappedCurrencyForAsset(
            asset_id,
            buyer,
            currency_amount,
            token_amount,
        ));
        Ok(())
    }

    #[transactional]
    pub(crate) fn do_cp_swap_asset_for_currency(
        mut pair: PairOf<T>,
        currency_amount: BalanceOf<T>,
        token_amount: AssetBalanceOf<T>,
        buyer: AccountIdOf<T>,
    ) -> DispatchResult {
        let asset_id = pair.asset_id.clone();
        let pallet_account = T::pallet_account();
        if buyer != pallet_account {
            <T as pallet::Config>::Currency::transfer(
                &buyer,
                &pallet_account,
                currency_amount,
                ExistenceRequirement::AllowDeath,
            )?;
        }
        T::Assets::transfer(
            asset_id.clone(),
            &pallet_account,
            &buyer,
            token_amount,
            Preservation::Expendable,
        )?;

        // update pair balances
        pair.currency_reserve.saturating_reduce(currency_amount);
        pair.token_reserve.saturating_accrue(token_amount);
        <Pairs<T>>::insert(asset_id.clone(), pair);

        // emit event
        Self::deposit_event(Event::SwappedAssetForCurrency(
            asset_id,
            buyer,
            currency_amount,
            token_amount,
        ));
        Ok(())
    }

    pub(crate) fn cp_compute_currency_to_asset(
        pair: &PairOf<T>,
        swap: CpSwap<BalanceOf<T>, AssetBalanceOf<T>>,
    ) -> Result<(BalanceOf<T>, AssetBalanceOf<T>), Error<T>> {
        match swap {
            CpSwap::BasedInput {
                input_amount: currency_amount,
                min_output: min_tokens,
            } => {
                let token_amount = Self::cp_get_output_amount(
                    &currency_amount,
                    &pair.currency_reserve,
                    &T::asset_to_currency(pair.token_reserve),
                )?;
                let token_amount = T::currency_to_asset(token_amount);
                ensure!(token_amount >= min_tokens, Error::SlippageExceeded);
                Ok((currency_amount, token_amount))
            }
            CpSwap::BasedOutput {
                max_input: max_currency,
                output_amount: token_amount,
            } => {
                let currency_amount = Self::cp_get_input_amount(
                    &T::asset_to_currency(token_amount),
                    &pair.currency_reserve,
                    &T::asset_to_currency(pair.token_reserve),
                )?;
                ensure!(currency_amount <= max_currency, Error::SlippageExceeded);
                Ok((currency_amount, token_amount))
            }
        }
    }

    pub(crate) fn cp_get_asset_to_currency_price(
        pair: &PairOf<T>,
        swap: CpSwap<AssetBalanceOf<T>, BalanceOf<T>>,
    ) -> Result<(BalanceOf<T>, AssetBalanceOf<T>), Error<T>> {
        match swap {
            CpSwap::BasedInput {
                input_amount: token_amount,
                min_output: min_currency,
            } => {
                let currency_amount = Self::cp_get_output_amount(
                    &T::asset_to_currency(token_amount),
                    &T::asset_to_currency(pair.token_reserve),
                    &pair.currency_reserve,
                )?;
                ensure!(currency_amount >= min_currency, Error::SlippageExceeded);
                Ok((currency_amount, token_amount))
            }
            CpSwap::BasedOutput {
                max_input: max_tokens,
                output_amount: currency_amount,
            } => {
                let token_amount = Self::cp_get_input_amount(
                    &currency_amount,
                    &T::asset_to_currency(pair.token_reserve),
                    &pair.currency_reserve,
                )?;
                let token_amount = T::currency_to_asset(token_amount);
                ensure!(token_amount <= max_tokens, Error::SlippageExceeded);
                Ok((currency_amount, token_amount))
            }
        }
    }


    pub(crate) fn cp_check_trade_amount<A: Zero, B: Zero>(
        amount: &CpSwap<A, B>,
    ) -> Result<(), Error<T>> {
        match amount {
            CpSwap::BasedInput {
                input_amount,
                min_output,
            } => {
                ensure!(!input_amount.is_zero(), Error::TradeAmountIsZero);
                ensure!(!min_output.is_zero(), Error::TradeAmountIsZero);
            }
            CpSwap::BasedOutput {
                output_amount,
                max_input,
            } => {
                ensure!(!output_amount.is_zero(), Error::TradeAmountIsZero);
                ensure!(!max_input.is_zero(), Error::TradeAmountIsZero);
            }
        };
        Ok(())
    }

    /// [Normal]
    /// 
    /// y' = (x' * y) / (x + x')
    /// 
    /// [With fee]
    /// 
    /// net = denominator - numerator
    /// y' = ( (x' * net) * y ) / (x * denominator) + (x' * net)
    ///
    pub(crate) fn cp_get_output_amount(
        input_amount: &BalanceOf<T>,
        input_reserve: &BalanceOf<T>,
        output_reserve: &BalanceOf<T>,
    ) -> Result<BalanceOf<T>, Error<T>> {
        debug_assert!(!input_reserve.is_zero());
        debug_assert!(!output_reserve.is_zero());
        let input_amount_with_fee = input_amount
            .checked_mul(&T::net_amount_numerator())
            .ok_or(Error::Overflow)?;
        let numerator = input_amount_with_fee
            .checked_mul(output_reserve)
            .ok_or(Error::Overflow)?;
        let denominator = input_reserve
            .checked_mul(&T::ProviderFeeDenominator::get())
            .ok_or(Error::Overflow)?
            .checked_add(&input_amount_with_fee)
            .ok_or(Error::Overflow)?;
        Ok(numerator / denominator)
    }

    /// [Normal]
    ///
    /// x' = (x * y') / (y + y')
    /// 
    /// [With fee]
    /// 
    /// net = denominator - numerator
    /// x' = ( x * (y' * denominator) ) / ( (y - y') * net )
    pub(crate) fn cp_get_input_amount(
        output_amount: &BalanceOf<T>,
        input_reserve: &BalanceOf<T>,
        output_reserve: &BalanceOf<T>,
    ) -> Result<BalanceOf<T>, Error<T>> {
        debug_assert!(!input_reserve.is_zero());
        debug_assert!(!output_reserve.is_zero());
        ensure!(output_amount < output_reserve, Error::<T>::OverLiquidityBalance);
        let numerator = input_reserve
            .checked_mul(output_amount)
            .ok_or(Error::Overflow)?
            .checked_mul(&T::ProviderFeeDenominator::get())
            .ok_or(Error::Overflow)?;
        let denominator = output_reserve
            .saturating_sub(*output_amount)
            .checked_mul(&T::net_amount_numerator())
            .ok_or(Error::Overflow)?;
        Ok((numerator / denominator).saturating_add(<BalanceOf<T>>::one()))
    }
}