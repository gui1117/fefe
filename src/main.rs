extern crate specs;
extern crate svgparser;
extern crate rand;
extern crate ron;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure;

use rand::distributions::{IndependentSample, Weighted, WeightedChoice};
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;
use svgparser::FromSpan;
use svgparser::svg::{Tokenizer, Token};
use svgparser::{ElementId, AttributeId};

fn load_map(path: PathBuf) -> Result<(), failure::Error> {
    let mut settings_path = path.clone();
    settings_path.set_extension("ron");
    let settings_file = File::open(settings_path)?;
    let settings: MapSettings = ron::de::from_reader(settings_file)?;

    let mut map_path = path.clone();
    map_path.set_extension("svg");
    let mut map_file = File::open(map_path)?;
    let mut map_string = String::new();
    map_file.read_to_string(&mut map_string)?;

    let mut rules_entities = settings.rules.iter().map(|_| vec![]).collect::<Vec<_>>();

    let mut tokenizer = Tokenizer::from_str(&map_string);
    while let Some(token) = tokenizer.next() {
        let token = token.map_err(|e| format_err!("{}", e))?;
        use svgparser::svg::Name::Svg;
        match token {
            Token::ElementStart(Svg(ElementId::Path)) => {
                let mut style = None;
                let mut d = None;
                while let Some(Ok(Token::Attribute(attribute, value))) = tokenizer.next() {
                    match attribute {
                        Svg(AttributeId::Style) => style = Some(value),
                        Svg(AttributeId::D) => d = Some(value),
                        _ => (),
                    }
                }
                if let (Some(style), Some(d)) = (style, d) {
                    for (rule, ref mut entities) in settings.rules.iter().zip(rules_entities.iter_mut()) {
                        if style.to_str().contains(&rule.trigger) {
                            entities.push(d.to_str());
                        }
                    }
                }
            }
            _ => (),
        }
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
enum EntitySettings {
    Monster1 {
        size: f32,
        attack: f32,
    },
    Monster2 {
        size: f32,
        attack: f32,
    },
}

#[derive(Serialize, Deserialize)]
struct MapSettings {
    rules: Vec<Rule>,
}

#[derive(Serialize, Deserialize)]
struct Rule {
    trigger: String,
    inserter: Inserter,
}

struct Isometry2;

#[derive(Serialize, Deserialize)]
/// entities are randomized before being processed
enum Inserter {
    InsertEntity(EntitySettings),
    TakeNInsertion(usize, Box<Inserter>),
    RandomInsertionDispatch(Vec<(u32, Box<Inserter>)>),
    OrdonateInsertionDispatch(Vec<Box<Inserter>>),
}

impl Inserter {
    fn insert(self, mut entities: Vec<Isometry2>, world: &mut ::specs::World) {
        use Inserter::*;
        match self {
            InsertEntity(entity_settings) => {
                unimplemented!();
            },
            TakeNInsertion(n, inserter) => {
                entities.truncate(n);
                inserter.insert(entities, world);
            },
            RandomInsertionDispatch(weighted_inserters) => {
                let mut rng = rand::thread_rng();
                let mut inserters_entities = weighted_inserters.iter().map(|_| vec![]).collect::<Vec<_>>();
                let mut items = weighted_inserters.iter()
                    .enumerate()
                    .map(|(item, &(weight, _))| Weighted { weight, item })
                    .collect::<Vec<_>>();
                let choices = WeightedChoice::new(&mut items);

                for entity in entities {
                    let i = choices.ind_sample(&mut rng);
                    inserters_entities[i].push(entity);
                }
                for (_, inserter) in weighted_inserters {
                    inserter.insert(inserters_entities.remove(0), world)
                }
            },
            OrdonateInsertionDispatch(inserters) => {
                let mut inserters_entities = inserters.iter().map(|_| vec![]).collect::<Vec<_>>();
                let mut i = 0;
                for entity in entities {
                    inserters_entities[i].push(entity);
                    i += 1;
                    i %= inserters_entities.len();
                }
                for inserter in inserters {
                    inserter.insert(inserters_entities.remove(0), world)
                }
            },
        }
    }
}

fn main() {
    if let Err(err) = load_map("map".into()) {
        println!("{}", err);
    }
}
