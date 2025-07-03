use clap::{ArgMatches, Parser};
use polars::prelude::*;
use std::net::{IpAddr, Ipv6Addr};

use crate::Plugin;

#[macro_export]
macro_rules! plugin {
    (#[plugin(name = $name:expr, description = $desc:expr)] $($rest:tt)*) => {
        $($rest)*

        impl PluginInfo for Self {
            const NAME: &'static str = $name;
            const DESCRIPTION: &'static str = $desc;
        }
    };
}

pub fn test() {
    let series = Series::new("a".into(), &[1i32, 2, 3]);
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

    let field = Field::new("b".into(), DataType::Int32);
    let schema = Schema::from_iter(vec![field]);
}

pub trait PluginInfo {
    const NAME: &'static str;
    const DESCRIPTION: &'static str;
}

pub trait Source: PluginInfo + Send + Sync {
    type Item;
    fn stream(&self) -> impl Iterator<Item = Self::Item>;
}

pub trait Sink: PluginInfo + Send + Sync {
    type Item;
    fn sink(&self, item: Self::Item);
}

pub trait Transform: PluginInfo + Send + Sync {
    type In;
    type Out;
    fn transform(&self, x: Self::In) -> Self::Out;
}

pub trait Predicate: PluginInfo + Send + Sync {
    type In;
    fn predicate(&self, x: Self::In) -> bool;
}

trait Aggregate: PluginInfo + Send + Sync {
    type Item;
    type Out;
    fn absorb(&mut self, item: Self::Item);
    fn aggregate(&self) -> Self::Out;
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
