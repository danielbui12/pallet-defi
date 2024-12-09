use super::*;

impl<T: Config> Pallet<T> {
    pub (crate) fn do_anti_mev_swap_currency_for_asset(
        asset_id: &AssetIdOf<T>,
        recipient: &T::AccountId,
        asset_out: &BalanceOf<T>,
        total_cumulative_currency: &BalanceOf<T>,
    ) -> DispatchResult {
        let amount_in: BalanceOf<T> = Self::get_pair_currency_cumulative(&asset_id, &recipient);
        let amount_out: AssetBalanceOf<T> = T::currency_to_asset(
            asset_out.clone() * amount_in / total_cumulative_currency.clone()
        );
        let pallet_account = T::pallet_account();
        T::Assets::transfer(
            asset_id.clone(),
            &pallet_account,
            &recipient,
            amount_out,
            Preservation::Expendable,
        )?;
        Self::deposit_event(Event::SwappedCurrencyForAsset(
            asset_id.clone(),
            recipient.clone(),
            recipient.clone(),
            amount_in,
            amount_out,
        ));
        Ok(())
    }

    pub (crate) fn do_anti_mev_swap_asset_for_currency(
        asset_id: &AssetIdOf<T>,
        recipient: &T::AccountId,
        asset_out: &AssetBalanceOf<T>,
        total_cumulative_asset: &AssetBalanceOf<T>,
    ) -> DispatchResult {
        let amount_in: AssetBalanceOf<T> = Self::get_pair_asset_cumulative(&asset_id, &recipient);
        let amount_out: BalanceOf<T> = T::asset_to_currency(
            asset_out.clone() * amount_in / total_cumulative_asset.clone()
        );
        let pallet_account = T::pallet_account();
        if recipient.clone() != pallet_account {
            <T as pallet::Config>::Currency::transfer(
                &pallet_account,
                &recipient,
                amount_out,
                ExistenceRequirement::AllowDeath,
            )?;
        }
        Self::deposit_event(Event::SwappedAssetForCurrency(
            asset_id.clone(),
            recipient.clone(),
            recipient.clone(),
            amount_out,
            amount_in,
        ));
        Ok(())
    }

    pub(crate) fn add_currency_to_asset_tx(
        asset_id: T::AssetId,
        amount_in: BalanceOf<T>,
        buyer: T::AccountId,
        mut pair_currency_cumulative: BalanceOf<T>,
        mut pair_currency_queue: Vec<T::AccountId>,
    ) -> Result<(), Error<T>> {
        pair_currency_queue.push(buyer.clone());
        <CurrencyToAssetQueue<T>>::insert(asset_id.clone(), pair_currency_queue);
        
        pair_currency_cumulative.saturating_accrue(amount_in);
        <CurrencyToAssetCumulative<T>>::insert(asset_id.clone(), buyer.clone(), pair_currency_cumulative);

        // emit event
        Self::deposit_event(Event::AddedSwapCurrencyForAsset(
            asset_id,
            buyer,
            amount_in,
        ));
        Ok(())
    }
   
    pub(crate) fn add_asset_to_currency_tx(
        asset_id: T::AssetId,
        amount_in: AssetBalanceOf<T>,
        buyer: T::AccountId,
        mut pair_asset_cumulative: AssetBalanceOf<T>,
        mut pair_asset_queue: Vec<T::AccountId>,
    ) -> Result<(), Error<T>> {
        pair_asset_queue.push(buyer.clone());
        <AssetToCurrencyQueue<T>>::insert(asset_id.clone(), pair_asset_queue);
        
        pair_asset_cumulative.saturating_accrue(amount_in);
        <AssetToCurrencyCumulative<T>>::insert(asset_id.clone(), buyer.clone(), pair_asset_cumulative);

        // emit event
        Self::deposit_event(Event::AddedSwapAssetForCurrency(
            asset_id,
            buyer,
            amount_in,
        ));
        Ok(())
    }
 
    pub(crate) fn get_pair_currency_cumulative(
        asset_id: &AssetIdOf<T>,
        account_id: &T::AccountId,
    ) -> BalanceOf<T> {
        <CurrencyToAssetCumulative<T>>::get(asset_id.clone(), account_id.clone())
            .unwrap_or_default()
    }

    pub(crate) fn get_pair_asset_cumulative(
        asset_id: &AssetIdOf<T>,
        account_id: &T::AccountId,
    ) -> AssetBalanceOf<T> {
        <AssetToCurrencyCumulative<T>>::get(asset_id.clone(), account_id.clone())
            .unwrap_or_default()
    }

    pub (crate) fn get_pair_currency_queue(
        asset_id: &AssetIdOf<T>,
    ) -> Result<Vec<T::AccountId>, Error<T>> {
        <CurrencyToAssetQueue<T>>::get(asset_id.clone())
            .ok_or(Error::PairNotFound)
    }

    pub(crate) fn get_pair_asset_queue(
        asset_id: &AssetIdOf<T>,
    ) -> Result<Vec<T::AccountId>, Error<T>> {
        <AssetToCurrencyQueue<T>>::get(asset_id.clone())
            .ok_or(Error::PairNotFound)
    }
}