use alloy_primitives::U256;

pub struct RollupCmt {
    r: libcmt_sys::cmt_rollup_t,
}

impl RollupCmt {
    pub fn new() -> Self {
        use std::mem::MaybeUninit;

        let r = {
            let mut r: MaybeUninit<libcmt_sys::cmt_rollup_t> = MaybeUninit::uninit();
            let err = unsafe { libcmt_sys::cmt_rollup_init(r.as_mut_ptr()) };
            assert!(err == 0, "failed to instantiate rollup: {}", err);
            unsafe { r.assume_init() }
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
            app_contract: libcmt_sys::cmt_abi_address { data: [0; 20] },
            msg_sender: libcmt_sys::cmt_abi_address { data: [0; 20] },
            block_number: 0,
            block_timestamp: 0,
            index: 0,
            prev_randao: libcmt_sys::cmt_abi_u256 { data: [0; 32] },
            payload: libcmt_sys::cmt_abi_bytes {
                data: std::ptr::null_mut(),
                length: 0,
            },
        };

        unsafe {
            assert!(libcmt_sys::cmt_rollup_finish(&mut self.r, &mut finish) == 0);
        }

        assert!(finish.next_request_type == libcmt_sys::HTIF_YIELD_REASON_ADVANCE as i32);

        unsafe {
            assert!(libcmt_sys::cmt_rollup_read_advance_state(&mut self.r, &mut advance) == 0);
        }

        let payload_length = advance.payload.length as usize;
        let mut payload = vec![0; payload_length];
        payload.copy_from_slice(unsafe {
            std::slice::from_raw_parts(advance.payload.data as *const u8, payload_length)
        });

        types::Input {
            chainId: U256::from(advance.chain_id),
            appContract: advance.app_contract.data.into(),
            msgSender: advance.msg_sender.data.into(),
            blockNumber: U256::from(advance.block_number),
            blockTimestamp: U256::from(advance.block_timestamp),
            prevRandao: U256::from_be_bytes(advance.prev_randao.data),
            index: U256::from(advance.index),
            payload,
        }
    }

    fn emit_voucher(&mut self, voucher: &types::Voucher) {
        let destination = voucher.destination;
        let value = voucher.value.to_be_bytes();
        let mut index = 0;

        unsafe {
            assert!(
                libcmt_sys::cmt_rollup_emit_voucher(
                    &mut self.r,
                    &libcmt_sys::cmt_abi_address {
                        data: **destination
                    },
                    &libcmt_sys::cmt_abi_u256 { data: value },
                    &libcmt_sys::cmt_abi_bytes_t {
                        data: voucher.payload.as_ptr() as *mut std::ffi::c_void,
                        length: voucher.payload.len(),
                    },
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
                    &libcmt_sys::cmt_abi_bytes_t {
                        data: notice.payload.as_ptr() as *mut std::ffi::c_void,
                        length: notice.payload.len()
                    },
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
                    &libcmt_sys::cmt_abi_bytes_t {
                        data: report.as_ptr() as *mut std::ffi::c_void,
                        length: report.len(),
                    }
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
