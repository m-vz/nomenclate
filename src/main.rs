use pdf::Parser;

mod pdf;

fn main() {
    pretty_env_logger::init();

    let parser = Parser::load("data/pdf/analysis_of_blood_flow_in_one.pdf")
        .expect("could not load document");
}
