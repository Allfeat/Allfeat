use crate::{mock::*, NftsSwapPrecompileCall};

type PCall = NftsSwapPrecompileCall<Runtime>;

fn _precompiles() -> Precompiles<Runtime> {
	PrecompilesValue::get()
}

#[test]
fn selectors() {
	assert!(PCall::create_swap_selectors().contains(&0xf93f143a));
	assert!(PCall::cancel_swap_selectors().contains(&0x83698d19));
	assert!(PCall::claim_swap_selectors().contains(&0x0406ab6c));
}
