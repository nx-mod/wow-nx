//! Stamps the commit this binary was built from; on the Switch, routes deko3d's GPU kickoffs through
//! benilla-horizon's kickoff.c, which reports a failed one.

fn main() {
    benilla_buildstamp::emit();
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("horizon") {
        println!("cargo:rustc-link-arg-bins=-Wl,--wrap=nvGpuChannelKickoff");
    }
}
