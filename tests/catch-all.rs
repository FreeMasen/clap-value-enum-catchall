use clap::Parser;
use clap_value_enum_catchall::ValueEnumCatchall;
use uuid::Uuid;

#[derive(Debug, Parser)]
pub struct UuidArgs {
    arg1: String,
    #[clap(long, short)]
    arg2: UuidEnum,
}

#[derive(Debug, Clone, ValueEnumCatchall)]
pub enum UuidEnum {
    One,
    Two(Uuid),
}

#[test]
fn uuid_arg() {
    UuidArgs::try_parse_from(["", "arg1", "--arg2", "36fb2016-e6b6-473a-9166-f9a63d2b72c2"]).unwrap();
}

#[test]
#[should_panic = "<uuid>"]
fn uuid_panic() {
    UuidArgs::try_parse_from(["", "arg1", "--arg2", "junk"]).unwrap();
}

#[derive(Debug, Parser)]
pub struct StringArgs {
    arg1: String,
    #[clap(long, short)]
    arg2: StringEnum,
}

#[derive(Debug, Clone, ValueEnumCatchall)]
pub enum StringEnum {
    One,
    Two(String),
}

#[test]
fn string_arg() {
    StringArgs::try_parse_from(["", "arg1", "--arg2", "36fb2016-e6b6-473a-9166-f9a63d2b72c2"]).unwrap();
}

#[test]
#[should_panic = "<string>"]
fn string_panic() {
    StringArgs::try_parse_from(["", "arg1", "--arg2"]).unwrap();
}

#[derive(Debug, Parser)]
pub struct U32Args {
    arg1: String,
    #[clap(long, short)]
    arg2: U32Enum,
}

#[derive(Debug, Clone, ValueEnumCatchall)]
pub enum U32Enum {
    One,
    Two(u32),
}

#[test]
fn u32_arg() {
    U32Args::try_parse_from(["", "arg1", "--arg2", "1"]).unwrap();
}

#[test]
#[should_panic = "<u32>"]
fn u32_panic() {
    U32Args::try_parse_from(["", "arg1", "--arg2"]).unwrap();
}
