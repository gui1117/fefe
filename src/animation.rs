use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use std::ffi::OsStr;

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum Framerate {
    /// Distance for one loop
    Walk(f32),
    /// Image per second
    Fix(f32),
}

#[derive(Serialize, Deserialize)]
pub struct AnimationsCfg {
    table: HashMap<(AnimationSpecie, AnimationName), Vec<String>>,
    parts: HashMap<String, AnimationPartCfg>,
    directory: PathBuf,
}

#[derive(Serialize, Deserialize)]
pub struct AnimationPartCfg {
    filename: String,
    layer: f32,
    framerate: Framerate,
}

lazy_static! {
    pub static ref ANIMATIONS: Animations = Animations::load().unwrap();
}

pub struct Animations {
    images: Vec<PathBuf>,
    table: HashMap<(AnimationSpecie, AnimationName), Vec<AnimationPart>>,
}

impl Animations {
    fn load() -> Result<Animations, ::failure::Error> {
        let mut parts_table = HashMap::new();
        let mut images = vec![];

        let mut dir_entries = vec![];
        for entry in fs::read_dir(&::CFG.animation.directory)
            .map_err(|e| format_err!("read dir \"{}\": {}", ::CFG.animation.directory.to_string_lossy(), e))?
        {
            let entry = entry
                .map_err(|e| format_err!("read dir \"{}\": {}", ::CFG.animation.directory.to_string_lossy(), e))?
                .path();

            if entry.extension().iter().any(|p| *p == OsStr::new("png")) {
                dir_entries.push(entry);
            }
        }

        for (part_name, part) in &::CFG.animation.parts {
            let mut part_images = dir_entries.iter()
                .filter(|p| {
                    if let Some(stem) = p.file_stem() {
                        let len = stem.len();
                        let stem_string = stem.to_string_lossy();
                        let (name, _number) = stem_string.split_at(len-4);
                        name == part_name
                    } else {
                        false
                    }
                })
                .cloned()
                .collect::<Vec<_>>();
            part_images.sort();

            parts_table.insert(part_name, AnimationPart {
                framerate: part.framerate,
                layer: part.layer,
                images: part_images.iter().enumerate().map(|(i, _)| i+images.len()).collect(),
            });

            images.append(&mut part_images);
        }

        let mut table = HashMap::new();

        for (&key, part_names) in &::CFG.animation.table {
            let mut parts = vec![];
            for part_name in part_names {
                let part = parts_table.get(&part_name)
                    .ok_or(format_err!("invalid animation configuration: \"{}\" does not correspond to any animation part", part_name))?;
                parts.push(part.clone());
            }
            table.insert(key, parts);
        }

        Ok(Animations {
            images,
            table,
        })
    }
}

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Clone, Copy)]
pub enum AnimationName {
    ShootRifle,
    IdleRifle,
    TakeRifle,
    UntakeRifle,
}

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Clone, Copy)]
pub enum AnimationSpecie {
    Character,
    Monster,
}

#[derive(Clone)]
pub struct AnimationPart {
    layer: f32,
    framerate: Framerate,
    images: Vec<usize>,
}

pub struct AnimationState {
    /// 0 is no walk
    walk_distance: f32,
    specie: AnimationSpecie,
    idle_animation: Vec<AnimationPart>,
    animations: Vec<Vec<AnimationPart>>,
    timer: f32,
}
