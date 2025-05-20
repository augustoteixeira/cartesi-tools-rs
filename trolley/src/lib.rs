#[cfg(feature = "cmt")]
pub mod cmt;

mod types;
pub use types::*;

pub trait Rollup {
    fn next_input(&mut self) -> types::Input;
    fn emit_voucher(&mut self, voucher: &types::Voucher);
    fn emit_notice(&mut self, notice: &types::Notice);
    fn emit_report(&mut self, report: &[u8]);
}
