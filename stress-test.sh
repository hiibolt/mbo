#!/usr/bin/env bash
# MBO Backend EXTREME Stress Test Script
# Generates MASSIVE sustained load: 100 concurrent clients, target 1M msg/s

set -e

BASE_URL="${1:-http://localhost:80}"
DURATION="${2:-30}"
CONCURRENT_STREAMS="${3:-100}"
HEALTH_CHECK_WORKERS="${4:-50}"

echo "🔥🔥🔥 MBO Backend EXTREME Stress Test 🔥🔥🔥"
echo "=============================================="
echo "Target: $BASE_URL"
echo "Duration: ${DURATION}s"
echo "Concurrent SSE Streams: $CONCURRENT_STREAMS"
echo "Health Check Workers: $HEALTH_CHECK_WORKERS"
echo ""
echo "⚠️  WARNING: This will generate MASSIVE load!"
echo "   Target: 1M messages/second"
echo "   100 concurrent streaming connections"
echo ""

# PID tracking
PIDS=()

# Cleanup function
cleanup() {
    echo ""
    echo "🧹 Cleaning up background processes..."
    for pid in "${PIDS[@]}"; do
        kill $pid 2>/dev/null || true
    done
    wait 2>/dev/null || true
}
trap cleanup EXIT

# Function to continuously stream full MBO data in a loop
continuous_stream() {
    local id=$1
    local duration=$2
    local end_time=$(($(date +%s) + duration))
    
    # Loop continuously to maintain constant throughput
    # Redirect all output to /dev/null for maximum performance
    while [ $(date +%s) -lt $end_time ]; do
        curl -s -N "$BASE_URL/api/mbo/stream/json" 2>&1 || true
    done
}

# Function to aggressively hammer health/ready/metrics endpoints
continuous_health_checks() {
    local duration=$1
    local end_time=$(($(date +%s) + duration))
    
    # Redirect all output to /dev/null
    while [ $(date +%s) -lt $end_time ]; do
        # Fire off 10 requests in rapid succession, no waiting
        for i in {1..10}; do
            curl -s "$BASE_URL/health" >/dev/null 2>&1 &
            curl -s "$BASE_URL/ready" >/dev/null 2>&1 &
            curl -s "$BASE_URL/metrics" >/dev/null 2>&1 &
        done
        # Very short sleep to prevent completely overwhelming the shell
        sleep 0.01
    done
}

# Function to show live metrics
show_metrics() {
    echo ""
    echo "📊 Live Metrics:"
    curl -s http://localhost:80/metrics 2>/dev/null | grep -E "mbo_(active_connections|http_requests_total|messages_processed_total) " | sed 's/# .*//' | sed 's/mbo_/  /'
}

echo "🚀 Launching ${CONCURRENT_STREAMS} streaming connections..."
echo "   (This will take ~$((CONCURRENT_STREAMS / 50)) seconds to start all)"
echo ""

# Start concurrent streaming connections in batches
BATCH_SIZE=50
for batch_start in $(seq 0 $BATCH_SIZE $((CONCURRENT_STREAMS - 1))); do
    batch_end=$((batch_start + BATCH_SIZE - 1))
    if [ $batch_end -ge $CONCURRENT_STREAMS ]; then
        batch_end=$((CONCURRENT_STREAMS - 1))
    fi
    
    for i in $(seq $batch_start $batch_end); do
        continuous_stream $i $DURATION &
        PIDS+=($!)
    done
    
    echo "  ✓ Batch $((batch_start / BATCH_SIZE + 1)): Started streams $batch_start-$batch_end"
    sleep 0.2
done

# Start multiple health check bombardment workers
echo ""
echo "🔫 Launching ${HEALTH_CHECK_WORKERS} health check workers..."
for i in $(seq 1 $HEALTH_CHECK_WORKERS); do
    continuous_health_checks $DURATION &
    PIDS+=($!)
done
echo "  ✓ All workers started"

echo ""
echo "💥💥💥 EXTREME LOAD TEST RUNNING 💥💥💥"
echo "Duration: ${DURATION}s"
echo "📈 Open Grafana NOW to watch the carnage: http://localhost:3001"
echo ""
echo "Metrics will update every 5 seconds..."

# Show metrics every 5 seconds
for i in $(seq 1 $((DURATION / 5))); do
    sleep 5
    echo ""
    echo "╔════════════════════════════════════════╗"
    echo "║  [$((i * 5))s / ${DURATION}s] LIVE METRICS"
    echo "╚════════════════════════════════════════╝"
    show_metrics
done

echo ""
echo "⏸️  Waiting for all streams to complete..."
wait

echo ""
echo "✅ EXTREME STRESS TEST COMPLETE!"
echo ""
echo "╔════════════════════════════════════════╗"
echo "║      FINAL PERFORMANCE METRICS        ║"
echo "╚════════════════════════════════════════╝"
show_metrics

echo ""
echo "📊 Detailed Stats from Prometheus:"
total_requests=$(curl -s 'http://localhost:9090/api/v1/query?query=mbo_http_requests_total' 2>/dev/null | grep -o '"value":\[[^]]*\]' | grep -o '[0-9]*"' | tail -1 | tr -d '"')
total_messages=$(curl -s 'http://localhost:9090/api/v1/query?query=mbo_messages_processed_total' 2>/dev/null | grep -o '"value":\[[^]]*\]' | grep -o '[0-9]*"' | tail -1 | tr -d '"')
peak_connections=$(curl -s 'http://localhost:9090/api/v1/query?query=max_over_time(mbo_active_connections[2m])' 2>/dev/null | grep -o '"value":\[[^]]*\]' | grep -o '[0-9]*"' | tail -1 | tr -d '"')

echo "  Total HTTP Requests: $(printf "%'d" $total_requests 2>/dev/null || echo $total_requests)"
echo "  Messages Processed: $(printf "%'d" $total_messages 2>/dev/null || echo $total_messages)"
echo "  Peak Connections: $peak_connections"

if [ ! -z "$total_messages" ] && [ "$total_messages" -gt 0 ]; then
    msg_per_sec=$((total_messages / DURATION))
    echo ""
    echo "  ⚡ Throughput: $(printf "%'d" $msg_per_sec 2>/dev/null || echo $msg_per_sec) messages/second"
    echo "  ⚡ Request Rate: $((total_requests / DURATION)) requests/second"
fi

echo ""
echo "📈 View dashboard: http://localhost:3001"
echo "🔍 Metrics endpoint: $BASE_URL/metrics"
echo ""
echo "Did we hit 1M msg/s? Check the throughput above! 🚀"
