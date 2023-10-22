use alloy_primitives::{Address, Selector};

#[derive(Clone, Debug)]
pub enum AdvanceStatus {
    Ok(Outputs),
    Rejected,
}

#[derive(Clone, Debug)]
pub struct Outputs {
    pub notices: Vec<Notice>,
    pub vouchers: Vec<Voucher>,
}

#[derive(Clone, Debug)]
pub struct Notice {
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct Voucher {
    pub destination: Address,
    pub selector: Selector,
    pub args: Vec<u8>,
}

impl Voucher {
    pub fn new(data: impl AsRef<[u8]>) -> Self {
        let data = data.as_ref();
        let destination = Address::new(data[12..32].try_into().unwrap());
        let selector = Selector::new(data[32..36].try_into().unwrap());
        let args = data[36..].into();

        Self {
            destination,
            selector,
            args,
        }
    }
}
