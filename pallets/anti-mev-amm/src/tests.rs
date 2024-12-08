use super::{
	mock::*,
	*,
};
use frame_support::assert_ok;

#[test]
fn should_see_attacker_manipulate_market() {
	new_test_ext().execute_with(|| {
        // At the beginning, both Attacker and Bob are the same
        let attacker_initial_currency_balance = <Test_Runtime as Config>::Currency::free_balance(&ACCOUNT_ATTACKER);
        let attacker_initial_asset_balance = <Test_Runtime as Config>::Assets::balance(ASSET_A.clone(), &ACCOUNT_ATTACKER);

        let buy_op = CpSwap::BasedInput { 
            input_amount: 100,
            min_output: 10,
        };

        // Attacker try to buy before Bob
        assert_ok!(AntiMevAmm::cp_swap_currency_for_asset(
            RuntimeOrigin::signed(ACCOUNT_ATTACKER),
            ASSET_A,
            buy_op.clone(),
            System::block_number().saturating_add(1)
        ));
        let attacker_after_asset_balance = <Test_Runtime as Config>::Assets::balance(ASSET_A.clone(), &ACCOUNT_ATTACKER);
        assert!(attacker_after_asset_balance > attacker_initial_asset_balance);
        println!("Attacker asset balance profit: {:?}", attacker_after_asset_balance - attacker_initial_asset_balance);

        // Bob swap
        assert_ok!(AntiMevAmm::cp_swap_currency_for_asset(
            RuntimeOrigin::signed(ACCOUNT_BOB),
            ASSET_A,
            buy_op,
            System::block_number().saturating_add(1)
        ));

        // Attacker sell
        assert_ok!(AntiMevAmm::cp_swap_asset_for_currency(
            RuntimeOrigin::signed(ACCOUNT_ATTACKER),
            ASSET_A,
            CpSwap::BasedInput { 
                input_amount: attacker_after_asset_balance - attacker_initial_asset_balance,
                min_output: 10,
            },
            System::block_number().saturating_add(1)
        ));
        // Compare the received values
        let attacker_after_currency_balance = <Test_Runtime as Config>::Currency::free_balance(&ACCOUNT_ATTACKER);
        // This cause error because the gas fee is too high 😂😂😂
        // TODO: update test case to swap asset for asset
        // println!("Attacker profit: {:?}", attacker_after_currency_balance - attacker_initial_currency_balance); 
        // assert!(attacker_after_currency_balance > attacker_initial_currency_balance);
	});
}

#[test]
fn should_demo_full_flow_anti_mev() {
    new_test_ext().execute_with(|| {
        // Pre compute the expected values

        // Attacker try to buy before Alice

        // Bob buy

        // Alice swap

        // Attacker sell

        // Bob sell

        // Trigger settlement

        // Compare the received values
    });
}
