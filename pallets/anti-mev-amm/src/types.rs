use super::*;

/// This struct represents a pair in the AMM.
/// It contains the (asset id, the currency reserve, the token reserve, and the liquidity token id).
#[derive(
    Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, MaxEncodedLen, TypeInfo,
)]
pub struct Pair<AssetId, Balance, AssetBalance> {
    pub asset_id: AssetId,
    pub currency_reserve: Balance,
    pub token_reserve: AssetBalance,
    pub liquidity_token_id: AssetId,
}

/// This enum represents the swap type.
/// It can be based on input or output.
#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum Swap<InputBalance, OutputBalance> {
    BasedInput {
        input_amount: InputBalance,
        min_output: OutputBalance,
    },
    BasedOutput {
        max_input: InputBalance,
        output_amount: OutputBalance,
    },
}


// (sold_token_amount, currency_amount, bought_token_amount)
pub type AssetToAssetPrice<T> = (AssetBalanceOf<T>, BalanceOf<T>, AssetBalanceOf<T>);

// Type alias for convenience
pub type PairOf<T> = Pair<AssetIdOf<T>, BalanceOf<T>, AssetBalanceOf<T>>;
