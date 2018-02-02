use std::fs::File;

const FILENAME: &str = "configuration.ron";

lazy_static! {
    pub static ref CFG: Configuration = {
        let file = File::open(FILENAME).unwrap();
        ::ron::de::from_reader(file).unwrap()
    };
}

#[derive(Serialize, Deserialize)]
pub struct Configuration {
    pub animation: ::animation::AnimationsCfg,
}
