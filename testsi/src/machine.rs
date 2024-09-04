use crate::types::{InputBuilder, OutputsForInput};

use alloy_primitives::{Address, U256};
use cartesi_machine::configuration::MachineConfigRef;
use std::{ops::ControlFlow, path::PathBuf};

type MachineError = cartesi_machine::errors::MachineError;
type Result<T> = std::result::Result<T, MachineError>;
type Report = Vec<u8>;

pub struct MachineBuilder {
    cartesi_machine_path: PathBuf,
    chain_id: usize,
    dapp_address: Address,
    input_index: usize,
    no_console_putchar: bool,
}

impl MachineBuilder {
    pub fn load_from<T: Into<PathBuf>>(path: T) -> MachineBuilder {
        Self {
            cartesi_machine_path: path.into(),
            chain_id: 1,
            dapp_address: Address::ZERO,
            input_index: 0,
            no_console_putchar: true,
        }
    }

    pub fn at_chain(mut self, chain_id: usize) -> MachineBuilder {
        self.chain_id = chain_id;
        self
    }

    pub fn deployed_at(mut self, dapp_address: Address) -> MachineBuilder {
        self.dapp_address = dapp_address;
        self
    }

    pub fn with_input_count(mut self, input_index: usize) -> MachineBuilder {
        self.input_index = input_index;
        self
    }

    pub fn no_console_putchar(mut self, no_console_putchar: bool) -> MachineBuilder {
        self.no_console_putchar = no_console_putchar;
        self
    }

    pub fn try_build(self) -> Result<Machine> {
        Machine::try_new(self)
    }
}

pub struct Machine {
    cartesi_machine: cartesi_machine::machine::Machine,
    builder: MachineBuilder,
}

impl Machine {
    pub fn try_new(builder: MachineBuilder) -> Result<Self> {
        let runtime_config = cartesi_machine::configuration::RuntimeConfig::default()
            .no_console_putchar(builder.no_console_putchar);

        // Instantiate Machine
        let cartesi_machine = {
            let cm = cartesi_machine::Machine::load(&builder.cartesi_machine_path, runtime_config)?;
            let c = cm.initial_config()?;
            sanity_check_cm_config(&c);
            cm
        };

        Ok(Self {
            cartesi_machine,
            builder,
        })
    }

    pub fn advance_state(
        &mut self,
        input: InputBuilder,
    ) -> Result<(OutputsForInput, Vec<Vec<u8>>)> {
        let encoded_input = input.encode(
            self.builder.chain_id,
            U256::from(self.builder.input_index),
            self.builder.dapp_address,
        );

        self.cartesi_machine.send_cmio_response(
            cartesi_machine::htif::fromhost::ADVANCE_STATE,
            &encoded_input,
        )?;

        let mut outputs = OutputsForInput::default();
        let mut reports = Vec::new();

        loop {
            match run_machine_increment(&mut self.cartesi_machine, &mut outputs, &mut reports)? {
                ControlFlow::Continue(_) => continue,
                ControlFlow::Break(_) => break,
            }
        }

        Ok((outputs, reports))
    }

    // TODO
    pub fn inspect(&self) -> Result<()> {
        todo!()
    }
}

fn run_machine_increment(
    cartesi_machine: &mut cartesi_machine::machine::Machine,
    outputs: &mut OutputsForInput,
    reports: &mut Vec<Report>,
) -> Result<ControlFlow<()>> {
    use cartesi_machine::break_reason;

    let break_reason = cartesi_machine.run(1 << 40)?;

    let control_flow = match break_reason {
        break_reason::FAILED => panic!("run failed"),

        break_reason::HALTED => {
            // TODO should it revert?
            panic!("run halted: {}", cartesi_machine.read_mcycle()?)
        }

        // TODO Implement increment and timeout
        break_reason::REACHED_TARGET_MCYCLE => todo!(),

        break_reason::YIELDED_MANUALLY => {
            handle_manual_yield(cartesi_machine)?;
            ControlFlow::Break(())
        }

        break_reason::YIELDED_AUTOMATICALLY => {
            handle_automatic_yield(cartesi_machine, outputs, reports)?;
            ControlFlow::Continue(())
        }

        // TODO
        break_reason::YIELDED_SOFTLY => todo!(),

        i => unreachable!("cartesi machine impossible break reason: {}", i),
    };

    Ok(control_flow)
}

fn handle_manual_yield(cartesi_machine: &mut cartesi_machine::machine::Machine) -> Result<()> {
    use cartesi_machine::htif;

    let (_, reason, _) = get_yield(cartesi_machine)?;

    match reason {
        htif::tohost::manual::RX_ACCEPTED => (),

        htif::tohost::manual::TX_EXCEPTION => {
            unimplemented!("TX_EXCEPTION not implemented")
        }

        htif::tohost::manual::RX_REJECTED => {
            unimplemented!("RX_REJECTED not implemented")
        }

        i => unreachable!("cartesi machine impossible manual reason: {}", i),
    }

    Ok(())
}

fn handle_automatic_yield(
    cartesi_machine: &mut cartesi_machine::machine::Machine,
    outputs: &mut OutputsForInput,
    reports: &mut Vec<Report>,
) -> Result<()> {
    use cartesi_machine::htif;

    let (_, reason, length) = get_yield(cartesi_machine)?;
    let data = cartesi_machine.read_memory(cartesi_machine::pma::CMIO_TX_BUFFER_START, length)?;

    match reason {
        htif::tohost::automatic::PROGRESS => (),

        htif::tohost::automatic::TX_OUTPUT => {
            outputs.push_encoded(&data);
        }

        htif::tohost::automatic::TX_REPORT => {
            reports.push(data);
        }

        i => unreachable!("cartesi machine impossible automatic reason: {}", i),
    }

    Ok(())
}

fn sanity_check_cm_config(config: &MachineConfigRef) {
    // Sanity checks
    assert!(
        config.inner().htif.yield_manual,
        "yield manual must be enabled for cmio"
    );
    assert!(
        config.inner().htif.yield_automatic,
        "yield automatic must be enabled for cmio"
    );

    assert!(
        !config.inner().htif.console_getchar,
        "console getchar must be disabled for cmio"
    );

    check_cmio_memory_range_config(config.inner().cmio.tx_buffer, "tx_buffer");
    check_cmio_memory_range_config(config.inner().cmio.rx_buffer, "rx_buffer");
}

fn check_cmio_memory_range_config(
    range: cartesi_machine::configuration::CmIoBufferConfig,
    name: &str,
) {
    assert!(!range.shared, "cmio range {} cannot be shared", name);
}

fn get_yield(machine: &cartesi_machine::Machine) -> Result<(isize, u32, u64)> {
    let cmd = machine.read_htif_tohost_cmd()? as isize;
    let data = machine.read_htif_tohost_data()?;

    let reason = data >> 32;
    let m16 = (1 << 16) - 1;
    let reason = reason & m16;
    let m32 = (1 << 32) - 1;
    let length = data & m32;

    Ok((cmd, reason as u32, length))
}
