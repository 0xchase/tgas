use polars::prelude::*;
use clap::{ArgMatches, Parser};
use std::net::{IpAddr, Ipv6Addr};

use crate::Plugin;

/* Plugin contracts to make */
// transform nothing into a series (download ips, generate ips, etc)
// transform nothing into tabular data (scan)
// transform tabular data into visual data (plot, etc)
// transform tabular data into tabular data (filter, etc)
// map tabular data into tabular data (enrich, etc)
// map tabular data into list data (enrich, etc)
// enrich specific fields (ip, domain, etc)

// acquire: scan, download, locate, etc
// enrich: lookup, locate, etc
// process: transform, filter, etc
// analyze: counts, entropy, dispersion, subnets, graphs, etc
// model: generate, train, etc
// serve
// report

pub fn test() {
    let series = Series::new("a".into(), &[1i32, 2, 3]);
    // let df = DataFrame::new(vec![series.into()]).unwrap();
    let data = series.strict_cast(&DataType::Int32).unwrap();

    let mut frame = series.into_frame();

    let column_1 = Column::new("b".into(), &[1i32, 2, 3]);
    let column_2 = Column::new("b".into(), &[1i32, 2, 3]);
    let column_3 = Column::new("b".into(), &[1i32, 2, 3]);

    let mut dataframe = DataFrame::empty()
        .with_column(column_1)
        .unwrap()
        .with_column(column_2)
        .unwrap()
        .with_column(column_3)
        .unwrap()
        .to_owned();

    /*let d = dataframe
        .clone()
        .lazy()
        // optional flag to select a column
        .with_column(col("b").into())
        .collect()
        .unwrap();*/

    // dataframe.replace_or_add("b".into(), data).unwrap();

    let field = Field::new("b".into(), DataType::Int32);
    let schema = Schema::from_iter(vec![field]);
}

fn for_array<T: PolarsDataType>(array: ChunkedArray<T>) {
}

fn for_series<T: Into<IpAddr>, I: Iterator<Item = T>>(iter: I) {
    for item in iter {
        let ip = item.into();
        println!("{}", ip);
    }
}

pub trait Categorizer: Plugin<Series, Series> {
    const CATEGORIES: &'static [&'static str];
}

/// A plugin that generates data from nothing
pub trait Generator: Plugin<(), Series> {
    /// The name of the series to generate
    const SERIES_NAME: &'static str;
}

pub trait MyField {
    const FIELD_NAME: &'static str;
    const FIELD_TYPE: &'static DataType;

    fn from_any_value(any_value: AnyValue) -> Self;
    fn to_any_value(&self) -> AnyValue;
}

impl MyField for Ipv6Addr {
    const FIELD_NAME: &'static str = "ipv6";
    const FIELD_TYPE: &'static DataType = &DataType::String;

    fn from_any_value(any_value: AnyValue) -> Self {
        todo!()
    }

    fn to_any_value(&self) -> AnyValue {
        todo!()
    }
}

pub trait AbsorbField<T: MyField> {
    type Config;

    fn absorb(&mut self, item: T);
    fn finalize(&mut self) -> DataFrame;

    fn absorb_series(&mut self, series: &Series) -> DataFrame {
        for item in series.iter() {
            let item = item.cast(&T::FIELD_TYPE);
            let item = T::from_any_value(item);
            self.absorb(item);
        }

        self.finalize()
    }
}
