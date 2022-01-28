#![no_main]

#[rtic::app(device = gd32vf103_pac, dispatchers = [UnknownInterrupt])]
mod app {
    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        (Shared {}, Local {}, init::Monotonics())
    }
}
