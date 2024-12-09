use super::{
	mock::*,
	*,
};
use frame_support::assert_ok;

#[test]
fn should_see_attacker_manipulate_market() {
	new_test_ext().execute_with(|| {
        // Create Pair for ASSET_B
        assert_ok!(AntiMevAmm::create_pair(
            RuntimeOrigin::signed(ACCOUNT_ALICE),
            ASSET_B,
            LIQ_TOKEN_B,
            INIT_LIQUIDITY,
            INIT_LIQUIDITY
        ));

        // At the beginning, both Attacker and Bob are the same
        let attacker_initial_asset_a_balance = <Test_Runtime as Config>::Assets::balance(ASSET_A.clone(), &ACCOUNT_ATTACKER);
        let attacker_initial_asset_b_balance = <Test_Runtime as Config>::Assets::balance(ASSET_B.clone(), &ACCOUNT_ATTACKER);

        let buy_op = CpSwap::BasedInput { 
            input_amount: 100,
            min_output: 10,
        };

        // Attacker try to buy before Bob
        assert_ok!(AntiMevAmm::cp_swap_asset_to_asset(
            RuntimeOrigin::signed(ACCOUNT_ATTACKER),
            ASSET_A,
            ASSET_B,
            buy_op.clone(),
            System::block_number().saturating_add(1)
        ));
        let attacker_after_asset_b_balance = <Test_Runtime as Config>::Assets::balance(ASSET_B.clone(), &ACCOUNT_ATTACKER);
        assert!(attacker_after_asset_b_balance > attacker_initial_asset_b_balance);

        // Bob swap
        assert_ok!(AntiMevAmm::cp_swap_asset_to_asset(
            RuntimeOrigin::signed(ACCOUNT_BOB),
            ASSET_A,
            ASSET_B,
            buy_op,
            System::block_number().saturating_add(1)
        ));

        // Attacker sell
        assert_ok!(AntiMevAmm::cp_swap_asset_to_asset(
            RuntimeOrigin::signed(ACCOUNT_ATTACKER),
            ASSET_B,
            ASSET_A,
            CpSwap::BasedInput { 
                input_amount: attacker_after_asset_b_balance - attacker_initial_asset_b_balance,
                min_output: 10,
            },
            System::block_number().saturating_add(1)
        ));

        // Compare the received values
        let attacker_after_asset_a_balance = <Test_Runtime as Config>::Assets::balance(ASSET_A.clone(), &ACCOUNT_ATTACKER);
        println!("Attacker balance asset a: {:?}", attacker_after_asset_a_balance); 
        let bob_after_asset_a_balance = <Test_Runtime as Config>::Assets::balance(ASSET_A.clone(), &ACCOUNT_BOB);
        println!("Bob balance asset a: {:?}", bob_after_asset_a_balance); 

        println!("Attacker profit: {:?}", attacker_after_asset_a_balance - attacker_initial_asset_a_balance); 
        assert!(attacker_after_asset_a_balance > attacker_initial_asset_a_balance);
	});
}

#[test]
fn should_demo_full_flow_anti_mev() {
    new_test_ext().execute_with(|| {
        let amount_in = 100;
        // Attacker try to buy before Bob
        assert_ok!(AntiMevAmm::add_swap_currency_for_asset(
            RuntimeOrigin::signed(ACCOUNT_ATTACKER),
            ASSET_A,
            amount_in.clone(),
            System::block_number().saturating_add(1)
        ));

        // Bob buy
        assert_ok!(AntiMevAmm::add_swap_currency_for_asset(
            RuntimeOrigin::signed(ACCOUNT_BOB),
            ASSET_A,
            amount_in.clone(),
            System::block_number().saturating_add(1)
        ));

        // Alice sell
        assert_ok!(AntiMevAmm::add_swap_currency_for_asset(
            RuntimeOrigin::signed(ACCOUNT_ALICE),
            ASSET_A,
            amount_in.clone(),
            System::block_number().saturating_add(1)
        ));

        // Attacker sell
        assert_ok!(AntiMevAmm::add_swap_asset_for_currency(
            RuntimeOrigin::signed(ACCOUNT_ATTACKER),
            ASSET_A,
            amount_in.clone(),
            System::block_number().saturating_add(1)
        ));

        // Trigger settlement
        assert_ok!(AntiMevAmm::settle_and_distribute_currency(
            RuntimeOrigin::signed(ACCOUNT_BOB),
            ASSET_A,
        ));

        // Compare the received values
        // Look at the event log for the final balances 😂😂😂
    });
}
