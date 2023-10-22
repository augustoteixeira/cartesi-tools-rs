// use test_case::*;

use alloy_primitives::Address;
use alloy_sol_types::{sol, SolType};
use testsi::{emulator::output::AdvanceStatus, Input, Machine};

testsi::testsi_main!();

type SolInput = sol! { bytes };

#[testsi::test_dapp(kind("dapp"))]
pub fn ba() -> testsi::TestResult {
    let mut m = Machine::default();

    // Input 0
    let status = m.advance_state(Input::new(Address::ZERO).with_payload("hello"));
    match status {
        AdvanceStatus::Ok(x) => {
            assert_eq!(x.notices[0].payload, SolInput::abi_encode("hello"));
        }
        AdvanceStatus::Rejected => panic!("should not have reverted"),
    }

    Ok(())
}
