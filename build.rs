use regex::Regex;
use std::{env, fs};
use std::path::PathBuf;

fn do_regex(outfile: PathBuf) {
    let asoundh = fs::read_to_string("/usr/include/sound/asound.h").unwrap();
    let mut out2 = String::new();

    // Handle stuff like: #define SNDRV_PCM_STATE_OPEN ((snd_pcm_state_t) 0)
    // which bindgen does not handle (yet)
    for m in Regex::new(r"#define\s+(\w+)\s+\(\((\w+)\)\s*(\d+)\)").unwrap().captures_iter(&asoundh) {
        // eprintln!("{:?}", &m);
        out2 += &format!("pub const {}: {} = {};\n", &m[1], &m[2], &m[3]);
    }

    // Convert stuff like: #define SNDRV_CTL_IOCTL_ELEM_LIST	_IOWR('U', 0x10, struct snd_ctl_elem_list)
    // \(([^,]+),([^,)]+),?([^)]*)\)
    for m in Regex::new(r"#define\s+(\w+)\s+_IO([WR]*)\(([^,)]+),\s*([^,)]+),?\s*([^,)]*)\)").unwrap().captures_iter(&asoundh) {
        let mut s2: &str = &m[5];
        if s2.starts_with("struct ") { s2 = s2.split_at(7).1; }

        let s = match &m[2] {
            "" => {
                out2 += &format!("nix::ioctl_none!({}, {}, {});\n", &m[1], &m[3], &m[4]);
                continue;
            },
            "W" => {
                if s2 == "int" {
                    out2 += &format!("nix::ioctl_write_int!({}, {}, {});\n", &m[1], &m[3], &m[4]);
                    continue;
                }
                "write_ptr"
            },
            "R" => "read",
            "WR" => "readwrite",
            _ => { panic!("m = {:?}", &m); }
        };

        if s2 == "int" { s2 = "std::os::raw::c_int" };
        // else { panic!("m = {:?}, s2 = {:?}", &m, s2); }

        out2 += &format!("nix::ioctl_{}!({}, {}, {}, {});\n", s, &m[1], &m[3], &m[4], s2);
    }

    fs::write(outfile, out2).unwrap();
}

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    do_regex(out_path.join("regex.rs"));

    let bindings = bindgen::Builder::default()
        .size_t_is_usize(true)
        .whitelist_type("snd_.*")
        .whitelist_var("SNDRV_.*")
        .header("wrapper.h")
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
