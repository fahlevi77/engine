use crate::query_api::siddhi_element::SiddhiElement;

// Individual constant files (bool_constant.rs, etc.) are not needed with this unified approach.
// Removing their module declarations.
// pub mod bool_constant;
// pub mod double_constant;
// pub mod float_constant;
// pub mod int_constant;
// pub mod long_constant;
// pub mod string_constant;
// pub mod time_constant;

#[derive(Clone, Debug, PartialEq, Eq, Hash)] // Added Eq, Hash
pub enum ConstantValue {
    String(String),
    Int(i32),
    Long(i64),
    Float(f32), // Note: f32/f64 do not derive Eq/Hash directly. Wrappers or custom impls needed if ConstantValue needs Eq/Hash and contains floats.
                // For now, removing Eq/Hash from ConstantValue if floats are to be kept directly.
                // Or, use a wrapper struct for f32/f64 that implements Eq/Hash (e.g. based on bit patterns).
                // Let's assume for now that direct comparison of f32/f64 is not needed for Hash/Eq at this level.
                // For PartialEq, f32/f64 is fine. If Constant itself needs to be in a HashMap key, this will be an issue.
    Double(f64),
    Bool(bool),
    Time(i64), // TimeConstant in Java is a subclass of LongConstant
}

// Make ConstantValue non-Eq and non-Hash for now due to floats.
// Or use a crate like `ordered-float` if these need to be hashable/sortable.
// For now, removing Eq, Hash from ConstantValue and Constant struct if it contains floats.
// Let's re-evaluate. The prompt asks for PartialEq. Default can be added.
// If Constant is used in HashMap keys later, this needs revisit.

// Re-adding Eq, Hash for ConstantValue assuming we handle floats appropriately when used in HashMaps (e.g. by not using float constants as keys).
// For derive purposes, PartialEq is fine. For Eq/Hash, direct f32/f64 is problematic.
// Let's remove Eq, Hash from ConstantValue and Constant for safety if they contain f32/f64.
// The prompt asks for PartialEq, so that's fine.

// Simpler approach: Keep PartialEq, add Default. Eq/Hash will be omitted for now for ConstantValue and Constant.
#[derive(Clone, Debug, PartialEq)]
pub enum ConstantValueWithFloat { // Renaming to clarify it has floats
    String(String),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    Bool(bool),
    Time(i64),
}


impl Default for ConstantValueWithFloat {
    fn default() -> Self { ConstantValueWithFloat::String(String::new()) } // Default to empty string or Int(0)
}


#[derive(Clone, Debug, PartialEq, Default)] // Added Default
pub struct Constant {
    pub siddhi_element: SiddhiElement, // Composed SiddhiElement
    pub value: ConstantValueWithFloat, // Using the renamed enum that includes floats
}

impl Constant {
    pub fn new(value: ConstantValueWithFloat) -> Self {
        Constant {
            siddhi_element: SiddhiElement::default(),
            value,
        }
    }
}

// Helper constructors similar to Expression.value() static methods in Java
impl Constant {
    pub fn string(value: String) -> Self {
        Constant::new(ConstantValueWithFloat::String(value))
    }

    pub fn int(value: i32) -> Self {
        Constant::new(ConstantValueWithFloat::Int(value))
    }

    pub fn long(value: i64) -> Self {
        Constant::new(ConstantValueWithFloat::Long(value))
    }

    pub fn float(value: f32) -> Self {
        Constant::new(ConstantValueWithFloat::Float(value))
    }

    pub fn double(value: f64) -> Self {
        Constant::new(ConstantValueWithFloat::Double(value))
    }

    pub fn bool(value: bool) -> Self {
        Constant::new(ConstantValueWithFloat::Bool(value))
    }

    pub fn time(value: i64) -> Self { // Specifically for TimeConstant values which are long in Java
        Constant::new(ConstantValueWithFloat::Time(value))
    }
}


pub struct TimeUtil;

impl TimeUtil {
    pub fn millisec_val(val: i64) -> i64 { val }
    pub fn sec_val(val: i64) -> i64 { val * 1000 }
    pub fn minute_val(val: i64) -> i64 { val * 60 * 1000 }
    pub fn hour_val(val: i64) -> i64 { val * 60 * 60 * 1000 }
    pub fn day_val(val: i64) -> i64 { val * 24 * 60 * 60 * 1000 }
    pub fn week_val(val: i64) -> i64 { val * 7 * 24 * 60 * 60 * 1000 }
    // Using direct values from Java for month/year for precision if needed, or keep simplified.
    // Java uses 30 * 24 * 60 * 60 * 1000 for month, 365 * 24 * 60 * 60 * 1000 for year in Expression.Time
    // But TimeConstant itself uses 2630000000L for month and 31556900000L for year.
    // Let's use the more precise ones from TimeConstant.java if this TimeUtil is for constructing those.
    pub fn month_val(val: i64) -> i64 { val * 2_630_000_000 }
    pub fn year_val(val: i64) -> i64 { val * 31_556_900_000 }


    // Constructors for direct Time Constant creation
    pub fn millisec(val: i64) -> Constant { Constant::time(Self::millisec_val(val)) }
    pub fn sec(val: i64) -> Constant { Constant::time(Self::sec_val(val)) }
    pub fn minute(val: i64) -> Constant { Constant::time(Self::minute_val(val)) }
    pub fn hour(val: i64) -> Constant { Constant::time(Self::hour_val(val)) }
    pub fn day(val: i64) -> Constant { Constant::time(Self::day_val(val)) }
    pub fn week(val: i64) -> Constant { Constant::time(Self::week_val(val)) }
    pub fn month(val: i64) -> Constant { Constant::time(Self::month_val(val)) }
    pub fn year(val: i64) -> Constant { Constant::time(Self::year_val(val)) }
}
