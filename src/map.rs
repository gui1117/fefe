use entity::{FillableObject, InsertableObject, SegmentableObject};
use lyon::svg::parser::svg::ElementEnd::Close;
use lyon::svg::parser::svg::ElementEnd::Empty;
use lyon::svg::parser::svg::Name::Svg;
use lyon::svg::parser::svg::{Token, Tokenizer};
use lyon::svg::parser::FromSpan;
use lyon::svg::parser::{AttributeId, ElementId};
use lyon::svg::path::default::Path;
use rand::distributions::{Distribution, Weighted, WeightedChoice};
use rand::{thread_rng, Rng};
use specs::World;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub(crate) fn load_map(name: String, world: &mut World) -> Result<(), ::failure::Error> {
    ::util::reset_world(world);

    let mut path = PathBuf::from("data/maps");
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

    let mut insert_rules_entities = settings
        .insert_rules
        .iter()
        .map(|_| vec![])
        .collect::<Vec<_>>();
    let mut fill_rules_entities = settings
        .fill_rules
        .iter()
        .map(|_| vec![])
        .collect::<Vec<_>>();
    let mut segment_rules_entities = settings
        .segment_rules
        .iter()
        .map(|_| vec![])
        .collect::<Vec<_>>();

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
                        // Insert rules
                        for (rule, ref mut insert_rule_entities) in settings
                            .insert_rules
                            .iter()
                            .zip(insert_rules_entities.iter_mut())
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
                                insert_rule_entities.push(path);
                            }
                        }

                        // Fill rules
                        for (rule, ref mut fill_rule_entities) in settings
                            .fill_rules
                            .iter()
                            .zip(fill_rules_entities.iter_mut())
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
                                fill_rule_entities.push(path);
                            }
                        }

                        // Segment rules
                        for (rule, ref mut segment_rules_entities) in settings
                            .segment_rules
                            .iter()
                            .zip(segment_rules_entities.iter_mut())
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
                                segment_rules_entities.push(path);
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
    let mut insertables = world
        .read_resource::<::resource::Conf>()
        .insertables
        .clone();
    insertables.extend(settings.insertables);
    world.add_resource(insertables);

    for (insert_rule, insert_rule_entities) in settings
        .insert_rules
        .drain(..)
        .zip(insert_rules_entities.drain(..))
    {
        let rule_trigger = insert_rule.trigger;
        insert_rule
            .processor
            .build(insert_rule_entities, world)
            .map_err(|e| {
                format_err!(
                    "\"{}\": insert rule \"{}\": {}",
                    svg_path.to_string_lossy(),
                    rule_trigger,
                    e
                )
            })?;
    }

    // Fill entities to world
    let mut fillables = world.read_resource::<::resource::Conf>().fillables.clone();
    fillables.extend(settings.fillables);
    world.add_resource(fillables);

    for (fill_rule, fill_rule_entities) in settings
        .fill_rules
        .drain(..)
        .zip(fill_rules_entities.drain(..))
    {
        let rule_trigger = fill_rule.trigger;
        fill_rule
            .processor
            .build(fill_rule_entities, world)
            .map_err(|e| {
                format_err!(
                    "\"{}\": fill rule \"{}\": {}",
                    svg_path.to_string_lossy(),
                    rule_trigger,
                    e
                )
            })?;
    }

    // Segment entities to world
    let mut segmentables = world
        .read_resource::<::resource::Conf>()
        .segmentables
        .clone();
    segmentables.extend(settings.segmentables);
    world.add_resource(segmentables);

    for (segment_rule, segment_rule_entities) in settings
        .segment_rules
        .drain(..)
        .zip(segment_rules_entities.drain(..))
    {
        let rule_trigger = segment_rule.trigger;
        segment_rule
            .processor
            .build(segment_rule_entities, world)
            .map_err(|e| {
                format_err!(
                    "\"{}\": segment rule \"{}\": {}",
                    svg_path.to_string_lossy(),
                    rule_trigger,
                    e
                )
            })?;
    }

    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapSettings {
    pub insert_rules: Vec<Rule<InsertableObject>>,
    pub fill_rules: Vec<Rule<FillableObject>>,
    pub segment_rules: Vec<Rule<SegmentableObject>>,

    pub insertables: HashMap<String, InsertableObject>,
    pub fillables: HashMap<String, FillableObject>,
    pub segmentables: HashMap<String, SegmentableObject>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule<B> {
    pub trigger: String,
    pub processor: Processor<B>,
}

#[doc(hidden)]
pub trait TryFromPath: Sized {
    fn try_from_path(value: Path) -> Result<Self, ::failure::Error>;
}

#[doc(hidden)]
pub trait Builder {
    type Position: TryFromPath;
    fn build(&self, position: Self::Position, world: &mut World);
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
/// entities are randomized before being processed
pub enum Processor<B> {
    Build(String),
    TakeNPositions(usize, Box<Processor<B>>),
    RandomPositionDispatch(Vec<(u32, Box<Processor<B>>)>),
    OrdonatePositionDispatch(Vec<Box<Processor<B>>>),
    #[doc(hidden)]
    _Phantom(::std::marker::PhantomData<B>),
}

impl<B> Processor<B>
where B: Builder + 'static + Send + Sync + Clone
{
    fn build(
        self,
        entities: Vec<Path>,
        world: &mut World,
    ) -> Result<(), ::failure::Error> {
        let mut positions = vec![];
        for entity in entities {
            let position = B::Position::try_from_path(entity)
                .map_err(|e| format_err!("path incompatible with builder: {}", e))?;
            positions.push(position);
        }
        thread_rng().shuffle(&mut positions);
        self.build_positions(positions, world)
    }

    fn build_positions(
        self,
        mut positions: Vec<B::Position>,
        world: &mut World,
    ) -> Result<(), ::failure::Error> {
        use self::Processor::*;
        match self {
            Build(def_name) => {
                let def = world.read_resource::<HashMap<String, B>>().get(&def_name).cloned()
                    .ok_or(::failure::err_msg(format!("unknown entity: {}", def_name)))?;
                for position in positions {
                    def.build(position, world);
                }
                Ok(())
            }
            TakeNPositions(n, processor) => {
                positions.truncate(n);
                processor.build_positions(positions, world)
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

                for position in positions {
                    let i = choices.sample(&mut rng);
                    processors_entities[i].push(position);
                }
                for (_, processor) in weighted_processors {
                    processor.build_positions(processors_entities.remove(0), world)?
                }
                Ok(())
            }
            OrdonatePositionDispatch(inserters) => {
                let mut processors_entities = inserters.iter().map(|_| vec![]).collect::<Vec<_>>();
                let mut i = 0;
                for position in positions {
                    processors_entities[i].push(position);
                    i += 1;
                    i %= processors_entities.len();
                }
                for processor in inserters {
                    processor.build_positions(processors_entities.remove(0), world)?
                }
                Ok(())
            }
            _Phantom(_) => unreachable!(),
        }
    }
}
