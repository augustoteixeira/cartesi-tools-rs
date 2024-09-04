use alloy_primitives::Address;

testsi::testsi_main!();

#[testsi::test_dapp(kind("dapp"))]
pub fn test_echo() -> testsi::TestResult {
    let mut machine = testsi::MachineBuilder::load_from("./echo")
        .at_chain(31337)
        .try_build()?;

    // Input 0
    let input = testsi::InputBuilder::from_address(Address::ZERO).with_payload(&"hello");
    let (outputs, _reports) = machine.advance_state(input)?;
    assert_eq!(
        outputs[0].expect_notice().payload.as_ref(),
        "hello".as_bytes()
    );
    assert_eq!(outputs.notices()[0].payload.as_ref(), "hello".as_bytes());

    Ok(())
}
