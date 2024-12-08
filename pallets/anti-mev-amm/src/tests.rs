use crate::mock::*;
use frame_support::assert_ok;

#[test]
fn should_see_attacker_manipulate_market() {
	new_test_ext().execute_with(|| {
        // From the origin, both Attacker and Alice are the same
        // Pre compute the expected values

        // Attacker try to buy before Alice

        // Alice swap

        // Attacker sell

        // Compare the received values
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
