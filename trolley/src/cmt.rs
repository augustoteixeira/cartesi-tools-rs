use alloy_primitives::U256;

pub struct RollupCmt {
    r: libcmt_sys::cmt_rollup_t,
}

impl RollupCmt {
    pub fn new() -> Self {
        use std::mem::MaybeUninit;

        let mut r_: MaybeUninit<libcmt_sys::cmt_rollup_t> = MaybeUninit::zeroed();
        let r = unsafe {
            let err = libcmt_sys::cmt_rollup_init(r_.as_mut_ptr());
            assert!(err == 0, "failed to instantiate rollup: {}", err);

            r_.assume_init()
        };

        Self { r }
    }
}

impl crate::Rollup for RollupCmt {
    fn next_input(&mut self) -> types::Input {
        let mut finish = libcmt_sys::cmt_rollup_finish {
            accept_previous_request: true,
            next_request_type: 0,
            next_request_payload_length: 0,
        };

        let mut advance = libcmt_sys::cmt_rollup_advance_t {
            chain_id: 0,
            app_contract: [0; 20],
            msg_sender: [0; 20],
            block_number: 0,
            block_timestamp: 0,
            index: 0,
            payload_length: 0,
            payload: std::ptr::null_mut(),
        };

        unsafe {
            assert!(libcmt_sys::cmt_rollup_finish(&mut self.r, &mut finish) == 0);
        }

        assert!(finish.next_request_type == libcmt_sys::HTIF_YIELD_REASON_ADVANCE as i32);

        unsafe {
            assert!(libcmt_sys::cmt_rollup_read_advance_state(&mut self.r, &mut advance) == 0);
        }

        let payload_length = advance.payload_length as usize;
        let mut payload = Vec::with_capacity(payload_length);
        payload.copy_from_slice(unsafe {
            std::slice::from_raw_parts(advance.payload as *const u8, payload_length)
        });

        types::Input {
            chainId: U256::from(advance.chain_id),
            appContract: advance.app_contract.into(),
            msgSender: advance.msg_sender.into(),
            blockNumber: U256::from(advance.block_number),
            blockTimestamp: U256::from(advance.block_timestamp),
            index: U256::from(advance.index),
            payload,
        }
    }

    fn emit_voucher(&mut self, voucher: &types::Voucher) {
        let destination = voucher.destination.to_vec();
        let value = voucher.value.as_le_slice();
        let mut index = 0;

        unsafe {
            assert!(
                libcmt_sys::cmt_rollup_emit_voucher(
                    &mut self.r,
                    destination.len() as u32,
                    destination.as_ptr() as *const std::ffi::c_void,
                    value.len() as u32,
                    value.as_ptr() as *const std::ffi::c_void,
                    voucher.payload.len() as u32,
                    voucher.payload.as_ptr() as *const std::ffi::c_void,
                    &mut index
                ) == 0,
                "failed emitting voucher"
            )
        }
    }

    fn emit_notice(&mut self, notice: &types::Notice) {
        let mut index = 0;
        unsafe {
            assert!(
                libcmt_sys::cmt_rollup_emit_notice(
                    &mut self.r,
                    notice.payload.len() as u32,
                    notice.payload.as_ptr() as *const std::ffi::c_void,
                    &mut index
                ) == 0,
                "failed emitting notice"
            )
        }
    }

    fn emit_report(&mut self, report: &[u8]) {
        unsafe {
            assert!(
                libcmt_sys::cmt_rollup_emit_report(
                    &mut self.r,
                    report.len() as u32,
                    report.as_ptr() as *const std::ffi::c_void
                ) == 0,
                "failed emitting report"
            )
        }
    }
}

impl Drop for RollupCmt {
    fn drop(&mut self) {
        unsafe { libcmt_sys::cmt_rollup_fini(&mut self.r) }
    }
}
