use std::fs::File;
use std::error::Error;
use std::f32;

use prettytable::{Table, Row, Cell};

#[derive(Clone, Debug)]
enum Value {
    Exist(f32),
    Empty
}

#[derive(Clone, Debug)]
enum FeatureContainer {
    String(Vec<String>),
    Continuous(Vec<Value>),
    Discreat(Vec<String>),
    Empty(Vec<Value>)
}

#[derive(Debug)]
struct Feature {
    pub name: String,
    content: FeatureContainer,
    count: usize,
    mean: Option<f32>,
    std: Option<f32>,
    min: Option<f32>,
    max: Option<f32>,
    quart: Option<f32>,
    median: Option<f32>,
    last_quart: Option<f32>,
    discreat_number: Option<usize>,
}

impl Feature {
    pub fn new(name: String, raw: Vec<String>) -> Feature {
        let raw_count = raw.len();
        let try_parse: Vec<Value> = raw.iter().filter_map(
            |v| match v.trim().parse::<f32>() {
                Ok(value) => {
                    Some(Value::Exist(value))
                },
                Err(_) => {
                    if v.len() == 0 { Some(Value::Empty) } else { None }
                }
            }
        ).collect();

        let count: usize;
        let discreat_number: Option<usize>;
        let parsed = if try_parse.len() == raw_count {
            count = try_parse.iter().fold(0, |acc, value| match value {
                Value::Exist(_) => acc + 1,
                Value::Empty => acc
            });
            
            discreat_number = None;
            if count > 0 {
                FeatureContainer::Continuous(try_parse)
            } else {
                FeatureContainer::Empty(try_parse)
            }
        } else {
            let mut try_discreat = raw.clone();
            try_discreat.sort();

            let number_of_values = try_discreat.iter().fold((String::new(), 0), |acc, value| {
                let value = value.to_string();

                if value.len() > 0 && acc.0 != value { (value, acc.1 + 1) } else { acc }
            }).1;

            count = raw_count;

            if number_of_values < count / 10 {
                discreat_number = Some(number_of_values);
                FeatureContainer::Discreat(raw.to_owned())
            } else {
            discreat_number = None;
                FeatureContainer::String(raw.to_owned())
            }
        };

        Feature {
            name: name,
            content: parsed,
            count: count,
            mean: None,
            std: None,
            min: None,
            max: None,
            quart: None,
            median: None,
            last_quart: None,
            discreat_number: discreat_number,
        }
    }

    pub fn get_mean(&mut self) -> Option<f32> {
        let mean = self.mean;
        let count = self.count as f32;
        let content = &self.content;

        self.mean = match (mean, content) {
            (Some(stored), _) => Some(stored),
            (None, FeatureContainer::Continuous(content)) =>
                Some(content.iter().fold(0.0, |acc, x| {
                    match x {
                        Value::Exist(x) => acc + x,
                        Value::Empty => acc
                    }
                }) / count),
            (_, _) => None
        };
        self.mean
    }

    pub fn get_std(&mut self) -> Option<f32> {
        let mean = self.get_mean();
        let std = self.std;
        let count = self.count;
        let content = &self.content;

        self.std = match (std, mean, content) {
            (Some(v), _, _) => Some(v),
            (None, Some(m), FeatureContainer::Continuous(content)) => Some(
                (content.iter()
                    .fold(0.0, |acc, x|
                        match x {
                            Value::Exist(x) => acc + (x - m).powi(2),
                            Value::Empty => acc
                        }
                    ) / count as f32
                ).sqrt()
            ),
            (_, _, _) => None,
        };
        self.std
    }

    pub fn get_min(&mut self) -> Option<f32> {
        let (min, _) = self.get_min_max();

        min
    }

    pub fn get_max(&mut self) -> Option<f32> {
        let (_, max) = self.get_min_max();

        max
    }

    pub fn get_quantils(&mut self) -> Option<(f32, f32, f32)> {
        let quart = self.quart;
        let median = self.median;
        let last_quart = self.last_quart;

        match (quart, median, last_quart, &mut self.content) {
            (Some(quart), Some(median), Some(last_quart), _) => Some((quart, median, last_quart)),
            (_, _, _, FeatureContainer::Continuous(content)) => {
                let mut content: Vec<&f32> = content.iter().filter_map(|x| match x {
                    Value::Exist(value) => Some(value),
                    Value::Empty => None
                }).collect();

                let len = content.len();
                if len == 0 {
                    return Some((f32::NAN, f32::NAN, f32::NAN))
                };

                let quart = (len + 1) / 4;
                let median = (len + 1) / 2;
                let last_quart = len - len / 4 ;

                content.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
                let quart = content[quart];
                let median = content[median];
                let last_quart = content[last_quart];
                
                self.quart = Some(*quart);
                self.median = Some(*median);
                self.last_quart = Some(*last_quart);

                Some((*quart, *median, *last_quart))
            }
            (_, _, _, _) => None
        }
    }

    pub fn get_quart(&mut self) -> Option<f32> {
        let quantils = self.get_quantils();

        match quantils {
            None => None,
            Some((quart, _, _)) => Some(quart)
        }
    }

    pub fn get_median(&mut self) -> Option<f32> {
        let quantils = self.get_quantils();

        match quantils {
            None => None,
            Some((_, median, _)) => Some(median)
        }
    }

    pub fn get_last_quart(&mut self) -> Option<f32> {
        let quantils = self.get_quantils();

        match quantils {
            None => None,
            Some((_, _, last_quart)) => Some(last_quart)
        }
    }

    pub fn get_min_max(&mut self) -> (Option<f32>, Option<f32>) {
        let min = self.min;
        let max = self.max;
        let content = &self.content;

        let (min, max) = match (min, max, content) {
            (Some(min), Some(max), _) => (Some(min), Some(max)),
            (None, None, FeatureContainer::Continuous(content)) => {
                let content = content.iter().filter_map(|x| match x {
                    Value::Exist(value) => Some(value),
                    Value::Empty => None
                });

                let start = content.clone().nth(0).clone();

                match start {
                    Some(start) => {
                        let (min, max) = content.fold((start, start), |acc, current| {
                            (if current < acc.0 { current } else { acc.0 },
                             if current > acc.1 { current } else { acc.1 })
                        });

                        (Some(*min), Some(*max))
                    },
                    None => (None, None)
                }
            },
            (_, _, _) => (None, None),
        };
        self.min = min;
        self.max = max;
        (self.min, self.max)
    }
}

#[derive(Debug)]
pub struct Dataset {
    features: Vec<Feature>,
}

impl Dataset {
    fn read_file(input: String) -> Result<Vec<(String, Vec<String>)>, Box<Error>> {
        let file = File::open(input)?;
        let mut reader = csv::Reader::from_reader(file);
        let headers = reader.headers()?;
        let mut dataset: Vec<(String, Vec<String>)> = headers.iter().map(|header| (
            String::from(header),
            Vec::<String>::new()
        )).collect();

        for result in reader.records() {
            let record = result?;

            record.iter().enumerate().for_each(|(i, col)| {
                dataset[i].1.push(String::from(col))
            });
        }
        Ok(dataset)
    }

    pub fn from_file(file_name: String) -> Result<Dataset, Box<Error>> {
        let dataset_raw: Vec<(String, Vec<String>)> = Dataset::read_file(file_name)?;

        Ok(Dataset {
            features: dataset_raw.iter().map(|(name, content)| {
                Feature::new(name.to_string(), content.to_vec())
            }).collect()
        })
    }

    fn build_row(name: &str, mut content: Vec<Cell>) -> Row {
        content.insert(0, Cell::new(name));
        Row::new(content)
    }

    fn metric_row(&self, name: &str, getter: fn(&Feature) -> Option<f32>) -> Row {
        let row: Vec<Cell> = self.features.iter().filter_map(|feature|
            match getter(feature) {
                Some(mean) => Some(Cell::new(format!("{}", mean).as_str())),
                None => None
            }
        ).collect();
        Dataset::build_row(name, row)
    }

    fn metric_row_usize(&self, name: &str, getter: fn(&Feature) -> Option<usize>) -> Row {
        let row: Vec<Cell> = self.features.iter().filter_map(|feature|
            match getter(feature) {
                Some(mean) => Some(Cell::new(format!("{}", mean).as_str())),
                None => None
            }
        ).collect();
        Dataset::build_row(name, row)
    }

    pub fn compute(&mut self) {
        self.features.iter_mut().for_each(|feature| {
            feature.get_mean();
            feature.get_std();
            feature.get_min();
            feature.get_max();
            feature.get_quart();
            feature.get_median();
            feature.get_last_quart();
        });
    }

    pub fn display(&mut self) {
        let mut table = Table::new();

        self.compute();
        table.add_row( Dataset::build_row("Metrics", self.features.iter().filter_map(|feature| {
            match feature.content {
                FeatureContainer::Continuous(_) => Some(Cell::new(feature.name.as_str())),
                _ => None,
            } 
        }).collect()));
        table.add_row( self.metric_row("Mean", |feature| feature.mean) );
        table.add_row( self.metric_row("Std", |feature| feature.std) );
        table.add_row( self.metric_row("25%", |feature| feature.quart) );
        table.add_row( self.metric_row("50%", |feature| feature.median) );
        table.add_row( self.metric_row("75%", |feature| feature.last_quart) );
        table.add_row( self.metric_row("Min", |feature| feature.min) );
        table.add_row( self.metric_row("Max", |feature| feature.max) );

        println!("Continuous features :");
        table.printstd();
        println!("");

        let mut discreat_table = Table::new();
        discreat_table.add_row( Dataset::build_row("Metrics", self.features.iter().filter_map(|feature| {
            match feature.content {
                FeatureContainer::Discreat(_) => Some(Cell::new(feature.name.as_str())),
                _ => None,
            } 
        }).collect()));
        discreat_table.add_row( self.metric_row_usize("Number of different values", |feature| feature.discreat_number) );
        println!("Discreat features :");
        discreat_table.printstd();
    }
}
