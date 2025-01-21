mod pdf;

fn main() {
    let document = pdf::load_document("data/pdf/analysis_of_blood_flow_in_one.pdf")
        .expect("could not load document");
}
