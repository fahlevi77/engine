use crate::query_api::siddhi_element::SiddhiElement;
// Expression is not directly part of TimePeriod fields, but Duration might be associated with values if not just enum.
// However, Java's TimePeriod.Duration is just an enum.

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub enum Duration {
    Seconds,
    Minutes,
    Hours,
    Days,
    Months,
    Years,
}

impl Default for Duration {
    fn default() -> Self { Duration::Seconds }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Copy)]
pub enum Operator {
    Range,
    Interval,
}

impl Default for Operator {
    fn default() -> Self { Operator::Interval } // Or Range, depending on common usage
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct TimePeriod {
    pub siddhi_element: SiddhiElement,
    pub operator: Operator,
    pub durations: Vec<Duration>,
}

impl TimePeriod {
    // Private constructor as in Java, use factory methods.
    fn new(operator: Operator, durations: Vec<Duration>) -> Self {
        Self {
            siddhi_element: SiddhiElement::default(),
            operator,
            durations,
        }
    }

    // Corresponds to TimePeriod.range(Duration begging, Duration end)
    pub fn range(start: Duration, end: Duration) -> Self {
        Self::new(Operator::Range, vec![start, end])
    }

    // Corresponds to TimePeriod.interval(Duration... durations)
    pub fn interval(durations: Vec<Duration>) -> Self {
        // Java uses varargs, which implies at least one.
        // An empty Vec<Duration> for interval might be valid or not depending on Siddhi logic.
        Self::new(Operator::Interval, durations)
    }
}
