use super::*;
// use frame_system::pallet_prelude::BlockNumberFor;

// pub type BalanceOf<T> =
// 	<<T as Config>::Currency as fungible::Inspect<<T as frame_system::Config>::AccountId>>::Balance;
type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
