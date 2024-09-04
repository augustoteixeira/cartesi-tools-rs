use alloy_sol_types::sol;

// TODO create crate with alloy type definitions in rollup-contracts once this is merged:
// https://github.com/foundry-rs/foundry/pull/7919
sol! {
    #[derive(Debug, PartialEq, Eq)]
    function EvmAdvance(
        uint256 chainId,
        address appContract,
        address msgSender,
        uint256 blockNumber,
        uint256 blockTimestamp,
        uint256 prevRandao,
        uint256 index,
        bytes calldata payload
    ) external;

    #[derive(Debug, PartialEq, Eq)]
    function Notice(bytes calldata payload) external;

    #[derive(Debug, PartialEq, Eq)]
    function Voucher(
        address destination,
        uint256 value,
        bytes calldata payload
    ) external;
}

pub type Input = EvmAdvanceCall;
pub type Voucher = VoucherCall;
pub type Notice = NoticeCall;
