use clap::ValueEnum;

pub mod import_collection;
pub mod import_digital_roster;
pub mod import_track;
pub mod import_wishlist;
pub mod info;
pub mod validate;
pub mod validation;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SchemaType {
    Collection,
    Wishlist,
    DigitalRoster,
    Track,
    Manifest,
}
