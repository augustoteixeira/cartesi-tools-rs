use trolley::Rollup;
use types::Notice;

pub fn run(mut rollup: impl Rollup) -> ! {
    loop {
        let i = rollup.next_input();
        rollup.emit_notice(&Notice { payload: i.payload });
    }
}
