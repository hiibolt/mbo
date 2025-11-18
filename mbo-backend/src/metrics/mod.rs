use prometheus::{
    Encoder, TextEncoder, Registry, Counter, Histogram, IntGauge, HistogramOpts, Opts,
};
use std::sync::Arc;
use anyhow::Result;

/// Application metrics for monitoring
pub struct Metrics {
    registry: Registry,
    
    // Message processing metrics
    pub messages_processed: Counter,
    pub messages_processing_errors: Counter,
    
    // Order book metrics
    pub order_book_updates: Counter,
    pub order_book_depth: IntGauge,
    pub order_book_apply_duration: Histogram,
    
    // API metrics
    pub active_connections: IntGauge,
    pub http_requests_total: Counter,
    pub http_request_duration: Histogram,
    
    // Database metrics
    pub db_operations_total: Counter,
    pub db_operation_duration: Histogram,
}

impl Metrics {
    /// Create a new metrics instance with all collectors registered
    pub fn new() -> Result<Arc<Self>> {
        let registry = Registry::new();
        
        // Message processing
        let messages_processed = Counter::with_opts(
            Opts::new("mbo_messages_processed_total", "Total number of MBO messages processed")
        )?;
        registry.register(Box::new(messages_processed.clone()))?;
        
        let messages_processing_errors = Counter::with_opts(
            Opts::new("mbo_messages_errors_total", "Total number of message processing errors")
        )?;
        registry.register(Box::new(messages_processing_errors.clone()))?;
        
        // Order book
        let order_book_updates = Counter::with_opts(
            Opts::new("mbo_order_book_updates_total", "Total number of order book updates")
        )?;
        registry.register(Box::new(order_book_updates.clone()))?;
        
        let order_book_depth = IntGauge::with_opts(
            Opts::new("mbo_order_book_depth", "Current total depth of the order book (bids + asks)")
        )?;
        registry.register(Box::new(order_book_depth.clone()))?;
        
        let order_book_apply_duration = Histogram::with_opts(
            HistogramOpts::new("mbo_order_book_apply_duration_seconds", "Duration of order book apply operations")
                .buckets(vec![0.00001, 0.00005, 0.0001, 0.0005, 0.001, 0.005, 0.01])
        )?;
        registry.register(Box::new(order_book_apply_duration.clone()))?;
        
        // API
        let active_connections = IntGauge::with_opts(
            Opts::new("mbo_active_connections", "Number of active SSE connections")
        )?;
        registry.register(Box::new(active_connections.clone()))?;
        
        let http_requests_total = Counter::with_opts(
            Opts::new("mbo_http_requests_total", "Total number of HTTP requests")
        )?;
        registry.register(Box::new(http_requests_total.clone()))?;
        
        let http_request_duration = Histogram::with_opts(
            HistogramOpts::new("mbo_http_request_duration_seconds", "HTTP request duration")
                .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0])
        )?;
        registry.register(Box::new(http_request_duration.clone()))?;
        
        // Database
        let db_operations_total = Counter::with_opts(
            Opts::new("mbo_db_operations_total", "Total number of database operations")
        )?;
        registry.register(Box::new(db_operations_total.clone()))?;
        
        let db_operation_duration = Histogram::with_opts(
            HistogramOpts::new("mbo_db_operation_duration_seconds", "Database operation duration")
                .buckets(vec![0.0001, 0.0005, 0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0])
        )?;
        registry.register(Box::new(db_operation_duration.clone()))?;
        
        Ok(Arc::new(Self {
            registry,
            messages_processed,
            messages_processing_errors,
            order_book_updates,
            order_book_depth,
            order_book_apply_duration,
            active_connections,
            http_requests_total,
            http_request_duration,
            db_operations_total,
            db_operation_duration,
        }))
    }
    
    /// Encode metrics in Prometheus text format
    pub fn encode(&self) -> Result<Vec<u8>> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = vec![];
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(buffer)
    }
}

// Note: No Default impl since we return Arc<Metrics> from new()
