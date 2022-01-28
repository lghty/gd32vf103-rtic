#![no_main]

#[rtic::app(device = gd32vf103)]
mod app {
    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        (Shared {}, Local {}, init::Monotonics())
    }

    #[task(binds = NonMaskableInt)]
    fn nmi(_: nmi::Context) {}
}
