use std::sync::Arc;

use crate::core::config::siddhi_app_context::SiddhiAppContext;
use crate::core::partition::PartitionRuntime;
use crate::core::siddhi_app_runtime_builder::SiddhiAppRuntimeBuilder;
use crate::core::util::parser::QueryParser;
use crate::query_api::execution::partition::Partition as ApiPartition;

pub struct PartitionParser;

impl PartitionParser {
    pub fn parse(
        builder: &mut SiddhiAppRuntimeBuilder,
        partition: &ApiPartition,
        siddhi_app_context: &Arc<SiddhiAppContext>,
    ) -> Result<PartitionRuntime, String> {
        let mut partition_runtime = PartitionRuntime::new();

        // ensure a named executor for partition queries exists
        if siddhi_app_context
            .get_siddhi_context()
            .get_executor_service("partition")
            .is_none()
        {
            siddhi_app_context.get_siddhi_context().add_executor_service(
                "partition".to_string(),
                Arc::new(crate::core::util::ExecutorService::new("partition", 2)),
            );
        }

        for query in &partition.query_list {
            let qr = QueryParser::parse_query(
                query,
                siddhi_app_context,
                &builder.stream_junction_map,
                &builder.table_definition_map,
                &builder.aggregation_map,
                Some("partition".to_string()),
            )?;
            let qr_arc = Arc::new(qr);
            builder.add_query_runtime(Arc::clone(&qr_arc));
            partition_runtime.add_query_runtime(qr_arc);
        }
        Ok(partition_runtime)
    }
}
