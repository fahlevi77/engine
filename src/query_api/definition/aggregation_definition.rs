// Corresponds to io.siddhi.query.api.definition.AggregationDefinition
use crate::query_api::definition::abstract_definition::AbstractDefinition;
use crate::query_api::annotation::Annotation;
// BasicSingleInputStream was merged into SingleInputStream; AggregationDefinition should use SingleInputStream.
use crate::query_api::execution::query::input::stream::SingleInputStream;
use crate::query_api::execution::query::selection::Selector;
use crate::query_api::expression::Variable;
use crate::query_api::aggregation::TimePeriod; // Using actual TimePeriod


#[derive(Clone, Debug, PartialEq, Default)] // Added Default
pub struct AggregationDefinition {
    // Composition for inheritance from AbstractDefinition
    pub abstract_definition: AbstractDefinition,

    // Fields specific to AggregationDefinition
    pub basic_single_input_stream: Option<SingleInputStream>, // Changed from BasicSingleInputStream
    pub selector: Option<Selector>, // In Java, it's Selector, not BasicSelector for the field type
    pub aggregate_attribute: Option<Variable>, // This is 'aggregateBy' in Java
    pub time_period: Option<TimePeriod>,     // This is 'every' in Java

    // annotations are in AbstractDefinition
}

impl AggregationDefinition {
    // Constructor that takes an id, as per Java's `AggregationDefinition.id(aggregationName)`
    pub fn new(id: String) -> Self {
        AggregationDefinition {
            abstract_definition: AbstractDefinition::new(id),
            basic_single_input_stream: None,
            selector: None, // Java initializes selector to null
            aggregate_attribute: None,
            time_period: None,
        }
    }

    // Static factory `id` from Java
    pub fn id(aggregation_name: String) -> Self {
        Self::new(aggregation_name)
    }

    // Builder-style methods from Java
    pub fn from(mut self, stream: SingleInputStream) -> Self { // Changed from BasicSingleInputStream
        self.basic_single_input_stream = Some(stream);
        self
    }

    // In Java, select takes BasicSelector, but field is Selector.
    // Using Selector type here.
    pub fn select(mut self, selector: Selector) -> Self {
        self.selector = Some(selector);
        self
    }

    pub fn aggregate_by(mut self, var: Variable) -> Self {
        self.aggregate_attribute = Some(var);
        self
    }

    pub fn every(mut self, period: TimePeriod) -> Self {
        self.time_period = Some(period);
        self
    }

    pub fn annotation(mut self, annotation: Annotation) -> Self {
        self.abstract_definition.annotations.push(annotation);
        self
    }
}

// Provide access to AbstractDefinition fields and SiddhiElement fields
impl AsRef<AbstractDefinition> for AggregationDefinition {
    fn as_ref(&self) -> &AbstractDefinition {
        &self.abstract_definition
    }
}

impl AsMut<AbstractDefinition> for AggregationDefinition {
    fn as_mut(&mut self) -> &mut AbstractDefinition {
        &mut self.abstract_definition
    }
}

use crate::query_api::siddhi_element::SiddhiElement;
impl AsRef<SiddhiElement> for AggregationDefinition {
    fn as_ref(&self) -> &SiddhiElement {
        self.abstract_definition.as_ref()
    }
}

impl AsMut<SiddhiElement> for AggregationDefinition {
    fn as_mut(&mut self) -> &mut SiddhiElement {
        self.abstract_definition.as_mut()
    }
}
