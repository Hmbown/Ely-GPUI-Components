mod body;
mod cell;
mod model;
mod simple;
mod table;
#[cfg(test)]
mod tests;

pub use cell::{Aggregate, Align, Cell, Column};
pub use simple::{HeatmapTable, Table};
pub use table::{DataTable, Row};
