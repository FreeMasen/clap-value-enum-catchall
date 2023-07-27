use clap::{Parser, ValueEnum};
use clap_value_enum_catchall::ValueEnumCatchall;
use uuid::Uuid;

#[derive(Debug, Parser)]
pub struct UuidArgs {
    #[clap(long, short)]
    one: UuidEnum,
    #[clap(value_enum, long, short)]
    two: UnitEnum,
}
#[derive(Debug, Clone, ValueEnum)]
pub enum UnitEnum {
    One,
    Two,
}

#[derive(Debug, Clone, ValueEnumCatchall)]
#[catchall(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum UuidEnum {
    One,
    Two(Uuid),
}

fn main() {
    UuidArgs::parse();
}
