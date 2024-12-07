// use crate::mock::*;
// use frame_support::assert_ok;

// #[test]
// fn timestamp_works() {
// 	new_test_ext().execute_with(|| {
// 		crate::Now::<Test>::put(46);
// 		assert_ok!(Timestamp::set(RuntimeOrigin::none(), 69));
// 		assert_eq!(Timestamp::now(), 69);
// 		assert_eq!(Some(69), get_captured_moment());
// 	});
// }
