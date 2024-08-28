use std::ops::Index;

use alloy_primitives::{Address, U256};
use alloy_sol_types::{sol, SolCall};

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

#[derive(Clone, Debug)]
pub enum Output {
    Voucher(Voucher),
    Notice(Notice),
}

impl Output {
    pub fn abi_decode<T: AsRef<[u8]>>(payload: &T) -> Self {
        let payload = payload.as_ref();
        let selector = &payload[..4];
        if selector == Notice::SELECTOR {
            Output::Notice(Notice::abi_decode(payload, true).expect("failed to decode notice"))
        } else {
            assert_eq!(selector, Voucher::SELECTOR);
            Output::Voucher(Voucher::abi_decode(payload, true).expect("failed to decode voucher"))
        }
    }

    pub fn try_notice(&self) -> Option<&Notice> {
        match self {
            Self::Notice(n) => Some(n),
            Self::Voucher(_) => None,
        }
    }

    pub fn expect_notice(&self) -> &Notice {
        self.try_notice()
            .expect(format!("expected voucher {:?} to be a notice", self).as_str())
    }

    pub fn try_voucher(&self) -> Option<&Voucher> {
        match self {
            Self::Notice(_) => None,
            Self::Voucher(v) => Some(v),
        }
    }

    pub fn expect_voucher(&self) -> &Voucher {
        self.try_voucher()
            .expect(format!("expected notice {:?} to be a voucher", self).as_str())
    }
}

#[derive(Clone, Debug, Default)]
pub struct OutputsForInput {
    list: Vec<Output>,
}

impl Index<usize> for OutputsForInput {
    type Output = Output;

    fn index(&self, index: usize) -> &Self::Output {
        &self.list[index]
    }
}

impl OutputsForInput {
    pub fn push(&mut self, output: Output) {
        self.list.push(output);
    }

    pub fn push_encoded<T: AsRef<[u8]>>(&mut self, encoded_output: &T) {
        self.push(Output::abi_decode(encoded_output));
    }

    pub fn list(&self) -> &Vec<Output> {
        &self.list
    }

    pub fn notices(&self) -> Vec<&Notice> {
        self.list.iter().filter_map(|x| x.try_notice()).collect()
    }

    pub fn vouchers(&self) -> Vec<&Voucher> {
        self.list.iter().filter_map(|x| x.try_voucher()).collect()
    }
}

#[derive(Clone, Debug)]
pub struct InputBuilder {
    pub sender: Address,
    pub prev_randao: U256,
    pub block_number: U256,
    pub block_timestamp: U256,
    pub payload: Vec<u8>,
}

impl InputBuilder {
    pub fn from_address(sender: Address) -> Self {
        Self {
            sender,
            prev_randao: U256::ZERO,
            block_number: U256::ZERO,
            block_timestamp: U256::ZERO,
            payload: Vec::new(),
        }
    }

    pub fn at_block(mut self, block: usize) -> Self {
        self.block_number = block.try_into().unwrap();
        self
    }

    pub fn with_payload<T: AsRef<[u8]>>(mut self, payload: &T) -> Self {
        self.payload = payload.as_ref().into();
        self
    }

    pub fn with_block_timestamp(mut self, block_timestamp: usize) -> Self {
        self.block_timestamp = block_timestamp.try_into().unwrap();
        self
    }

    pub fn encode(self, chain_id: usize, input_index: U256, dapp: Address) -> Vec<u8> {
        let x = EvmAdvanceCall::new((
            U256::from(chain_id),
            dapp,
            self.sender,
            self.block_number,
            self.block_timestamp,
            self.prev_randao,
            input_index,
            self.payload.into(),
        ));

        x.abi_encode()
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}
