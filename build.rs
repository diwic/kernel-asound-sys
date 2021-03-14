use regex::Regex;
use std::{env, fs};
use std::path::PathBuf;

fn do_regex(outfile: PathBuf) {
    // Same as wrapper.h
    let files = ["/usr/include/sound/asound.h", "/usr/include/sound/asequencer.h", "/usr/include/sound/tlv.h"];

    let mut out2 = String::new();
    for file in &files {
        let asoundh = fs::read_to_string(file).unwrap();

        // Handle stuff like: #define SNDRV_PCM_STATE_OPEN ((snd_pcm_state_t) 0)
        // which bindgen does not handle (yet)
        for m in Regex::new(r"#define\s+(\w+)\s+\(\((\w+)\)\s*(\d+)\)").unwrap().captures_iter(&asoundh) {
            // eprintln!("{:?}", &m);
            out2 += &format!("pub const {}: {} = {};\n", &m[1], &m[2], &m[3]);
        }

        // Convert stuff like: #define SNDRV_CTL_IOCTL_ELEM_LIST	_IOWR('U', 0x10, struct snd_ctl_elem_list)
        // \(([^,]+),([^,)]+),?([^)]*)\)
        for m in Regex::new(r"#define\s+(\w+)\s+_IO([WR]*)\(([^,)]+),\s*([^,)]+),?\s*([^,)]*)\)").unwrap().captures_iter(&asoundh) {
            let rw = match &m[2] {
                "" => "none",
                "R" => "read",
                "W" => "write",
                "WR" => "readwrite",
                _ => panic!("RW = {}", &m[2])
            };
            let data: &str =
                if rw == "none" { "" }
                else if &m[5] == "int" { "std::os::raw::c_int" }
                else if m[5].starts_with("struct ") { &m[5][7..] }
                else { &m[5] };

            out2 += &format!("ioctl_sys::ioctl!({} {} with b{}, {}{}{});\n", rw, &m[1], &m[3], &m[4],
                if rw == "none" { "" } else { "; "}, data);
        }
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
        // .blacklist_type("_bindgen_ty_.*") // Blacklisting these bogus types will remove SNDRV vars as well
        .layout_tests(false)
        .header("wrapper.h")
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
