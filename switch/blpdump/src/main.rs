//! blpdump: BLP textures out of a WoW client's MPQs, as PNG. The patches are searched first (patch-2, then patch),
//! then the base archives, as the client reads them.

use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [data, out, names @ ..] = args.as_slice() else {
        eprintln!("usage: blpdump <Data folder> <out folder> <file in the MPQs>...");
        std::process::exit(2);
    };
    let mut archives: Vec<_> = std::fs::read_dir(data)
        .expect("Data folder")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("mpq")))
        .collect();
    // patch-2 before patch before the rest.
    archives.sort_by_key(|p| {
        let n = p.file_name().unwrap().to_string_lossy().to_lowercase();
        (if n.starts_with("patch-2") { 0 } else if n.starts_with("patch") { 1 } else { 2 }, n)
    });
    let opened: Vec<_> = archives.iter().filter_map(|p| benilla_mpq::Archive::open(p).ok().map(|a| (p.clone(), a))).collect();
    std::fs::create_dir_all(out).expect("out folder");
    for name in names {
        let Some((from, archive)) = opened.iter().find(|(_, a)| a.contains(name)) else {
            println!("missing {name}");
            continue;
        };
        let bytes = archive.read_file(name).expect("read");
        let blp = match benilla_blp::decode(&bytes) {
            Ok(b) => b,
            Err(e) => {
                println!("{name}: {e:?}");
                continue;
            }
        };
        let mip = &blp.mips[0];
        let stem = name.rsplit(['\\', '/']).next().unwrap().trim_end_matches(".blp").trim_end_matches(".BLP");
        let path = Path::new(out).join(format!("{stem}.png"));
        let mut enc = png::Encoder::new(std::fs::File::create(&path).unwrap(), mip.width, mip.height);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header().unwrap().write_image_data(&mip.rgba).unwrap();
        println!("{name} ({}x{}) from {} -> {}", mip.width, mip.height, from.file_name().unwrap().to_string_lossy(), path.display());
    }
}
