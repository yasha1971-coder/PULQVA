use arti::ArtiCombinedConfig;
use tor_config::{
    ConfigurationSource, ConfigurationSources,
    sources::MustRead,
};

fn main() {
    let config = include_str!("../../../sidecars/arti/pulqva.toml");

    let mut sources = ConfigurationSources::new_empty();
    sources.push_source(
        ConfigurationSource::from_verbatim(config.to_owned()),
        MustRead::MustRead,
    );

    let tree = sources.load().expect("PULQVA Arti config must load as TOML");
    let _: ArtiCombinedConfig =
        tor_config::resolve(tree).expect("PULQVA Arti config must resolve under Arti 2.6.0");

    println!("PULQVA_ARTI_CONFIG_OK");
}
