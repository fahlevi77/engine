//! StreamJunction Factory for Performance-Optimized Event Routing
//!
//! Provides intelligent selection between original crossbeam_channel-based
//! implementation and new high-performance crossbeam pipeline-based implementation.

use crate::core::config::siddhi_app_context::SiddhiAppContext;
use crate::core::stream::{OptimizedStreamJunction, StreamJunction};
use crate::query_api::definition::StreamDefinition;
use std::sync::{Arc, Mutex};

/// Performance optimization levels for StreamJunction selection
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PerformanceLevel {
    /// Use original crossbeam_channel implementation
    Standard,
    /// Use optimized crossbeam pipeline implementation  
    HighPerformance,
    /// Automatically select based on workload characteristics
    Auto,
}

impl Default for PerformanceLevel {
    fn default() -> Self {
        PerformanceLevel::Auto
    }
}

/// Configuration for StreamJunction creation
#[derive(Debug, Clone)]
pub struct JunctionConfig {
    pub stream_id: String,
    pub buffer_size: usize,
    pub is_async: bool,
    pub performance_level: PerformanceLevel,
    pub expected_throughput: Option<u64>, // events/second
    pub subscriber_count: Option<usize>,
}

impl JunctionConfig {
    /// Create a new junction configuration with synchronous processing by default
    ///
    /// **Default Mode: Synchronous (is_async: false)**
    /// - Guarantees strict event ordering
    /// - Events are processed sequentially in the order they arrive
    /// - Suitable for scenarios where event order is critical
    ///
    /// **Async Mode Option: Use with_async(true) for high-throughput scenarios**
    /// - Trades event ordering guarantees for higher performance
    /// - Events may be processed out of order due to concurrent processing
    /// - Suitable for scenarios where throughput > ordering
    pub fn new(stream_id: String) -> Self {
        Self {
            stream_id,
            buffer_size: 4096,
            is_async: false, // DEFAULT: Synchronous to guarantee event ordering
            performance_level: PerformanceLevel::Auto,
            expected_throughput: None,
            subscriber_count: None,
        }
    }

    /// Set buffer size
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Enable or disable async processing mode [CRITICAL ORDERING TRADE-OFF]
    ///
    /// **⚠️  IMPORTANT: Enabling async mode may break event ordering guarantees!**
    ///
    /// **Synchronous Mode (false - DEFAULT):**
    /// - ✅ **Strict event ordering preserved**
    /// - ✅ Events processed sequentially in arrival order
    /// - ✅ Predictable, deterministic behavior
    /// - ❌ Lower throughput (~thousands events/sec)
    /// - **Use when**: Event order is critical for correctness
    ///
    /// **Async Mode (true):**
    /// - ✅ **High throughput** (>100K events/sec capability)
    /// - ✅ Better resource utilization with concurrent processing
    /// - ✅ Non-blocking, scalable performance
    /// - ❌ **Events may be processed out of order**
    /// - ❌ Less predictable timing behavior
    /// - **Use when**: Throughput > strict ordering requirements
    ///
    /// # Example
    /// ```
    /// use siddhi_rust::core::stream::JunctionConfig;
    ///
    /// // Default: Synchronous processing (guaranteed ordering)
    /// let sync_config = JunctionConfig::new("stream".to_string());
    ///
    /// // High-throughput async processing (potential reordering)
    /// let async_config = JunctionConfig::new("stream".to_string())
    ///     .with_async(true)
    ///     .with_expected_throughput(100_000);
    /// ```
    pub fn with_async(mut self, async_mode: bool) -> Self {
        self.is_async = async_mode;
        self
    }

    /// Set performance level
    pub fn with_performance_level(mut self, level: PerformanceLevel) -> Self {
        self.performance_level = level;
        self
    }

    /// Set expected throughput hint
    pub fn with_expected_throughput(mut self, throughput: u64) -> Self {
        self.expected_throughput = Some(throughput);
        self
    }

    /// Set expected subscriber count hint
    pub fn with_subscriber_count(mut self, count: usize) -> Self {
        self.subscriber_count = Some(count);
        self
    }
}

/// High-level junction types for different use cases
pub enum JunctionType {
    /// Original implementation for compatibility
    Standard(Arc<Mutex<StreamJunction>>),
    /// Optimized implementation for high performance
    Optimized(Arc<Mutex<OptimizedStreamJunction>>),
}

impl JunctionType {
    /// Get stream ID regardless of implementation
    pub fn stream_id(&self) -> String {
        match self {
            JunctionType::Standard(junction) => junction.lock().unwrap().stream_id.clone(),
            JunctionType::Optimized(junction) => junction.lock().unwrap().stream_id.clone(),
        }
    }

    /// Get total events processed
    pub fn total_events(&self) -> Option<u64> {
        match self {
            JunctionType::Standard(junction) => junction.lock().unwrap().total_events(),
            JunctionType::Optimized(junction) => junction.lock().unwrap().total_events(),
        }
    }

    /// Get average latency in nanoseconds
    pub fn average_latency_ns(&self) -> Option<u64> {
        match self {
            JunctionType::Standard(junction) => junction.lock().unwrap().average_latency_ns(),
            JunctionType::Optimized(junction) => junction.lock().unwrap().average_latency_ns(),
        }
    }

    /// Check if junction is using optimized implementation
    pub fn is_optimized(&self) -> bool {
        matches!(self, JunctionType::Optimized(_))
    }
}

/// Factory for creating StreamJunctions with automatic optimization
pub struct StreamJunctionFactory;

impl StreamJunctionFactory {
    /// Create a StreamJunction with automatic optimization selection
    pub fn create(
        config: JunctionConfig,
        stream_definition: Arc<StreamDefinition>,
        siddhi_app_context: Arc<SiddhiAppContext>,
        fault_stream_junction: Option<Arc<Mutex<StreamJunction>>>,
    ) -> Result<JunctionType, String> {
        let should_optimize = Self::should_use_optimized_junction(&config, &siddhi_app_context);

        if should_optimize {
            Self::create_optimized_junction(
                config,
                stream_definition,
                siddhi_app_context,
                fault_stream_junction,
            )
        } else {
            Self::create_standard_junction(
                config,
                stream_definition,
                siddhi_app_context,
                fault_stream_junction,
            )
        }
    }

    /// Create the standard crossbeam_channel-based junction
    pub fn create_standard_junction(
        config: JunctionConfig,
        stream_definition: Arc<StreamDefinition>,
        siddhi_app_context: Arc<SiddhiAppContext>,
        fault_stream_junction: Option<Arc<Mutex<StreamJunction>>>,
    ) -> Result<JunctionType, String> {
        let junction = StreamJunction::new(
            config.stream_id,
            stream_definition,
            siddhi_app_context,
            config.buffer_size,
            config.is_async,
            fault_stream_junction,
        );

        Ok(JunctionType::Standard(Arc::new(Mutex::new(junction))))
    }

    /// Create the optimized crossbeam pipeline-based junction
    pub fn create_optimized_junction(
        config: JunctionConfig,
        stream_definition: Arc<StreamDefinition>,
        siddhi_app_context: Arc<SiddhiAppContext>,
        _fault_stream_junction: Option<Arc<Mutex<StreamJunction>>>,
    ) -> Result<JunctionType, String> {
        // For now, we'll keep fault junction as None for the optimized implementation
        // In a full implementation, we'd want to create an optimized fault junction
        let optimized_fault_junction = None;

        let junction = OptimizedStreamJunction::new(
            config.stream_id,
            stream_definition,
            siddhi_app_context,
            config.buffer_size,
            config.is_async,
            optimized_fault_junction,
        )?;

        Ok(JunctionType::Optimized(Arc::new(Mutex::new(junction))))
    }

    /// Determine if optimized junction should be used
    fn should_use_optimized_junction(
        config: &JunctionConfig,
        siddhi_app_context: &SiddhiAppContext,
    ) -> bool {
        match config.performance_level {
            PerformanceLevel::Standard => false,
            PerformanceLevel::HighPerformance => true,
            PerformanceLevel::Auto => Self::auto_select_optimization(config, siddhi_app_context),
        }
    }

    /// Automatic selection logic based on workload characteristics
    fn auto_select_optimization(
        config: &JunctionConfig,
        _siddhi_app_context: &SiddhiAppContext,
    ) -> bool {
        let mut score = 0;

        // High throughput workloads benefit from optimization
        if let Some(throughput) = config.expected_throughput {
            if throughput > 100000 {
                // >100K events/sec
                score += 3;
            } else if throughput > 10000 {
                // >10K events/sec
                score += 1;
            }
        }

        // Multiple subscribers benefit from optimization
        if let Some(subscribers) = config.subscriber_count {
            if subscribers > 5 {
                score += 2;
            } else if subscribers > 2 {
                score += 1;
            }
        }

        // Large buffers suggest high-throughput scenarios
        if config.buffer_size > 8192 {
            score += 2;
        } else if config.buffer_size > 4096 {
            score += 1;
        }

        // Async mode benefits more from optimization
        if config.is_async {
            score += 1;
        }

        // Use optimized version if score suggests high-performance needs
        score >= 3
    }

    /// Create a junction with performance hints
    pub fn create_with_hints(
        stream_id: String,
        stream_definition: Arc<StreamDefinition>,
        siddhi_app_context: Arc<SiddhiAppContext>,
        expected_throughput: Option<u64>,
        subscriber_count: Option<usize>,
    ) -> Result<JunctionType, String> {
        let config = JunctionConfig::new(stream_id)
            .with_expected_throughput(expected_throughput.unwrap_or(0))
            .with_subscriber_count(subscriber_count.unwrap_or(1));

        Self::create(config, stream_definition, siddhi_app_context, None)
    }

    /// Create a high-performance junction for known high-throughput scenarios
    pub fn create_high_performance(
        stream_id: String,
        stream_definition: Arc<StreamDefinition>,
        siddhi_app_context: Arc<SiddhiAppContext>,
        buffer_size: usize,
    ) -> Result<JunctionType, String> {
        let config = JunctionConfig::new(stream_id)
            .with_buffer_size(buffer_size)
            .with_async(true)
            .with_performance_level(PerformanceLevel::HighPerformance);

        Self::create(config, stream_definition, siddhi_app_context, None)
    }
}

/// Performance benchmark for comparing junction implementations
pub struct JunctionBenchmark;

impl JunctionBenchmark {
    /// Run a simple throughput benchmark
    pub fn benchmark_throughput(
        junction: &JunctionType,
        num_events: usize,
        num_threads: usize,
    ) -> Result<BenchmarkResult, String> {
        use crate::core::event::{value::AttributeValue, Event};
        use std::thread;
        use std::time::Instant;

        let start = Instant::now();
        let mut handles = Vec::new();

        for thread_id in 0..num_threads {
            let events_per_thread = num_events / num_threads;

            match junction {
                JunctionType::Standard(junction) => {
                    let junction_clone = Arc::clone(junction);
                    handles.push(thread::spawn(move || {
                        for i in 0..events_per_thread {
                            let event = Event::new_with_data(
                                i as i64,
                                vec![AttributeValue::Int(thread_id as i32 * 1000 + i as i32)],
                            );
                            let _ = junction_clone.lock().unwrap().send_event(event);
                        }
                    }));
                }
                JunctionType::Optimized(junction) => {
                    let junction_clone = Arc::clone(junction);
                    handles.push(thread::spawn(move || {
                        for i in 0..events_per_thread {
                            let event = Event::new_with_data(
                                i as i64,
                                vec![AttributeValue::Int(thread_id as i32 * 1000 + i as i32)],
                            );
                            let _ = junction_clone.lock().unwrap().send_event(event);
                        }
                    }));
                }
            }
        }

        for handle in handles {
            handle.join().map_err(|_| "Thread join failed")?;
        }

        let duration = start.elapsed();
        let throughput = num_events as f64 / duration.as_secs_f64();

        Ok(BenchmarkResult {
            events_sent: num_events,
            duration,
            throughput,
            implementation: if junction.is_optimized() {
                "Optimized"
            } else {
                "Standard"
            }
            .to_string(),
        })
    }
}

/// Benchmark result for junction performance testing
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub events_sent: usize,
    pub duration: std::time::Duration,
    pub throughput: f64,
    pub implementation: String,
}

impl BenchmarkResult {
    /// Print benchmark results
    pub fn print(&self) {
        println!("Junction Benchmark Results:");
        println!("  Implementation: {}", self.implementation);
        println!("  Events sent: {}", self.events_sent);
        println!("  Duration: {:.2?}", self.duration);
        println!("  Throughput: {:.0} events/sec", self.throughput);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::siddhi_context::SiddhiContext;
    use crate::query_api::definition::attribute::Type as AttrType;

    fn create_test_context() -> Arc<SiddhiAppContext> {
        let siddhi_context = Arc::new(SiddhiContext::new());
        let app = Arc::new(crate::query_api::siddhi_app::SiddhiApp::new(
            "TestApp".to_string(),
        ));
        Arc::new(SiddhiAppContext::new(
            siddhi_context,
            "TestApp".to_string(),
            app,
            String::new(),
        ))
    }

    fn create_test_stream_definition() -> Arc<StreamDefinition> {
        Arc::new(
            StreamDefinition::new("TestStream".to_string())
                .attribute("id".to_string(), AttrType::INT),
        )
    }

    #[test]
    fn test_auto_selection_low_throughput() {
        let config = JunctionConfig::new("TestStream".to_string())
            .with_expected_throughput(1000) // Low throughput
            .with_subscriber_count(1);

        let context = create_test_context();
        let should_optimize =
            StreamJunctionFactory::should_use_optimized_junction(&config, &context);

        // Low throughput should use standard implementation
        assert!(!should_optimize);
    }

    #[test]
    fn test_auto_selection_high_throughput() {
        let config = JunctionConfig::new("TestStream".to_string())
            .with_expected_throughput(200000) // High throughput
            .with_subscriber_count(3) // Multiple subscribers
            .with_buffer_size(16384); // Large buffer

        let context = create_test_context();
        let should_optimize =
            StreamJunctionFactory::should_use_optimized_junction(&config, &context);

        // High throughput should use optimized implementation
        assert!(should_optimize);
    }

    #[test]
    fn test_force_standard_implementation() {
        let config = JunctionConfig::new("TestStream".to_string())
            .with_expected_throughput(500000) // Would normally trigger optimization
            .with_performance_level(PerformanceLevel::Standard); // But force standard

        let context = create_test_context();
        let stream_def = create_test_stream_definition();

        let junction = StreamJunctionFactory::create(config, stream_def, context, None).unwrap();
        assert!(!junction.is_optimized());
    }

    #[test]
    fn test_force_optimized_implementation() {
        let config = JunctionConfig::new("TestStream".to_string())
            .with_expected_throughput(100) // Low throughput
            .with_performance_level(PerformanceLevel::HighPerformance); // But force optimized

        let context = create_test_context();
        let stream_def = create_test_stream_definition();

        let junction = StreamJunctionFactory::create(config, stream_def, context, None).unwrap();
        assert!(junction.is_optimized());
    }

    #[test]
    fn test_high_performance_factory_method() {
        let context = create_test_context();
        let stream_def = create_test_stream_definition();

        let junction = StreamJunctionFactory::create_high_performance(
            "HighPerfStream".to_string(),
            stream_def,
            context,
            32768,
        )
        .unwrap();

        assert!(junction.is_optimized());
        assert_eq!(junction.stream_id(), "HighPerfStream");
    }

    #[test]
    fn test_junction_with_hints() {
        let context = create_test_context();
        let stream_def = create_test_stream_definition();

        // High throughput hint should trigger optimization
        let junction = StreamJunctionFactory::create_with_hints(
            "HintedStream".to_string(),
            stream_def,
            context,
            Some(150000), // High throughput
            Some(4),      // Multiple subscribers
        )
        .unwrap();

        assert!(junction.is_optimized());
    }
}
