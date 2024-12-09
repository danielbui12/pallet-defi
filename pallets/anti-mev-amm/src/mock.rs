use crate as pallet_anti_mev_amm;
use frame_support::{
    traits::{
        AsEnsureOriginWithArg, ConstU128, ConstU16, ConstU32, ConstU64,
    },
    construct_runtime, parameter_types, derive_impl,
    PalletId,
};
use sp_runtime::{
    traits::{BlakeTwo256, Identity, IdentityLookup},
    BuildStorage,
};
use frame_system::{EnsureRoot, EnsureSigned};
use sp_core::H256;

type Balance = u128;
type Block = frame_system::mocking::MockBlock<TestRuntime>;
type Hash = H256;
type AssetId = u32;

construct_runtime!(
	pub struct TestRuntime {
		System: frame_system,
		Balances: pallet_balances,
        Assets: pallet_assets,
		AntiMevAmm: pallet_anti_mev_amm,
	}
);

// Feel free to remove more items from this, as they are the same as
// `frame_system::config_preludes::TestDefaultConfig`. We have only listed the full `type` list here
// for verbosity. Same for `pallet_balances::Config`.
// https://paritytech.github.io/polkadot-sdk/master/frame_support/attr.derive_impl.html
#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for TestRuntime {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = Hash;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for TestRuntime {
    type Balance = u128;
    type DustRemoval = ();
    type RuntimeEvent = RuntimeEvent;
    type ExistentialDeposit = ConstU128<1>;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = ();
    type MaxReserves = ();
    type ReserveIdentifier = [u8; 8];
    type FreezeIdentifier = ();
    type MaxFreezes = ();
    type RuntimeHoldReason = ();
    type RuntimeFreezeReason = ();
}

impl pallet_assets::Config for TestRuntime {
    type RuntimeEvent = RuntimeEvent;
    type Balance = Balance;
    type AssetId = AssetId;
    type AssetIdParameter = AssetId;
    type Currency = Balances;
    type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<u64>>;
    type ForceOrigin = EnsureRoot<u64>;
    type AssetDeposit = ConstU128<1>;
    type AssetAccountDeposit = ConstU128<10>;
    type MetadataDepositBase = ConstU128<1>;
    type MetadataDepositPerByte = ConstU128<1>;
    type ApprovalDeposit = ConstU128<1>;
    type StringLimit = ConstU32<50>;
    type Freezer = ();
    type Extra = ();
    type WeightInfo = ();
    type RemoveItemsLimit = ConstU32<5>;
    type CallbackHandle = ();
}


parameter_types! {
    pub const AniMevAmmPalletId: PalletId = PalletId(*b"anti_mev");
}

impl pallet_anti_mev_amm::Config for TestRuntime {
    type PalletId = AniMevAmmPalletId;
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type AssetBalance = Balance;
    type AssetToCurrencyBalance = Identity;
    type CurrencyToAssetBalance = Identity;
    type AssetId = u32;
    type Assets = Assets;
    type AssetRegistry = Assets;
    type WeightInfo = ();
    // Provider fee is 0.3%
    type ProviderFeeNumerator = ConstU128<3>;
    type ProviderFeeDenominator = ConstU128<1000>;
    type MinInitialCurrency = ConstU128<MIN_INITIAL_CURRENCY>;
    type MinInitialToken = ConstU128<MIN_INITIAL_TOKEN>;
    type Fragment = ConstU32<10>;
    // Max queue amount is 2, there can be at most 4 transactions
    type MinQueueAmount = ConstU32<2>;
}

pub(crate) const ACCOUNT_ALICE: u64 = 0;
pub(crate) const ACCOUNT_BOB: u64 = 1;
pub(crate) const ACCOUNT_CHARLIE: u64 = 2;
pub(crate) const ACCOUNT_DAVE: u64 = 3;
pub(crate) const ACCOUNT_ERWIN: u64 = 4;
pub(crate) const ACCOUNT_ATTACKER: u64 = 5;

pub(crate) const MIN_INITIAL_CURRENCY: u128 = 1;
pub(crate) const MIN_INITIAL_TOKEN: u128 = 1;
pub(crate) const INIT_BALANCE: u128 = 1_000_000_000_000_000;
pub(crate) const INIT_LIQUIDITY: u128 = 1_000_000_000_000;

pub(crate) const ASSET_A: u32 = 100;
pub(crate) const ASSET_B: u32 = 101;

pub(crate) const LIQ_TOKEN_A: u32 = 200;
pub(crate) const LIQ_TOKEN_B: u32 = 201;

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    // Initialize the storage of the mock runtime
    let mut storage = frame_system::GenesisConfig::<TestRuntime>::default()
        .build_storage()
        .unwrap();
   
    // Initialize the balances of the accounts
    pallet_balances::GenesisConfig::<TestRuntime> {
        balances: vec![
            (ACCOUNT_ALICE, INIT_BALANCE),
            (ACCOUNT_BOB, INIT_BALANCE),
            (ACCOUNT_CHARLIE, INIT_BALANCE),
            (ACCOUNT_DAVE, INIT_BALANCE),
            (ACCOUNT_ERWIN, INIT_BALANCE),
            (ACCOUNT_ATTACKER, INIT_BALANCE),
        ],
    }
    .assimilate_storage(&mut storage)
    .unwrap();
    
    // Initialize the assets
	pallet_assets::GenesisConfig::<TestRuntime> {
        assets: vec![
            (ASSET_A, ACCOUNT_ALICE, true, 1),
            (ASSET_B, ACCOUNT_ALICE, true, 1),
        ],
        metadata: vec![],
        accounts: vec![
            (ASSET_A, ACCOUNT_ALICE, INIT_BALANCE),
            (ASSET_A, ACCOUNT_BOB, INIT_BALANCE),
            (ASSET_A, ACCOUNT_ATTACKER, INIT_BALANCE),
            (ASSET_B, ACCOUNT_ALICE, INIT_BALANCE),
            (ASSET_B, ACCOUNT_BOB, INIT_BALANCE),
            (ASSET_B, ACCOUNT_ATTACKER, INIT_BALANCE),
        ],
	}
    .assimilate_storage(&mut storage)
	.unwrap();

    // Initialize genesis assets
    pallet_anti_mev_amm::GenesisConfig::<TestRuntime> {
        pairs: vec![(ACCOUNT_ALICE, ASSET_A, LIQ_TOKEN_A, INIT_LIQUIDITY, INIT_LIQUIDITY)],
    }
    .assimilate_storage(&mut storage)
    .unwrap();

	

    // Create the test externalities
    let mut test_ext: sp_io::TestExternalities = storage.into();
    test_ext.execute_with(|| System::set_block_number(1));
    test_ext
}

// pub(crate) fn last_event() -> pallet_anti_mev_amm::Event<TestRuntime> {
//     last_n_events(1).pop().unwrap()
// }

// pub(crate) fn last_n_events(n: usize) -> Vec<pallet_anti_mev_amm::Event<TestRuntime>> {
//     let mut events: Vec<pallet_anti_mev_amm::Event<TestRuntime>> = System::events()
//         .into_iter()
//         .map(|r| r.event)
//         .filter_map(|event| match event {
//             RuntimeEvent::AntiMevAmm(inner) => Some(inner),
//             _ => None,
//         })
//         .collect();
//     events.split_off(events.len() - n)
// }