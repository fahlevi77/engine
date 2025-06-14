use crate::core::event::complex_event::ComplexEvent;
use crate::core::event::value::AttributeValue;
use crate::core::executor::expression_executor::ExpressionExecutor;
use std::cmp::Ordering;

/// Comparator for events according to ORDER BY clauses.
#[derive(Debug)]
pub struct OrderByEventComparator {
    executors: Vec<Box<dyn ExpressionExecutor>>,
    ascending: Vec<bool>,
}

impl Clone for OrderByEventComparator {
    fn clone(&self) -> Self {
        Self {
            executors: Vec::new(),
            ascending: self.ascending.clone(),
        }
    }
}

impl OrderByEventComparator {
    pub fn new(executors: Vec<Box<dyn ExpressionExecutor>>, ascending: Vec<bool>) -> Self {
        Self {
            executors,
            ascending,
        }
    }

    pub fn compare(&self, ev1: &dyn ComplexEvent, ev2: &dyn ComplexEvent) -> Ordering {
        for (exec, asc) in self.executors.iter().zip(self.ascending.iter()) {
            let v1 = exec.execute(Some(ev1));
            let v2 = exec.execute(Some(ev2));
            let ord = match (v1, v2) {
                (Some(a), Some(b)) => compare_attr_values(&a, &b),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (None, None) => Ordering::Equal,
            };
            if ord != Ordering::Equal {
                return if *asc { ord } else { ord.reverse() };
            }
        }
        Ordering::Equal
    }
}

fn compare_attr_values(a: &AttributeValue, b: &AttributeValue) -> Ordering {
    match (a, b) {
        (AttributeValue::String(x), AttributeValue::String(y)) => x.cmp(y),
        (AttributeValue::Int(x), AttributeValue::Int(y)) => x.cmp(y),
        (AttributeValue::Long(x), AttributeValue::Long(y)) => x.cmp(y),
        (AttributeValue::Float(x), AttributeValue::Float(y)) => {
            x.partial_cmp(y).unwrap_or(Ordering::Equal)
        }
        (AttributeValue::Double(x), AttributeValue::Double(y)) => {
            x.partial_cmp(y).unwrap_or(Ordering::Equal)
        }
        (AttributeValue::Bool(x), AttributeValue::Bool(y)) => x.cmp(y),
        _ => a.to_string().cmp(&b.to_string()),
    }
}
