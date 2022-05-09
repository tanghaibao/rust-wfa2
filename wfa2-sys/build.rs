extern crate bindgen;
extern crate cc;

use fs_utils::copy::copy_directory;

use std::env;
use std::path::PathBuf;

// these need to be kept in sync with the WFA2 Makefile
const FILES: &[&str] = &[
    "./utils/string_padded.c",
    "./utils/bitmap.c",
    "./utils/vector.c",
    "./utils/commons.c",
    "./utils/sequence_buffer.c",
    "./utils/heatmap.c",
    "./utils/dna_text.c",
    "./system/mm_stack.c",
    "./system/profiler_counter.c",
    "./system/profiler_timer.c",
    "./system/mm_allocator.c",
    "./alignment/affine_penalties.c",
    "./alignment/cigar.c",
    "./alignment/score_matrix.c",
    "./alignment/affine2p_penalties.c",
    "./wavefront/wavefront_display.c",
    "./wavefront/wavefront_pcigar.c",
    "./wavefront/wavefront.c",
    "./wavefront/wavefront_compute_affine.c",
    "./wavefront/wavefront_compute_affine2p.c",
    "./wavefront/wavefront_penalties.c",
    "./wavefront/wavefront_aligner.c",
    "./wavefront/wavefront_backtrace.c",
    "./wavefront/wavefront_attributes.c",
    "./wavefront/wavefront_slab.c",
    "./wavefront/wavefront_extend.c",
    "./wavefront/wavefront_backtrace_buffer.c",
    "./wavefront/wavefront_align.c",
    "./wavefront/wavefront_debug.c",
    "./wavefront/wavefront_compute_linear.c",
    "./wavefront/wavefront_components.c",
    "./wavefront/wavefront_compute.c",
    "./wavefront/wavefront_compute_edit.c",
    "./wavefront/wavefront_heuristic.c",
    "./wavefront/wavefront_backtrace_offload.c",
    "./wavefront/wavefront_plot.c",
];

fn main() {
    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    let mut cfg = cc::Build::new();

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out_wfa2 = out.join("WFA2-lib");

    if out_wfa2.exists() {
        std::fs::remove_dir_all(&out_wfa2).unwrap();
    }
    copy_directory("WFA2-lib", &out).unwrap();

    let wfa2 = PathBuf::from("WFA2-lib");

    for f in FILES {
        let c_file = out_wfa2.join(f);
        cfg.file(&c_file);
        println!("cargo:rerun-if-changed={}", wfa2.join(c_file).display());
    }
    cfg.include(out_wfa2);
    cfg.compile("wfa2");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        .clang_arg("-IWFA2-lib")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
