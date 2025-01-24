#![warn(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]

mod pdf;

fn main() {
    pretty_env_logger::init();

    pdf::parse_pdf("data/pdf/analysis_of_blood_flow_in_one.pdf", 2)
        .expect("could not load document");
}
