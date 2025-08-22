# Docker Setup for Siddhi Rust Testing

This directory contains Docker Compose configuration for testing Siddhi Rust's distributed state backend with real Redis instances.

## Quick Start

### 1. Run the Complete Example

The easiest way to test the Redis state backend:

```bash
./run_redis_example.sh
```

This script will:
- Start Redis using Docker Compose
- Wait for Redis to be ready
- Run the comprehensive Redis state backend example
- Provide helpful commands for further exploration

### 2. Manual Setup

If you prefer to run things step by step:

```bash
# Start Redis
docker-compose up -d

# Wait a moment for Redis to start
sleep 3

# Run the example
cargo run --example redis_state_example

# Or run the full test suite
cargo test distributed_redis_state
```

## Services Included

### Redis
- **Container**: `siddhi-redis`
- **Port**: 6379 (Redis default)
- **Features**:
  - Persistent storage with AOF (Append Only File)
  - Memory limit: 256MB with LRU eviction
  - Health checks enabled
- **Data**: Stored in Docker volume `redis-data`

### Redis Commander (Web UI)
- **Container**: `siddhi-redis-commander`
- **Port**: 8081
- **URL**: http://localhost:8081
- **Purpose**: Web-based Redis management interface

## Configuration Details

### Redis Configuration
The Redis instance is configured for testing with:
- AOF persistence enabled for data durability
- 256MB memory limit with `allkeys-lru` eviction policy
- Health checks every 5 seconds
- Automatic restart unless stopped

### Network
- Custom bridge network: `siddhi-network`
- Enables container-to-container communication
- Isolates Redis traffic

## Useful Commands

### Docker Management
```bash
# Start services
docker-compose up -d

# Stop services
docker-compose down

# View logs
docker logs siddhi-redis
docker logs siddhi-redis-commander

# Remove everything (including data)
docker-compose down -v
```

### Redis Operations
```bash
# Connect to Redis CLI
docker exec -it siddhi-redis redis-cli

# View all keys (from host)
docker exec siddhi-redis redis-cli keys '*'

# Monitor Redis activity
docker exec siddhi-redis redis-cli monitor

# Get Redis info
docker exec siddhi-redis redis-cli info
```

### Testing Commands
```bash
# Run specific Redis tests
cargo test distributed_redis_state -- --nocapture

# Run all distributed tests
cargo test distributed -- --nocapture

# Run the Redis example
cargo run --example redis_state_example
```

## Troubleshooting

### Redis Won't Start
```bash
# Check Docker is running
docker info

# Check logs
docker logs siddhi-redis

# Restart services
docker-compose restart
```

### Connection Issues
```bash
# Test Redis connectivity
docker exec siddhi-redis redis-cli ping

# Check if port is accessible
nc -zv localhost 6379

# Verify Redis is listening
docker exec siddhi-redis netstat -tlnp | grep 6379
```

### Example Fails
```bash
# Ensure Redis is healthy
docker-compose ps

# Check Redis logs for errors
docker logs siddhi-redis

# Try manual Redis connection
redis-cli -h localhost -p 6379 ping
```

## Development Workflow

### 1. Testing New Features
```bash
# Start Redis
docker-compose up -d

# Develop and test
cargo test distributed_redis_state

# Make changes, repeat tests
cargo run --example redis_state_example

# Stop when done
docker-compose down
```

### 2. Performance Testing
```bash
# Start with monitoring
docker-compose up -d
docker exec siddhi-redis redis-cli monitor &

# Run performance tests
cargo test distributed_redis_state --release

# Check Redis stats
docker exec siddhi-redis redis-cli info stats
```

### 3. Data Inspection
```bash
# Start services
docker-compose up -d

# Run example to populate data
cargo run --example redis_state_example

# View data in web UI
open http://localhost:8081

# Or use CLI
docker exec siddhi-redis redis-cli keys 'siddhi:*'
```

## Data Persistence

Redis data is stored in a Docker volume (`redis-data`) and persists between container restarts. To completely reset:

```bash
# Stop and remove everything including data
docker-compose down -v

# Start fresh
docker-compose up -d
```

## Production Considerations

This Docker setup is designed for development and testing. For production:

1. **Security**: Add authentication (`requirepass`)
2. **Persistence**: Configure RDB + AOF for better durability
3. **Memory**: Adjust memory limits based on your data size
4. **Networking**: Use proper Docker networks or external Redis
5. **Monitoring**: Integrate with your monitoring stack

## Integration with CI/CD

This setup can be used in CI/CD pipelines:

```yaml
# Example GitHub Actions step
- name: Start Redis for testing
  run: docker-compose up -d

- name: Wait for Redis
  run: sleep 5

- name: Run Redis tests
  run: cargo test distributed_redis_state

- name: Stop Redis
  run: docker-compose down
```