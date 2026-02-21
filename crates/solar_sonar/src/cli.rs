use crate::*;

#[derive(Debug, Clone, clap::Parser)]
pub struct Cli {
    /// Characters to watch intel for
    pub watch_characters: String,
    /// Systems to watch
    pub watch_systems: String,
    /// Jump range of systems
    #[clap(default_value_t = 2)]
    pub jumps: u8,
}

#[derive(Debug, Clone)]
pub struct Args {
    pub watch_character_ids: Vec<CharacterID>,
    pub watch_system_ids: Vec<SolarID>,
    pub jumps: u8,
}

impl Args {
    pub(crate) fn try_from_cli(cli: Cli, cfg: &Config) -> SolarResult<Self> {
        let jumps = cli.jumps;
        let watch_character_ids = cli.watch_characters.split(',')
            .map(|snake| snake.to_snake_case())
            .map(|snake| cfg.characters.iter()
                .find(|chr| chr.alias == snake)
                .map(|chr| chr.id)
                .ok_or_else(|| SolarError::msg(format!("Character unknown: {snake}")))
            )
            .collect::<SolarResult<Vec<_>>>()?;

        let watch_system_ids = cli.watch_systems.split(',')
            .map(|name| name.to_uppercase())
            .map(|ref name| STAR_MAP.get_system_named(name)
                .map(|sys| sys.id)
                .ok_or_else(|| SolarError::msg(format!("System unknown: {name}")))
            )
            .collect::<SolarResult<Vec<_>>>()?;

        Ok(Self { watch_character_ids, watch_system_ids, jumps })
    }

    pub(crate) fn watch_characters<'a>(&self, cfg: &'a Config) -> Vec<&'a CharacterConfig> {
        self.watch_character_ids.iter()
            .filter_map(|id| cfg.characters.iter().find(|chr| &chr.id == id))
            .collect()
    }

    pub(crate) fn watch_systems(&self) -> Vec<&'static SolarSystem > {
        self.watch_system_ids.iter()
            .map(|id| STAR_MAP.system(id))
            .collect()
    }
}

