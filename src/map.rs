// TODO 3 rule
// filled
// launched
// inserted
use rand::distributions::{IndependentSample, Weighted, WeightedChoice};
use std::fs::File;
use std::io::Read;
use lyon::svg::parser::FromSpan;
use lyon::svg::parser::svg::{Token, Tokenizer};
use lyon::svg::parser::{AttributeId, ElementId};
use lyon::svg::parser::svg::Name::Svg;
use lyon::svg::parser::svg::ElementEnd::Close;
use lyon::svg::parser::svg::ElementEnd::Empty;
use lyon::svg::path::default::Path;
use entity::InsertableObject;

pub fn load_map(name: String, world: &mut ::specs::World) -> Result<(), ::failure::Error> {
    let mut path = ::CFG.map_directory.clone();
    path.push(name);
    if !path.is_dir() {
        return Err(format_err!(
            "\"{}\": does not exist",
            path.to_string_lossy()
        ));
    }

    let mut settings_path = path.clone();
    settings_path.push("settings.ron");
    let settings_file = File::open(&settings_path)
        .map_err(|e| format_err!("\"{}\": {}", settings_path.to_string_lossy(), e))?;
    let mut settings: MapSettings = ::ron::de::from_reader(settings_file)
        .map_err(|e| format_err!("\"{}\": {}", settings_path.to_string_lossy(), e))?;

    let mut svg_path = path.clone();
    svg_path.push("map.svg");
    let mut svg_file = File::open(&svg_path)
        .map_err(|e| format_err!("\"{}\": {}", svg_path.to_string_lossy(), e))?;
    let mut svg_string = String::new();
    svg_file
        .read_to_string(&mut svg_string)
        .map_err(|e| format_err!("\"{}\": {}", svg_path.to_string_lossy(), e))?;

    let mut rules_entities = settings.rules.iter().map(|_| vec![]).collect::<Vec<_>>();

    let mut tokenizer = Tokenizer::from_str(&svg_string);

    let mut in_marker = false;
    let mut style = None;
    let mut d = None;
    let mut in_path_attribute = false;
    while let Some(token) = tokenizer.next() {
        let token = token.map_err(|e| format_err!("\"{}\": {}", svg_path.to_string_lossy(), e))?;

        // Ignore markers
        if in_marker {
            match token {
                Token::ElementEnd(Close(Svg(ElementId::Marker))) => {
                    in_marker = false;
                }
                _ => (),
            }
        // Process path attribute
        } else if in_path_attribute {
            match token {
                Token::Attribute(Svg(AttributeId::Style), value) => {
                    style = Some(value);
                }
                Token::Attribute(Svg(AttributeId::D), value) => {
                    d = Some(value);
                }
                Token::ElementEnd(Empty) => {
                    if let (Some(style), Some(d)) = (style, d) {
                        for (rule, ref mut rule_entities) in
                            settings.rules.iter().zip(rules_entities.iter_mut())
                        {
                            if style.to_str().contains(&rule.trigger) {
                                let svg_builder = Path::builder().with_svg();
                                let commands = d.to_str();
                                let path =
                                    ::lyon::svg::path_utils::build_path(svg_builder, commands)
                                        .map_err(|e| {
                                            format_err!(
                                                "\"{}\": invalid path \"{}\": {:?}",
                                                svg_path.to_string_lossy(),
                                                commands,
                                                e
                                            )
                                        })?;
                                rule_entities.push(path);
                            }
                        }
                    }
                    in_path_attribute = false;
                }
                _ => (),
            }
        } else {
            match token {
                Token::ElementStart(Svg(ElementId::Marker)) => {
                    in_marker = true;
                }
                Token::ElementStart(Svg(ElementId::Path)) => {
                    in_path_attribute = true;
                    style = None;
                    d = None;
                }
                _ => (),
            }
        }
    }

    // Insert entities to world
    for (rule, rule_entities) in settings.rules.drain(..).zip(rules_entities.drain(..)) {
        let rule_trigger = rule.trigger;
        rule.processor.build(rule_entities, world).map_err(|e| {
            format_err!(
                "\"{}\": rule \"{}\": {}",
                svg_path.to_string_lossy(),
                rule_trigger,
                e
            )
        })?;
    }
    Ok(())
}

#[derive(Serialize, Deserialize)]
struct MapSettings {
    rules: Vec<Rule>,
}

#[derive(Serialize, Deserialize)]
struct Rule {
    trigger: String,
    processor: Processor<InsertableObject>,
}

pub trait TryFromPath: Sized {
    fn try_from_path(value: Path) -> Result<Self, ::failure::Error>;
}

pub trait Builder {
    type Position: TryFromPath;
    fn build(&self, position: Self::Position, world: &mut ::specs::World);
}

#[derive(Serialize, Deserialize)]
/// entities are randomized before being processed
enum Processor<B: Builder> {
    BuildEntity(B),
    TakeNPositions(usize, Box<Processor<B>>),
    RandomPositionDispatch(Vec<(u32, Box<Processor<B>>)>),
    OrdonatePositionDispatch(Vec<Box<Processor<B>>>),
}

impl<B: Builder> Processor<B> {
    fn build(
        self,
        entities: Vec<Path>,
        world: &mut ::specs::World,
    ) -> Result<(), ::failure::Error> {
        let mut positions = vec![];
        for entity in entities {
            let position = B::Position::try_from_path(entity)
                .map_err(|e| format_err!("path incompatible with builder: {}", e))?;
            positions.push(position);
        }
        self.build_positions(positions, world);
        Ok(())
    }

    fn build_positions(self, mut entities: Vec<B::Position>, world: &mut ::specs::World) {
        use self::Processor::*;
        match self {
            BuildEntity(builder) => for entity in entities {
                builder.build(entity, world);
            },
            TakeNPositions(n, processor) => {
                entities.truncate(n);
                processor.build_positions(entities, world);
            }
            RandomPositionDispatch(weighted_processors) => {
                let mut rng = ::rand::thread_rng();
                let mut processors_entities = weighted_processors
                    .iter()
                    .map(|_| vec![])
                    .collect::<Vec<_>>();
                let mut items = weighted_processors
                    .iter()
                    .enumerate()
                    .map(|(item, &(weight, _))| Weighted { weight, item })
                    .collect::<Vec<_>>();
                let choices = WeightedChoice::new(&mut items);

                for entity in entities {
                    let i = choices.ind_sample(&mut rng);
                    processors_entities[i].push(entity);
                }
                for (_, processor) in weighted_processors {
                    processor.build_positions(processors_entities.remove(0), world)
                }
            }
            OrdonatePositionDispatch(inserters) => {
                let mut processors_entities = inserters.iter().map(|_| vec![]).collect::<Vec<_>>();
                let mut i = 0;
                for entity in entities {
                    processors_entities[i].push(entity);
                    i += 1;
                    i %= processors_entities.len();
                }
                for processor in inserters {
                    processor.build_positions(processors_entities.remove(0), world)
                }
            }
        }
    }
}
