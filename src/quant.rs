pub mod bonds;
pub mod options;
pub mod r#trait;
pub mod volatility;
pub mod yahoo;

/// Option type.
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum OptionType {
  #[default]
  Call,
  Put,
}
