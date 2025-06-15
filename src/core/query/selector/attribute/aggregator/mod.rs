use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use crate::core::config::siddhi_app_context::SiddhiAppContext;
use crate::core::config::siddhi_query_context::SiddhiQueryContext;
use crate::core::event::complex_event::{ComplexEvent, ComplexEventType};
use crate::core::event::value::AttributeValue;
use crate::core::executor::expression_executor::ExpressionExecutor;
use crate::core::query::processor::ProcessingMode;
use crate::query_api::definition::attribute::Type as ApiAttributeType;

pub trait AttributeAggregatorExecutor: ExpressionExecutor {
    fn init(
        &mut self,
        executors: Vec<Box<dyn ExpressionExecutor>>,
        processing_mode: ProcessingMode,
        expired_output: bool,
        ctx: &SiddhiQueryContext,
    ) -> Result<(), String>;

    fn process_add(&self, data: Option<AttributeValue>) -> Option<AttributeValue>;
    fn process_remove(&self, data: Option<AttributeValue>) -> Option<AttributeValue>;
    fn reset(&self) -> Option<AttributeValue>;
    fn clone_box(&self) -> Box<dyn AttributeAggregatorExecutor>;
}

impl Clone for Box<dyn AttributeAggregatorExecutor> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

fn value_as_f64(v: &AttributeValue) -> Option<f64> {
    match v {
        AttributeValue::Int(i) => Some(*i as f64),
        AttributeValue::Long(l) => Some(*l as f64),
        AttributeValue::Float(f) => Some(*f as f64),
        AttributeValue::Double(d) => Some(*d),
        _ => None,
    }
}

#[derive(Debug, Clone, Default)]
struct SumState {
    sum: f64,
    count: u64,
}

#[derive(Debug)]
pub struct SumAttributeAggregatorExecutor {
    arg_exec: Option<Box<dyn ExpressionExecutor>>,
    return_type: ApiAttributeType,
    state: Mutex<SumState>,
    app_ctx: Option<Arc<SiddhiAppContext>>,
}

impl Default for SumAttributeAggregatorExecutor {
    fn default() -> Self {
        Self {
            arg_exec: None,
            return_type: ApiAttributeType::DOUBLE,
            state: Mutex::new(SumState::default()),
            app_ctx: None,
        }
    }
}

impl AttributeAggregatorExecutor for SumAttributeAggregatorExecutor {
    fn init(
        &mut self,
        mut executors: Vec<Box<dyn ExpressionExecutor>>,
        _mode: ProcessingMode,
        _expired_output: bool,
        ctx: &SiddhiQueryContext,
    ) -> Result<(), String> {
        if executors.len() != 1 {
            return Err("sum() requires exactly one argument".to_string());
        }
        let exec = executors.remove(0);
        let rtype = match exec.get_return_type() {
            ApiAttributeType::INT | ApiAttributeType::LONG => ApiAttributeType::LONG,
            ApiAttributeType::FLOAT | ApiAttributeType::DOUBLE => ApiAttributeType::DOUBLE,
            t => return Err(format!("sum not supported for {:?}", t)),
        };
        self.return_type = rtype;
        self.arg_exec = Some(exec);
        self.app_ctx = Some(Arc::clone(&ctx.siddhi_app_context));
        Ok(())
    }

    fn process_add(&self, data: Option<AttributeValue>) -> Option<AttributeValue> {
        if let Some(v) = data.as_ref().and_then(|v| value_as_f64(v)) {
            let mut st = self.state.lock().unwrap();
            st.sum += v;
            st.count += 1;
        }
        let st = self.state.lock().unwrap();
        match self.return_type {
            ApiAttributeType::LONG => Some(AttributeValue::Long(st.sum as i64)),
            ApiAttributeType::DOUBLE => Some(AttributeValue::Double(st.sum)),
            _ => None,
        }
    }

    fn process_remove(&self, data: Option<AttributeValue>) -> Option<AttributeValue> {
        if let Some(v) = data.as_ref().and_then(|v| value_as_f64(v)) {
            let mut st = self.state.lock().unwrap();
            st.sum -= v;
            if st.count > 0 {
                st.count -= 1;
            }
        }
        let st = self.state.lock().unwrap();
        match self.return_type {
            ApiAttributeType::LONG => Some(AttributeValue::Long(st.sum as i64)),
            ApiAttributeType::DOUBLE => Some(AttributeValue::Double(st.sum)),
            _ => None,
        }
    }

    fn reset(&self) -> Option<AttributeValue> {
        let mut st = self.state.lock().unwrap();
        st.sum = 0.0;
        st.count = 0;
        None
    }

    fn clone_box(&self) -> Box<dyn AttributeAggregatorExecutor> {
        let ctx = self.app_ctx.as_ref().unwrap();
        Box::new(SumAttributeAggregatorExecutor {
            arg_exec: self.arg_exec.as_ref().map(|e| e.clone_executor(ctx)),
            return_type: self.return_type,
            state: Mutex::new(self.state.lock().unwrap().clone()),
            app_ctx: Some(Arc::clone(ctx)),
        })
    }
}

impl ExpressionExecutor for SumAttributeAggregatorExecutor {
    fn execute(&self, event: Option<&dyn ComplexEvent>) -> Option<AttributeValue> {
        let event = event?;
        let data = self.arg_exec.as_ref().and_then(|e| e.execute(Some(event)));
        match event.get_event_type() {
            ComplexEventType::Current => self.process_add(data),
            ComplexEventType::Expired => self.process_remove(data),
            ComplexEventType::Reset => self.reset(),
            _ => None,
        }
    }

    fn get_return_type(&self) -> ApiAttributeType {
        self.return_type
    }

    fn clone_executor(&self, _ctx: &Arc<SiddhiAppContext>) -> Box<dyn ExpressionExecutor> {
        self.clone_box()
    }

    fn is_attribute_aggregator(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Default)]
struct AvgState {
    sum: f64,
    count: u64,
}

#[derive(Debug)]
pub struct AvgAttributeAggregatorExecutor {
    arg_exec: Option<Box<dyn ExpressionExecutor>>,
    state: Mutex<AvgState>,
    app_ctx: Option<Arc<SiddhiAppContext>>,
}

impl Default for AvgAttributeAggregatorExecutor {
    fn default() -> Self {
        Self {
            arg_exec: None,
            state: Mutex::new(AvgState::default()),
            app_ctx: None,
        }
    }
}

impl AttributeAggregatorExecutor for AvgAttributeAggregatorExecutor {
    fn init(
        &mut self,
        mut execs: Vec<Box<dyn ExpressionExecutor>>,
        _m: ProcessingMode,
        _e: bool,
        ctx: &SiddhiQueryContext,
    ) -> Result<(), String> {
        if execs.len() != 1 {
            return Err("avg() requires one argument".to_string());
        }
        self.arg_exec = Some(execs.remove(0));
        self.app_ctx = Some(Arc::clone(&ctx.siddhi_app_context));
        Ok(())
    }

    fn process_add(&self, data: Option<AttributeValue>) -> Option<AttributeValue> {
        if let Some(v) = data.as_ref().and_then(|v| value_as_f64(v)) {
            let mut st = self.state.lock().unwrap();
            st.sum += v;
            st.count += 1;
        }
        let st = self.state.lock().unwrap();
        if st.count == 0 {
            None
        } else {
            Some(AttributeValue::Double(st.sum / st.count as f64))
        }
    }

    fn process_remove(&self, data: Option<AttributeValue>) -> Option<AttributeValue> {
        if let Some(v) = data.as_ref().and_then(|v| value_as_f64(v)) {
            let mut st = self.state.lock().unwrap();
            st.sum -= v;
            if st.count > 0 {
                st.count -= 1;
            }
        }
        let st = self.state.lock().unwrap();
        if st.count == 0 {
            None
        } else {
            Some(AttributeValue::Double(st.sum / st.count as f64))
        }
    }

    fn reset(&self) -> Option<AttributeValue> {
        let mut st = self.state.lock().unwrap();
        st.sum = 0.0;
        st.count = 0;
        None
    }

    fn clone_box(&self) -> Box<dyn AttributeAggregatorExecutor> {
        let ctx = self.app_ctx.as_ref().unwrap();
        Box::new(AvgAttributeAggregatorExecutor {
            arg_exec: self.arg_exec.as_ref().map(|e| e.clone_executor(ctx)),
            state: Mutex::new(self.state.lock().unwrap().clone()),
            app_ctx: Some(Arc::clone(ctx)),
        })
    }
}

impl ExpressionExecutor for AvgAttributeAggregatorExecutor {
    fn execute(&self, event: Option<&dyn ComplexEvent>) -> Option<AttributeValue> {
        let event = event?;
        let data = self.arg_exec.as_ref().and_then(|e| e.execute(Some(event)));
        match event.get_event_type() {
            ComplexEventType::Current => self.process_add(data),
            ComplexEventType::Expired => self.process_remove(data),
            ComplexEventType::Reset => self.reset(),
            _ => None,
        }
    }

    fn get_return_type(&self) -> ApiAttributeType {
        ApiAttributeType::DOUBLE
    }

    fn clone_executor(&self, _ctx: &Arc<SiddhiAppContext>) -> Box<dyn ExpressionExecutor> {
        self.clone_box()
    }

    fn is_attribute_aggregator(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Default)]
struct CountState {
    count: i64,
}

#[derive(Debug)]
pub struct CountAttributeAggregatorExecutor {
    state: Mutex<CountState>,
    app_ctx: Option<Arc<SiddhiAppContext>>,
}

impl Default for CountAttributeAggregatorExecutor {
    fn default() -> Self {
        Self {
            state: Mutex::new(CountState::default()),
            app_ctx: None,
        }
    }
}

impl AttributeAggregatorExecutor for CountAttributeAggregatorExecutor {
    fn init(
        &mut self,
        _e: Vec<Box<dyn ExpressionExecutor>>,
        _m: ProcessingMode,
        _ex: bool,
        ctx: &SiddhiQueryContext,
    ) -> Result<(), String> {
        self.app_ctx = Some(Arc::clone(&ctx.siddhi_app_context));
        Ok(())
    }

    fn process_add(&self, _d: Option<AttributeValue>) -> Option<AttributeValue> {
        let mut st = self.state.lock().unwrap();
        st.count += 1;
        Some(AttributeValue::Long(st.count))
    }
    fn process_remove(&self, _d: Option<AttributeValue>) -> Option<AttributeValue> {
        let mut st = self.state.lock().unwrap();
        st.count -= 1;
        Some(AttributeValue::Long(st.count))
    }
    fn reset(&self) -> Option<AttributeValue> {
        self.state.lock().unwrap().count = 0;
        Some(AttributeValue::Long(0))
    }
    fn clone_box(&self) -> Box<dyn AttributeAggregatorExecutor> {
        Box::new(CountAttributeAggregatorExecutor {
            state: Mutex::new(self.state.lock().unwrap().clone()),
            app_ctx: self.app_ctx.as_ref().cloned(),
        })
    }
}

impl ExpressionExecutor for CountAttributeAggregatorExecutor {
    fn execute(&self, event: Option<&dyn ComplexEvent>) -> Option<AttributeValue> {
        let event = event?;
        match event.get_event_type() {
            ComplexEventType::Current => self.process_add(None),
            ComplexEventType::Expired => self.process_remove(None),
            ComplexEventType::Reset => self.reset(),
            _ => None,
        }
    }

    fn get_return_type(&self) -> ApiAttributeType {
        ApiAttributeType::LONG
    }

    fn clone_executor(&self, _ctx: &Arc<SiddhiAppContext>) -> Box<dyn ExpressionExecutor> {
        self.clone_box()
    }

    fn is_attribute_aggregator(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Default)]
struct DistinctCountState {
    map: HashMap<String, i64>,
}

#[derive(Debug)]
pub struct DistinctCountAttributeAggregatorExecutor {
    arg_exec: Option<Box<dyn ExpressionExecutor>>,
    state: Mutex<DistinctCountState>,
    app_ctx: Option<Arc<SiddhiAppContext>>,
}

impl Default for DistinctCountAttributeAggregatorExecutor {
    fn default() -> Self {
        Self {
            arg_exec: None,
            state: Mutex::new(DistinctCountState::default()),
            app_ctx: None,
        }
    }
}

impl AttributeAggregatorExecutor for DistinctCountAttributeAggregatorExecutor {
    fn init(
        &mut self,
        mut e: Vec<Box<dyn ExpressionExecutor>>,
        _m: ProcessingMode,
        _ex: bool,
        ctx: &SiddhiQueryContext,
    ) -> Result<(), String> {
        if e.len() != 1 {
            return Err("distinctCount() requires one arg".to_string());
        }
        self.arg_exec = Some(e.remove(0));
        self.app_ctx = Some(Arc::clone(&ctx.siddhi_app_context));
        Ok(())
    }
    fn process_add(&self, data: Option<AttributeValue>) -> Option<AttributeValue> {
        if let Some(v) = data {
            let key = format!("{:?}", v);
            let mut st = self.state.lock().unwrap();
            let c = st.map.entry(key).or_insert(0);
            *c += 1;
        }
        Some(AttributeValue::Long(
            self.state.lock().unwrap().map.len() as i64
        ))
    }
    fn process_remove(&self, data: Option<AttributeValue>) -> Option<AttributeValue> {
        if let Some(v) = data {
            let key = format!("{:?}", v);
            let mut st = self.state.lock().unwrap();
            if let Some(c) = st.map.get_mut(&key) {
                *c -= 1;
                if *c <= 0 {
                    st.map.remove(&key);
                }
            }
        }
        Some(AttributeValue::Long(
            self.state.lock().unwrap().map.len() as i64
        ))
    }
    fn reset(&self) -> Option<AttributeValue> {
        self.state.lock().unwrap().map.clear();
        Some(AttributeValue::Long(0))
    }
    fn clone_box(&self) -> Box<dyn AttributeAggregatorExecutor> {
        let ctx = self.app_ctx.as_ref().unwrap();
        Box::new(DistinctCountAttributeAggregatorExecutor {
            arg_exec: self.arg_exec.as_ref().map(|e| e.clone_executor(ctx)),
            state: Mutex::new(self.state.lock().unwrap().clone()),
            app_ctx: Some(Arc::clone(ctx)),
        })
    }
}

impl ExpressionExecutor for DistinctCountAttributeAggregatorExecutor {
    fn execute(&self, event: Option<&dyn ComplexEvent>) -> Option<AttributeValue> {
        let event = event?;
        let data = self.arg_exec.as_ref().and_then(|e| e.execute(Some(event)));
        match event.get_event_type() {
            ComplexEventType::Current => self.process_add(data),
            ComplexEventType::Expired => self.process_remove(data),
            ComplexEventType::Reset => self.reset(),
            _ => None,
        }
    }
    fn get_return_type(&self) -> ApiAttributeType {
        ApiAttributeType::LONG
    }
    fn clone_executor(&self, _ctx: &Arc<SiddhiAppContext>) -> Box<dyn ExpressionExecutor> {
        self.clone_box()
    }
    fn is_attribute_aggregator(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Default)]
struct MinMaxState {
    value: Option<f64>,
}

macro_rules! minmax_exec {
    ($name:ident, $cmp:expr) => {
        #[derive(Debug)]
        pub struct $name {
            arg_exec: Option<Box<dyn ExpressionExecutor>>,
            return_type: ApiAttributeType,
            state: Mutex<MinMaxState>,
            app_ctx: Option<Arc<SiddhiAppContext>>,
        }
        impl Default for $name {
            fn default() -> Self {
                Self {
                    arg_exec: None,
                    return_type: ApiAttributeType::DOUBLE,
                    state: Mutex::new(MinMaxState::default()),
                    app_ctx: None,
                }
            }
        }
        impl AttributeAggregatorExecutor for $name {
            fn init(
                &mut self,
                mut e: Vec<Box<dyn ExpressionExecutor>>,
                _m: ProcessingMode,
                _ex: bool,
                ctx: &SiddhiQueryContext,
            ) -> Result<(), String> {
                if e.len() != 1 {
                    return Err("aggregator requires one arg".into());
                }
                let exec = e.remove(0);
                self.return_type = exec.get_return_type();
                self.arg_exec = Some(exec);
                self.app_ctx = Some(Arc::clone(&ctx.siddhi_app_context));
                Ok(())
            }
            fn process_add(&self, data: Option<AttributeValue>) -> Option<AttributeValue> {
                if let Some(v) = data.and_then(|v| value_as_f64(&v)) {
                    let mut st = self.state.lock().unwrap();
                    match st.value {
                        Some(current) => {
                            if $cmp(v, current) {
                                st.value = Some(v);
                            }
                        }
                        None => st.value = Some(v),
                    }
                }
                st_to_val(self.return_type, &self.state.lock().unwrap())
            }
            fn process_remove(&self, _data: Option<AttributeValue>) -> Option<AttributeValue> {
                st_to_val(self.return_type, &self.state.lock().unwrap())
            }
            fn reset(&self) -> Option<AttributeValue> {
                self.state.lock().unwrap().value = None;
                None
            }
            fn clone_box(&self) -> Box<dyn AttributeAggregatorExecutor> {
                let ctx = self.app_ctx.as_ref().unwrap();
                Box::new($name {
                    arg_exec: self.arg_exec.as_ref().map(|e| e.clone_executor(ctx)),
                    return_type: self.return_type,
                    state: Mutex::new(self.state.lock().unwrap().clone()),
                    app_ctx: Some(Arc::clone(ctx)),
                })
            }
        }
        impl ExpressionExecutor for $name {
            fn execute(&self, event: Option<&dyn ComplexEvent>) -> Option<AttributeValue> {
                let event = event?;
                let data = self.arg_exec.as_ref().and_then(|e| e.execute(Some(event)));
                match event.get_event_type() {
                    ComplexEventType::Current => self.process_add(data),
                    ComplexEventType::Expired => self.process_remove(data),
                    ComplexEventType::Reset => self.reset(),
                    _ => None,
                }
            }
            fn get_return_type(&self) -> ApiAttributeType {
                self.return_type
            }
            fn clone_executor(&self, _ctx: &Arc<SiddhiAppContext>) -> Box<dyn ExpressionExecutor> {
                self.clone_box()
            }
            fn is_attribute_aggregator(&self) -> bool {
                true
            }
        }
    };
}

fn st_to_val(rt: ApiAttributeType, st: &MinMaxState) -> Option<AttributeValue> {
    st.value.map(|v| match rt {
        ApiAttributeType::INT => AttributeValue::Int(v as i32),
        ApiAttributeType::LONG => AttributeValue::Long(v as i64),
        ApiAttributeType::FLOAT => AttributeValue::Float(v as f32),
        _ => AttributeValue::Double(v),
    })
}

minmax_exec!(MinAttributeAggregatorExecutor, |v, current| v < current);
minmax_exec!(MaxAttributeAggregatorExecutor, |v, current| v > current);
minmax_exec!(MinForeverAttributeAggregatorExecutor, |v, current| v
    < current);
minmax_exec!(MaxForeverAttributeAggregatorExecutor, |v, current| v
    > current);

use crate::core::extension::AttributeAggregatorFactory;

#[derive(Debug, Clone)]
pub struct SumAttributeAggregatorFactory;

impl AttributeAggregatorFactory for SumAttributeAggregatorFactory {
    fn name(&self) -> &'static str { "sum" }
    fn create(&self) -> Box<dyn AttributeAggregatorExecutor> {
        Box::new(SumAttributeAggregatorExecutor::default())
    }
    fn clone_box(&self) -> Box<dyn AttributeAggregatorFactory> {
        Box::new(Self)
    }
}

#[derive(Debug, Clone)]
pub struct AvgAttributeAggregatorFactory;

impl AttributeAggregatorFactory for AvgAttributeAggregatorFactory {
    fn name(&self) -> &'static str { "avg" }
    fn create(&self) -> Box<dyn AttributeAggregatorExecutor> {
        Box::new(AvgAttributeAggregatorExecutor::default())
    }
    fn clone_box(&self) -> Box<dyn AttributeAggregatorFactory> {
        Box::new(Self)
    }
}

#[derive(Debug, Clone)]
pub struct CountAttributeAggregatorFactory;

impl AttributeAggregatorFactory for CountAttributeAggregatorFactory {
    fn name(&self) -> &'static str { "count" }
    fn create(&self) -> Box<dyn AttributeAggregatorExecutor> {
        Box::new(CountAttributeAggregatorExecutor::default())
    }
    fn clone_box(&self) -> Box<dyn AttributeAggregatorFactory> {
        Box::new(Self)
    }
}

#[derive(Debug, Clone)]
pub struct DistinctCountAttributeAggregatorFactory;

impl AttributeAggregatorFactory for DistinctCountAttributeAggregatorFactory {
    fn name(&self) -> &'static str { "distinctCount" }
    fn create(&self) -> Box<dyn AttributeAggregatorExecutor> {
        Box::new(DistinctCountAttributeAggregatorExecutor::default())
    }
    fn clone_box(&self) -> Box<dyn AttributeAggregatorFactory> {
        Box::new(Self)
    }
}

#[derive(Debug, Clone)]
pub struct MinAttributeAggregatorFactory;

impl AttributeAggregatorFactory for MinAttributeAggregatorFactory {
    fn name(&self) -> &'static str { "min" }
    fn create(&self) -> Box<dyn AttributeAggregatorExecutor> {
        Box::new(MinAttributeAggregatorExecutor::default())
    }
    fn clone_box(&self) -> Box<dyn AttributeAggregatorFactory> {
        Box::new(Self)
    }
}

#[derive(Debug, Clone)]
pub struct MaxAttributeAggregatorFactory;

impl AttributeAggregatorFactory for MaxAttributeAggregatorFactory {
    fn name(&self) -> &'static str { "max" }
    fn create(&self) -> Box<dyn AttributeAggregatorExecutor> {
        Box::new(MaxAttributeAggregatorExecutor::default())
    }
    fn clone_box(&self) -> Box<dyn AttributeAggregatorFactory> {
        Box::new(Self)
    }
}

#[derive(Debug, Clone)]
pub struct MinForeverAttributeAggregatorFactory;

impl AttributeAggregatorFactory for MinForeverAttributeAggregatorFactory {
    fn name(&self) -> &'static str { "minForever" }
    fn create(&self) -> Box<dyn AttributeAggregatorExecutor> {
        Box::new(MinForeverAttributeAggregatorExecutor::default())
    }
    fn clone_box(&self) -> Box<dyn AttributeAggregatorFactory> {
        Box::new(Self)
    }
}

#[derive(Debug, Clone)]
pub struct MaxForeverAttributeAggregatorFactory;

impl AttributeAggregatorFactory for MaxForeverAttributeAggregatorFactory {
    fn name(&self) -> &'static str { "maxForever" }
    fn create(&self) -> Box<dyn AttributeAggregatorExecutor> {
        Box::new(MaxForeverAttributeAggregatorExecutor::default())
    }
    fn clone_box(&self) -> Box<dyn AttributeAggregatorFactory> {
        Box::new(Self)
    }
}
