// siddhi_rust/src/core/executor/function/scalar_function_executor.rs
use crate::core::executor::expression_executor::ExpressionExecutor;
use crate::core::config::siddhi_app_context::SiddhiAppContext;
use std::sync::Arc;
use std::fmt::Debug;

// ScalarFunctionExecutor represents a user-defined scalar function.
// It extends ExpressionExecutor because it's ultimately an expression that can be executed.
// The `init` method allows it to configure itself based on the arguments it receives
// (e.g., validate types, number of args, store argument types if needed for its execute logic).
// The `clone_scalar_function` is crucial for allowing the parser to create new instances
// of this function when building the execution plan, especially if plans are partitioned or
// if multiple parts of a query use the same UDF (they'd get independent instances).
pub trait ScalarFunctionExecutor: ExpressionExecutor {
    // Called by ExpressionParser after parsing function arguments into executors.
    // Allows the UDF to validate/configure itself based on argument types/count.
    // `argument_executors` are the already parsed executors for the function call's arguments.
    fn init(
        &mut self,
        argument_executors: &Vec<Box<dyn ExpressionExecutor>>,
        siddhi_app_context: &Arc<SiddhiAppContext>,
        // extension_configs: &HashMap<String, String> // If functions can have static configs from deployment.yaml
    ) -> Result<(), String>; // Return error if init fails (e.g. wrong arg count/type)

    // Returns the name of the function (e.g., "myUDF" or "custom:myUDF")
    // Used for identification and potentially by the Debug impl.
    fn get_name(&self) -> String;

    // For cloning itself when the parent AttributeFunctionExpressionExecutor is cloned.
    // This should create a "fresh" instance of the UDF logic, ready for its own `init`.
    fn clone_scalar_function(&self) -> Box<dyn ScalarFunctionExecutor>;
}

// Helper for cloning Box<dyn ScalarFunctionExecutor>
impl Clone for Box<dyn ScalarFunctionExecutor> {
   fn clone(&self) -> Self {
       self.clone_scalar_function()
   }
}

// Manual Debug for Box<dyn ScalarFunctionExecutor> is removed.
// The default Debug impl for Box<T> where T: Debug should be sufficient,
// and ScalarFunctionExecutor requires Debug.
// impl Debug for Box<dyn ScalarFunctionExecutor> {
//    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//        f.debug_struct("Box<dyn ScalarFunctionExecutor>")
//         .field("name", &self.get_name())
//         .finish_non_exhaustive()
//    }
// }
