use clap::Parser;

#[derive(Parser)]
pub struct MachineConfig {
    #[arg(long, default_value_t = String::from(super::DEFAULT_REMOTE_MACHINE))]
    pub remote_machine: String,

    #[arg(long, default_value_t = String::from(super::DEFAULT_MACHINE))]
    pub cartesi_machine: String,

    #[arg(long, default_value_t = super::DEFAULT_REMOTE_MACHINE_PORT)]
    pub remote_machine_port: u16,
}

impl Default for MachineConfig {
    fn default() -> Self {
        Self {
            remote_machine: String::from(super::DEFAULT_REMOTE_MACHINE),
            cartesi_machine: String::from(super::DEFAULT_MACHINE),
            remote_machine_port: super::DEFAULT_REMOTE_MACHINE_PORT,
        }
    }
}

impl MachineConfig {
    pub fn machine(mut self, cartesi_machine: String) -> Self {
        self.cartesi_machine = cartesi_machine;
        self
    }

    pub fn remote_machine(mut self, remote_machine: String) -> Self {
        self.remote_machine = remote_machine;
        self
    }

    pub fn remote_machine_port(mut self, remote_machine_port: u16) -> Self {
        self.remote_machine_port = remote_machine_port;
        self
    }
}
