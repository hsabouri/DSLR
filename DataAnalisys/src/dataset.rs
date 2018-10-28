#[derive(Clone)]
enum FeatureContainer {
    String(Vec<String>),
    Continuous(Vec<f32>),
}

struct Feature {
    name: String,
    content: FeatureContainer,
    count: usize,
    mean: Option<f32>,
    std: Option<f32>,
}

impl Feature {
    pub fn new(name: String, raw: Vec<String>) -> Feature {
        let count = raw.len();
        let try_parse: Vec<f32> = raw.iter().filter_map(
            |v| match v.parse::<f32>() {
                Ok(value) => Some(value),
                Err(_) => None
            }
        ).collect();

        let parsed = if try_parse.len() == count {
            FeatureContainer::Continuous(try_parse)
        } else {
            FeatureContainer::String(raw)
        };

        Feature {
            name: name,
            content: parsed,
            count: count,
            mean: None,
            std: None,
        }
    }

    pub fn get_mean(&mut self) -> Option<f32> {
        let mean = self.mean;
        let count = self.count as f32;
        let content = &self.content;

        self.mean = match (mean, content) {
            (Some(stored), _) => Some(stored),
            (None, FeatureContainer::Continuous(content)) =>
                Some(content.iter().fold(0.0, |acc, x| acc + x) / count),
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
                        .fold(
                            0.0,
                            |acc, x|
                                acc + (x - m).powi(2)
                        ) / count as f32
                ).sqrt()
            ),
            (_, _, _) => None,
        };
        self.std
    }
}

struct Dataset {
    feature: Vec<Feature>,
}

impl Dataset {
    
}
