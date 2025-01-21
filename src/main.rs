mod pdf;

fn main() {
    pretty_env_logger::init();

    let document = pdf::load_document("data/pdf/analysis_of_blood_flow_in_one.pdf")
        .expect("could not load document");
}
