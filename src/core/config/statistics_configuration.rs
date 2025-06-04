// Corresponds to io.siddhi.core.config.StatisticsConfiguration

// Placeholder for StatisticsTrackerFactory
// This would be a trait or struct related to metrics.
#[derive(Clone, Debug, Default)]
pub struct StatisticsTrackerFactoryPlaceholder {
    _dummy: String,
}

#[derive(Clone, Debug)] // Default can be custom
pub struct StatisticsConfiguration {
    // StatisticsConfiguration in Java is not a SiddhiElement, so no siddhi_element field.
    pub metric_prefix: String,
    pub factory: StatisticsTrackerFactoryPlaceholder, // Placeholder for actual factory type
}

impl StatisticsConfiguration {
    // Constructor takes the factory, metric_prefix has a default in Java.
    pub fn new(factory: StatisticsTrackerFactoryPlaceholder) -> Self {
        Self {
            metric_prefix: "io.siddhi".to_string(), // Default from Java
            factory,
        }
    }

    /// Returns the statistics tracker factory associated with this configuration.
    pub fn get_factory(&self) -> &StatisticsTrackerFactoryPlaceholder {
        &self.factory
    }

    /// Returns the configured metric prefix.
    pub fn get_metric_prefix(&self) -> &str {
        &self.metric_prefix
    }

    /// Sets the metric prefix. Mirrors the Java setter.
    pub fn set_metric_prefix(&mut self, metric_prefix: String) {
        self.metric_prefix = metric_prefix;
    }
}

impl Default for StatisticsConfiguration {
    fn default() -> Self {
        Self {
            metric_prefix: "io.siddhi".to_string(),
            factory: StatisticsTrackerFactoryPlaceholder::default(),
        }
    }
}
