// schema_defs.rs  (shared by host + plugins)
use arrow_array::{FixedSizeBinaryArray, Float64Array};
use arrow_schema::DataType;
use arrow_struct::ArrowStruct;
use crate::{FieldSpec};

// ① zero-sized typestate markers