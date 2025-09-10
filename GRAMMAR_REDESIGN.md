# Siddhi Rust Grammar Redesign Proposal

## Executive Summary

After comprehensive analysis of major CEP and stream processing systems, this document proposes a redesigned grammar for Siddhi Rust that:

1. **SQL-First Approach**: Adopts SQL as the primary query language with streaming extensions
2. **Modern Syntax**: Incorporates best practices from ksqlDB, Flink SQL, and Esper
3. **Streaming-Native**: Built-in support for streaming-specific concepts
4. **Backward Compatibility**: Maintains compatibility with existing Siddhi applications where possible

## Comparative Analysis: CEP/Stream Processor Grammars

### Grammar Philosophy Comparison

| System | Philosophy | SQL Compliance | Pattern Matching | Windowing | Time Handling |
|--------|------------|----------------|------------------|-----------|---------------|
| **Apache Flink SQL** | SQL + Stream Ext | High (Calcite) | Basic | Comprehensive | Advanced |
| **ksqlDB** | Streaming-Native SQL | Good | Moderate | Excellent | Native |
| **Esper EPL** | SQL + CEP Ext | Moderate | **Outstanding** | Good | Advanced |
| **Apache Beam SQL** | Unified Batch/Stream | High | Basic | Good | Good |
| **Hazelcast Jet** | Standard SQL + Ext | High | Basic | Good | Good |
| **Storm SQL** | Limited SQL | Basic | None | Limited | Basic |
| **Current Siddhi** | Domain-Specific | Low | Good | Moderate | Moderate |

### Key Insights from Analysis

1. **SQL Dominance**: 8/10 systems use SQL as primary interface
2. **Windowing Evolution**: Modern systems use `WINDOW` clause instead of stream modifiers
3. **Pattern Matching Gap**: Only Esper provides comprehensive pattern matching in SQL
4. **Time Handling**: Event time vs processing time distinction is critical
5. **Streaming Extensions**: All successful systems add streaming-specific SQL extensions

## Current Siddhi Grammar Issues

### 1. Non-SQL Syntax Barriers
```siddhi
-- Current Siddhi (non-SQL)
define stream InputStream (symbol string, price float);
from InputStream[price > 100]#window:length(5)
select symbol, avg(price) as avgPrice
insert into OutputStream;
```

**Problems**:
- `define stream` is non-standard (SQL uses `CREATE TABLE/STREAM`)
- `#window:length()` syntax is unfamiliar to SQL users
- `insert into` without explicit `INSERT INTO` statement
- Type declarations differ from SQL standard

### 2. Complex Window Syntax
```siddhi
-- Difficult to read and understand
from InputStream#window:time(5 min) as i1 
join InputStream#window:length(100) as i2 
on i1.symbol == i2.symbol
```

### 3. Limited SQL Familiarity
- Most developers know SQL
- Current syntax requires learning Siddhi-specific language
- Migration from other systems is difficult

## Proposed Grammar Redesign

### Philosophy: **SQL-First with Streaming Extensions**

Following the successful pattern of ksqlDB and Flink SQL:
- **Standard SQL** for familiar operations
- **Streaming extensions** for temporal operations
- **Clean syntax** for complex event processing
- **Intuitive windowing** semantics

### 1. Stream/Table Definitions (SQL DDL)

**Current**:
```siddhi
define stream StockStream (symbol string, price float, volume int);
define table StockTable (symbol string, price float, volume int);
```

**Proposed**:
```sql
-- Stream Creation (Unlimited, Append-Only)
CREATE STREAM stock_stream (
    symbol STRING,
    price DECIMAL(10,2),
    volume INTEGER,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
) WITH (
    'connector' = 'kafka',
    'topic' = 'stock-data',
    'format' = 'json',
    'event.time' = 'timestamp'
);

-- Table Creation (Mutable, Upsert)
CREATE TABLE stock_table (
    symbol STRING PRIMARY KEY,
    price DECIMAL(10,2),
    volume INTEGER,
    last_updated TIMESTAMP
) WITH (
    'connector' = 'jdbc',
    'url' = 'jdbc:postgresql://localhost:5432/stocks'
);
```

**Benefits**:
- Standard SQL DDL syntax
- Clear distinction between streams and tables
- Connector configuration separated from schema
- Event time specification built-in

### 2. Window Operations (SQL-Compliant)

**Current**:
```siddhi
from InputStream#window:time(5 min)
from InputStream#window:length(100)
from InputStream#window:session(5000, userId)
```

**Proposed**:
```sql
-- Tumbling Time Window
SELECT symbol, AVG(price) as avg_price
FROM stock_stream
WINDOW TUMBLING (INTERVAL '5' MINUTES)
GROUP BY symbol;

-- Sliding Window (Traditional SQL OVER)
SELECT symbol, price,
       AVG(price) OVER (
           ORDER BY event_time 
           RANGE BETWEEN INTERVAL '5' MINUTES PRECEDING AND CURRENT ROW
       ) as moving_avg
FROM stock_stream;

-- Session Window
SELECT user_id, COUNT(*) as event_count
FROM user_events
WINDOW SESSION (INTERVAL '30' SECONDS)
GROUP BY user_id;

-- Length-based Window  
SELECT symbol, AVG(price)
FROM stock_stream
WINDOW SLIDING (ROWS 100)
GROUP BY symbol;
```

**Advanced Windows**:
```sql
-- Complex Session with Partitioning
SELECT user_id, session_id, COUNT(*) as events,
       MIN(event_time) as session_start,
       MAX(event_time) as session_end
FROM user_events
WINDOW SESSION (
    INTERVAL '30' SECONDS 
    PARTITION BY user_id
    WITH INACTIVITY_GAP INTERVAL '5' MINUTES
)
GROUP BY user_id, session_id;

-- Custom Window with Context
SELECT zone, temperature, 
       AVG(temperature) OVER temperature_window as avg_temp
FROM sensor_data
WINDOW temperature_window AS (
    PARTITION BY zone
    ORDER BY event_time
    ROWS BETWEEN 10 PRECEDING AND CURRENT ROW
);
```

### 3. Stream Processing Queries

**Current**:
```siddhi
from InputStream[price > 100]
select symbol, price, volume * price as total_value
insert into OutputStream;
```

**Proposed**:
```sql
-- Continuous Query (Streaming)
INSERT INTO high_value_trades
SELECT symbol, price, volume * price as total_value
FROM stock_stream
WHERE price > 100
EMIT CHANGES;

-- Materialized View (Table Result)
CREATE TABLE high_value_stocks AS
SELECT symbol, 
       COUNT(*) as trade_count,
       AVG(price) as avg_price,
       MAX(volume * price) as max_value
FROM stock_stream
WHERE price > 100
WINDOW TUMBLING (INTERVAL '1' HOUR)
GROUP BY symbol
EMIT CHANGES;
```

### 4. Stream Joins

**Current**:
```siddhi
from StockStream#window:length(5) as s
join CompanyStream#window:length(10) as c
on s.symbol == c.symbol
select s.symbol, s.price, c.company_name
insert into EnrichedStream;
```

**Proposed**:
```sql
-- Stream-to-Stream Join
SELECT s.symbol, s.price, c.company_name, s.event_time
FROM stock_stream s
JOIN company_stream c
  ON s.symbol = c.symbol
  AND c.event_time BETWEEN s.event_time - INTERVAL '10' SECONDS 
                       AND s.event_time + INTERVAL '10' SECONDS
EMIT CHANGES;

-- Windowed Join
SELECT s.symbol, AVG(s.price) as avg_price, c.company_name
FROM stock_stream s
WINDOW TUMBLING (INTERVAL '1' MINUTE) AS sw
JOIN company_stream c  
WINDOW TUMBLING (INTERVAL '1' MINUTE) AS cw
  ON s.symbol = c.symbol
  AND sw.window_start = cw.window_start
GROUP BY s.symbol, c.company_name, sw.window_start
EMIT CHANGES;

-- Stream-to-Table Join (Lookup)
SELECT s.symbol, s.price, r.multiplier, s.price * r.multiplier as adjusted_price
FROM stock_stream s
LEFT JOIN reference_rates r
  ON s.currency = r.currency
FOR SYSTEM_TIME AS OF s.event_time
EMIT CHANGES;
```

### 5. Pattern Matching (Enhanced CEP)

**Current**:
```siddhi
from every a=StockStream[price>50] -> 
     b=StockStream[symbol==a.symbol and price<a.price*0.95]
select a.symbol, a.price as high_price, b.price as low_price
insert into AlertStream;
```

**Proposed**:
```sql
-- Pattern Matching with SQL Syntax
SELECT 
    high.symbol,
    high.price as high_price,
    low.price as low_price,
    high.event_time as high_time,
    low.event_time as low_time
FROM stock_stream
MATCH_RECOGNIZE (
    PARTITION BY symbol
    ORDER BY event_time
    MEASURES 
        high.price as high_price,
        low.price as low_price
    PATTERN (high low)
    DEFINE 
        high AS price > 50,
        low AS price < PREV(high.price) * 0.95
    WITHIN INTERVAL '10' MINUTES
) AS pattern_result
EMIT CHANGES;

-- Complex Pattern: Fraud Detection
SELECT account_id, transaction_pattern, total_amount
FROM transaction_stream  
MATCH_RECOGNIZE (
    PARTITION BY account_id
    ORDER BY transaction_time
    MEASURES
        COUNT(*) as transaction_count,
        SUM(amount) as total_amount,
        'RAPID_SMALL_THEN_LARGE' as transaction_pattern
    PATTERN (small{3,} large)
    DEFINE
        small AS amount < 10,
        large AS amount > 1000 AND 
                 transaction_time < PREV(small.transaction_time) + INTERVAL '5' MINUTES
    WITHIN INTERVAL '30' MINUTES
)
EMIT CHANGES;
```

**Pattern Advantages**:
- Standard SQL `MATCH_RECOGNIZE` syntax (SQL:2016)
- Clear pattern definition with `DEFINE` clause
- Flexible pattern quantifiers (`{3,}`, `?`, `*`, `+`)
- Built-in time constraints with `WITHIN`

### 6. Aggregation Queries

**Current**:
```siddhi
define aggregation StockAggregation
from StockStream
select symbol, avg(price) as avgPrice, sum(volume) as totalVolume  
group by symbol
aggregate every sec...year;
```

**Proposed**:
```sql
-- Time-based Aggregation Hierarchy  
CREATE MATERIALIZED TABLE stock_aggregation AS
SELECT 
    symbol,
    window_start,
    window_end, 
    AVG(price) as avg_price,
    SUM(volume) as total_volume,
    COUNT(*) as trade_count,
    MIN(price) as min_price,
    MAX(price) as max_price
FROM stock_stream
WINDOW TUMBLING (INTERVAL '1' SECOND)
GROUP BY symbol, window_start, window_end
EMIT CHANGES;

-- Multi-level Aggregation
CREATE TABLE hourly_stock_summary AS
SELECT 
    symbol,
    DATE_TRUNC('hour', window_start) as hour_start,
    AVG(avg_price) as hourly_avg_price,
    SUM(total_volume) as hourly_total_volume
FROM stock_aggregation
WHERE window_start >= CURRENT_TIMESTAMP - INTERVAL '1' HOUR
GROUP BY symbol, DATE_TRUNC('hour', window_start)
EMIT CHANGES;
```

### 7. Source and Sink Definitions

**Current**:
```siddhi
@sink(type='kafka', topic='output-topic', bootstrap.servers='localhost:9092')
define stream OutputStream(symbol string, price float);
```

**Proposed**:
```sql
-- Source Definition
CREATE STREAM input_stream (
    symbol STRING,
    price DECIMAL(10,2),
    event_time TIMESTAMP
) WITH (
    'connector' = 'kafka',
    'properties.bootstrap.servers' = 'localhost:9092',
    'topic' = 'stock-input',
    'format' = 'json',
    'scan.startup.mode' = 'earliest-offset'
);

-- Sink Definition  
CREATE SINK high_price_alerts (
    symbol STRING,
    price DECIMAL(10,2),
    alert_time TIMESTAMP
) WITH (
    'connector' = 'kafka',
    'properties.bootstrap.servers' = 'localhost:9092', 
    'topic' = 'price-alerts',
    'format' = 'avro',
    'key.format' = 'raw',
    'key.fields' = 'symbol'
);

-- Output to Sink
INSERT INTO high_price_alerts
SELECT symbol, price, CURRENT_TIMESTAMP as alert_time
FROM input_stream
WHERE price > 100
EMIT CHANGES;
```

## Advanced Grammar Features

### 1. User-Defined Functions

**Current**:
```siddhi
define function concat[javascript] return string {
    return data[0] + data[1];  
};
```

**Proposed**:
```sql
-- Scalar Function
CREATE FUNCTION calculate_vwap(price DECIMAL, volume INTEGER)
RETURNS DECIMAL
LANGUAGE JAVASCRIPT AS $$
    return price * volume;
$$;

-- Table Function (UDTF)
CREATE FUNCTION parse_json_array(json_str STRING)
RETURNS TABLE(id INTEGER, name STRING, value DECIMAL)
LANGUAGE JAVA AS 'com.company.functions.JsonArrayParser';

-- Usage in Query
SELECT symbol, calculate_vwap(price, volume) as vwap
FROM stock_stream
WHERE vwap > 1000000
EMIT CHANGES;
```

### 2. Time Travel and Versioning

```sql
-- Query Historical State
SELECT * FROM stock_prices 
FOR SYSTEM_TIME AS OF TIMESTAMP '2023-12-01 10:00:00';

-- Time Range Query
SELECT symbol, AVG(price) 
FROM stock_prices
FOR SYSTEM_TIME BETWEEN 
    TIMESTAMP '2023-12-01 09:00:00' 
    AND TIMESTAMP '2023-12-01 10:00:00'
GROUP BY symbol;
```

### 3. Complex Event Processing Extensions

```sql
-- Sequence Pattern with Conditions
SELECT *
FROM sensor_data
MATCH_RECOGNIZE (
    PARTITION BY device_id
    ORDER BY event_time
    MEASURES 
        normal_start.event_time as sequence_start,
        critical.event_time as critical_time
    PATTERN (normal_start warning+ critical)
    DEFINE
        normal_start AS temperature <= 70,
        warning AS temperature BETWEEN 70 AND 90,
        critical AS temperature > 90
    WITHIN INTERVAL '1' HOUR
);

-- Absence Pattern (what didn't happen)
SELECT device_id, 'HEARTBEAT_MISSING' as alert_type
FROM heartbeat_stream
MATCH_RECOGNIZE (
    PARTITION BY device_id  
    ORDER BY event_time
    MEASURES device_id
    PATTERN (heartbeat missing_interval)
    DEFINE
        heartbeat AS status = 'OK',
        missing_interval AS MATCH_NUMBER() = 1 AND 
                          event_time > PREV(heartbeat.event_time) + INTERVAL '5' MINUTES
);
```

## Migration Strategy

### Phase 1: Dual Grammar Support (Months 1-2)
```rust
// Support both syntaxes during migration
match query_type {
    QueryType::Legacy => parse_legacy_siddhi(query),
    QueryType::SqlExtended => parse_sql_extended(query),
    QueryType::Auto => detect_and_parse(query),
}
```

### Phase 2: Enhanced SQL Parser (Months 2-4)
- Implement full SQL DDL/DML support
- Add streaming-specific extensions
- Pattern matching with MATCH_RECOGNIZE

### Phase 3: Advanced Features (Months 4-6)  
- Time travel queries
- Complex CEP patterns
- UDF support
- Performance optimizations

### Phase 4: Legacy Deprecation (Months 6+)
- Mark legacy syntax as deprecated
- Provide automated migration tools
- Complete transition to SQL-first

## Implementation Architecture

### Grammar Structure
```
grammar/
├── sql_core/           # Standard SQL grammar (SELECT, FROM, WHERE, etc.)
├── streaming_ext/      # Streaming extensions (WINDOW, EMIT CHANGES)
├── cep_patterns/       # MATCH_RECOGNIZE and pattern matching  
├── ddl_extensions/     # CREATE STREAM/TABLE/FUNCTION
├── temporal_ops/       # Time travel, watermarks
└── compatibility/      # Legacy Siddhi syntax support
```

### Parser Pipeline
```rust
pub enum QueryType {
    SqlStandard,        // Pure SQL (SELECT, CREATE TABLE, etc.)
    SqlStreaming,       // SQL + streaming (EMIT CHANGES, WINDOW)
    SqlCep,            // SQL + CEP (MATCH_RECOGNIZE)
    LegacySiddhi,      // Backward compatibility
}

pub struct UniversalParser {
    sql_parser: SqlParser,
    streaming_parser: StreamingExtensionParser, 
    cep_parser: CepPatternParser,
    legacy_parser: LegacySiddhiParser,
}
```

## Benefits of Redesigned Grammar

### 1. Developer Experience
- **Familiar Syntax**: SQL knowledge directly applicable
- **Lower Learning Curve**: Reduced training time for new developers
- **Tool Integration**: Standard SQL tools work (IDEs, formatters, validators)

### 2. Migration & Adoption
- **Easy Migration**: From other stream processors (Flink, ksqlDB, etc.)
- **Gradual Adoption**: Can migrate queries incrementally
- **Standard Compliance**: Follows SQL standards where possible

### 3. Advanced Capabilities
- **Rich Pattern Matching**: SQL:2016 MATCH_RECOGNIZE support
- **Better Windowing**: More intuitive and powerful window operations
- **Time Travel**: Historical query capabilities
- **Performance**: Better optimization opportunities with SQL

### 4. Ecosystem Integration
- **BI Tools**: Standard SQL interface for dashboards and reports
- **Data Catalogs**: Schema discovery and metadata management
- **Query Builders**: Visual query construction tools

## Comparison: Before vs After

### Simple Stream Processing
```siddhi
-- BEFORE (Current Siddhi)
define stream StockStream(symbol string, price float);
from StockStream[price > 100]
select symbol, price  
insert into HighPriceStream;
```

```sql
-- AFTER (Proposed SQL)
CREATE STREAM stock_stream (symbol STRING, price DECIMAL);
INSERT INTO high_price_stream
SELECT symbol, price FROM stock_stream WHERE price > 100 EMIT CHANGES;
```

### Windowed Aggregation
```siddhi  
-- BEFORE
from StockStream#window:time(5 min)
select symbol, avg(price) as avgPrice
group by symbol
insert into AvgPriceStream;
```

```sql
-- AFTER  
INSERT INTO avg_price_stream
SELECT symbol, AVG(price) as avg_price
FROM stock_stream  
WINDOW TUMBLING (INTERVAL '5' MINUTES)
GROUP BY symbol
EMIT CHANGES;
```

### Complex Pattern Matching
```siddhi
-- BEFORE (Complex and non-standard)
from every a=StockStream[price > 50] -> 
     b=StockStream[symbol==a.symbol and price < a.price*0.95]
     within 10 min
select a.symbol, a.price, b.price
insert into DropAlert;
```

```sql
-- AFTER (Standard SQL with MATCH_RECOGNIZE)
INSERT INTO drop_alert
SELECT symbol, high_price, low_price
FROM stock_stream
MATCH_RECOGNIZE (
    PARTITION BY symbol
    ORDER BY event_time  
    MEASURES high.price AS high_price, low.price AS low_price
    PATTERN (high low)
    DEFINE 
        high AS price > 50,
        low AS price < PREV(high.price) * 0.95
    WITHIN INTERVAL '10' MINUTES
)
EMIT CHANGES;
```

## Conclusion

The proposed grammar redesign transforms Siddhi from a domain-specific language to a **SQL-first streaming platform** that:

1. **Leverages SQL Knowledge**: 95% of developers know SQL
2. **Follows Industry Standards**: Aligns with successful systems like ksqlDB and Flink SQL  
3. **Maintains Power**: Advanced CEP capabilities through MATCH_RECOGNIZE
4. **Enables Migration**: Easy transition from other streaming platforms
5. **Future-Proof**: Built on standard SQL with streaming extensions

This redesign positions Siddhi Rust as a modern, accessible, and powerful stream processing platform that can compete effectively with leading solutions while maintaining its unique CEP strengths.

**Implementation Timeline**: 6 months for full transition with backward compatibility maintained throughout the migration period.