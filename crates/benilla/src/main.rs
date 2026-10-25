//! The `benilla` launcher: a shim that carries the build stamp, so a new commit recompiles these
//! few lines and relinks instead of rebuilding `benilla-app`.

use benilla_app::BuildId;

fn main() -> benilla_app::AppExit {
    let build = BuildId {
        version: env!("CARGO_PKG_VERSION"),
        describe: env!("BENILLA_GIT_DESCRIBE"),
        sha: env!("BENILLA_GIT_SHA"),
        short: env!("BENILLA_GIT_SHORT"),
        date: env!("BENILLA_GIT_DATE"),
        profile: env!("BENILLA_PROFILE"),
        project_dir: env!("BENILLA_PROJECT_DIR"),
        ..Default::default()
    };
    // wow-nx: the Switch plays with the Joy-Con, through benilla-pad.
    #[cfg(target_os = "horizon")]
    return benilla_app::run_with(build, |app| {
        app.add_plugins(benilla_pad::PadPlugin);
    });
    #[cfg(not(target_os = "horizon"))]
    benilla_app::run(build)
}
