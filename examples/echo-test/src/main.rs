use alloy_primitives::Address;
use std::{path::PathBuf, str::FromStr};

testsi::testsi_main!();

#[testsi::test_dapp(kind("dapp"))]
pub fn test_echo() -> testsi::TestResult {
    let mut machine = testsi::Machine::try_new(
        &PathBuf::from_str("./echo")?.canonicalize()?,
        Address::ZERO,
        31337,
        0,
    )?;

    let x = vec![2, 3];

    // Input 0
    let input = types::InputBuilder::from_address(Address::ZERO).with_payload(&x);
    let (outputs, _reports) = machine.advance_state(input)?;
    assert_eq!(outputs[0].expect_notice().payload.as_ref(), x);
    assert_eq!(outputs.notices()[0].payload.as_ref(), x);

    Ok(())
}
