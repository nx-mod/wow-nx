//! Stamps the commit this binary was built from; on the Switch, routes deko3d's GPU kickoffs through
//! benilla-horizon's kickoff.c, which reports a failed one, and fork handlers through its tls.c.

fn main() {
    benilla_buildstamp::emit();
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("horizon") {
        println!("cargo:rustc-link-arg-bins=-Wl,--wrap=nvGpuChannelKickoff");
        // benilla-horizon's tls.c: fork handlers register as a no-op (rand panics when newlib refuses them).
        println!("cargo:rustc-link-arg-bins=-Wl,--wrap=pthread_atfork");
    }
}
