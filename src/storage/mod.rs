use rusqlite::{Connection, params};
use databento::dbn::MboMsg;
use anyhow::{Context, Result};
use tracing::{info, debug};
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct Storage {
    conn: Arc<Mutex<Connection>>,
}
impl Storage {
    #[tracing::instrument]
    pub fn new<P: AsRef<Path> + std::fmt::Debug>(db_path: P) -> Result<Self> {
        info!("Opening SQLite database at {:?}", db_path.as_ref());
        
        let conn = Connection::open(db_path)
            .context("Failed to open SQLite database")?;
        
        // Enable WAL mode for better concurrent reads
        conn.pragma_update(None, "journal_mode", "WAL")
            .context("Failed to enable WAL mode")?;
        
        // Enable foreign keys
        conn.pragma_update(None, "foreign_keys", "ON")
            .context("Failed to enable foreign keys")?;
        
        let storage = Self { 
            conn: Arc::new(Mutex::new(conn))
        };
        
        storage.initialize_schema()
            .context("Failed to initialize database schema")?;
        
        Ok(storage)
    }

    /// Create all tables if they don't exist
    #[tracing::instrument(skip(self))]
    fn initialize_schema(&self) -> Result<()> {
        info!("Initializing database schema...");
        
        let conn = self.conn.lock().unwrap();
        
        // Create instruments table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS instruments (
                instrument_id INTEGER PRIMARY KEY,
                symbol TEXT,
                publisher INTEGER NOT NULL
            )",
            [],
        ).context("Failed to create instruments table")?;

        // Create MBO messages table with all relevant fields
        conn.execute(
            "CREATE TABLE IF NOT EXISTS mbo_messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ts_recv INTEGER NOT NULL,
                ts_event INTEGER NOT NULL,
                instrument_id INTEGER NOT NULL,
                publisher INTEGER NOT NULL,
                order_id INTEGER NOT NULL,
                action TEXT NOT NULL,
                side TEXT NOT NULL,
                price INTEGER NOT NULL,
                size INTEGER NOT NULL,
                flags INTEGER NOT NULL,
                sequence INTEGER NOT NULL,
                ts_in_delta INTEGER NOT NULL,
                channel_id INTEGER NOT NULL
            )",
            [],
        ).context("Failed to create mbo_messages table")?;

        // Create indices for common queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_mbo_instrument_time 
             ON mbo_messages(instrument_id, ts_recv DESC)",
            [],
        ).context("Failed to create instrument_time index")?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_mbo_order 
             ON mbo_messages(order_id, ts_recv DESC)",
            [],
        ).context("Failed to create order index")?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_mbo_publisher 
             ON mbo_messages(publisher, instrument_id, ts_recv DESC)",
            [],
        ).context("Failed to create publisher index")?;

        info!("Database schema initialized successfully");
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(order_id = msg.order_id, instrument_id = msg.hd.instrument_id))]
    pub fn insert_mbo(&self, msg: &MboMsg) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        
        // Convert action enum to string
        let action = format!("{:?}", msg.action);
        
        // Convert side enum to string
        let side = format!("{:?}", msg.side);
        
        conn.execute(
            "INSERT INTO mbo_messages 
             (ts_recv, ts_event, instrument_id, publisher, order_id, action, side, 
              price, size, flags, sequence, ts_in_delta, channel_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                msg.hd.ts_event as i64,
                msg.ts_recv as i64,
                msg.hd.instrument_id,
                msg.hd.publisher_id,
                msg.order_id as i64,
                action,
                side,
                msg.price,
                msg.size,
                msg.flags.raw(),
                msg.sequence,
                msg.ts_in_delta,
                msg.channel_id,
            ],
        ).context("Failed to insert MBO message")?;

        Ok(())
    }
    
    #[tracing::instrument(skip(self, messages), fields(count = messages.len()))]
    pub fn insert_mbo_batch(&self, messages: &[MboMsg]) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()
            .context("Failed to begin transaction")?;

        {
            let mut stmt = tx.prepare(
                "INSERT INTO mbo_messages 
                 (ts_recv, ts_event, instrument_id, publisher, order_id, action, side, 
                  price, size, flags, sequence, ts_in_delta, channel_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)"
            ).context("Failed to prepare statement")?;

            for msg in messages {
                let action = format!("{:?}", msg.action);
                let side = format!("{:?}", msg.side);
                
                stmt.execute(params![
                    msg.hd.ts_event as i64,
                    msg.ts_recv as i64,
                    msg.hd.instrument_id,
                    msg.hd.publisher_id,
                    msg.order_id as i64,
                    action,
                    side,
                    msg.price,
                    msg.size,
                    msg.flags.raw(),
                    msg.sequence,
                    msg.ts_in_delta,
                    msg.channel_id,
                ]).context("Failed to execute insert statement")?;
            }
        }

        tx.commit().context("Failed to commit transaction")?;
        debug!("Inserted batch of {} MBO messages", messages.len());
        
        Ok(())
    }

    pub fn count_messages(&self) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM mbo_messages",
            [],
            |row| row.get(0)
        ).context("Failed to count messages")?;
        
        Ok(count as usize)
    }

    #[allow(dead_code)]
    pub fn get_messages_for_instrument(
        &self,
        instrument_id: u32,
        start_ts: Option<i64>,
        end_ts: Option<i64>,
        limit: Option<usize>,
    ) -> Result<Vec<(i64, String, String, i64, u32)>> {
        let conn = self.conn.lock().unwrap();
        
        let mut query = String::from(
            "SELECT ts_recv, action, side, price, size 
             FROM mbo_messages 
             WHERE instrument_id = ?1"
        );
        
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(instrument_id)];
        
        if let Some(start) = start_ts {
            query.push_str(" AND ts_recv >= ?");
            params_vec.push(Box::new(start));
        }
        
        if let Some(end) = end_ts {
            query.push_str(" AND ts_recv <= ?");
            params_vec.push(Box::new(end));
        }
        
        query.push_str(" ORDER BY ts_recv DESC");
        
        if let Some(lim) = limit {
            query.push_str(" LIMIT ?");
            params_vec.push(Box::new(lim));
        }
        
        let mut stmt = conn.prepare(&query)
            .context("Failed to prepare query")?;
        
        let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter()
            .map(|p| p.as_ref())
            .collect();
        
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        }).context("Failed to query messages")?;
        
        rows.collect::<Result<Vec<_>, _>>()
            .context("Failed to collect query results")
    }
}