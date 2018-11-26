mod dataset;

use clap::{Arg, App};
use self::dataset::Dataset;

fn main() {
    let matches = App::new("Describe")
        .version("1.0")
        .author("hsabouri <hsabouri@student.42.fr>")
        .about("Display features' informations of a csv dataset.")
        .arg(Arg::with_name("INPUT")
            .help("Sets the csv input file to use")
            .required(true)
            .index(1))
        .get_matches();

    let input = matches.value_of("INPUT").unwrap();
    let dataset = Dataset::from_file(String::from(input));
    dataset.unwrap().display();
}
