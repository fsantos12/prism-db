mod comparison;

use std::{cmp::Ordering, collections::HashMap, sync::{Arc, RwLock}};
use async_trait::async_trait;
use rand::seq::SliceRandom;
use rand::rngs::ThreadRng;

use crate::{
    driver::{driver::Driver, memory::comparison::{strict_eq, strict_partial_cmp}}, 
    query::{DeleteQuery, FindQuery, InsertQuery, UpdateQuery, filters::{Filter, FilterDefinition}, sorts::{Sort, SortDefinition}}, 
    types::{DbError, DbRow, DbValue}
};

#[derive(Default, Clone)]
pub struct MemoryDriver {
    /// Internal storage: Thread-safe map of collections to rows.
    storage: Arc<RwLock<HashMap<String, Vec<DbRow>>>>,
}

impl MemoryDriver {
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Evaluates the entire FilterDefinition (implicit AND) against a row.
    fn matches_filter(&self, row: &DbRow, filters: &FilterDefinition) -> bool {
        filters.iter().all(|f| self.evaluate_node(row, f))
    }

    /// Recursively traverses the Filter AST.
    fn evaluate_node(&self, row: &DbRow, filter: &Filter) -> bool {
        match filter {
            // --- Null Checks ---
            Filter::IsNull(field) => row.get(field).map_or(true, |v| v.is_null()),
            Filter::IsNotNull(field) => row.get(field).map_or(false, |v|!v.is_null()),

            // --- Comparisons (Uses strict_eq/strict_partial_cmp) ---
            Filter::Eq(field, val) => row.get(field).is_some_and(|rv| strict_eq(rv, val)),
            Filter::Gt(field, val) => row.get(field).is_some_and(|rv| strict_partial_cmp(rv, val) == Some(Ordering::Greater)),
            Filter::Lt(field, val) => row.get(field).is_some_and(|rv| strict_partial_cmp(rv, val) == Some(Ordering::Less)),

            // --- Recursive Nodes (Handles Boxed collections) ---
            Filter::And(def) => def.iter().all(|f| self.evaluate_node(row, f)),
            Filter::Or(def) => def.iter().any(|f| self.evaluate_node(row, f)),
            Filter::Not(f) =>!self.evaluate_node(row, f),

            // --- Advanced Logic ---
            Filter::Between(field, range) => {
                let (low, high) = &**range; // Double deref for the Boxed tuple
                row.get(field).is_some_and(|rv| {
                    let gte = matches!(strict_partial_cmp(rv, low), Some(Ordering::Greater | Ordering::Equal));
                    let lte = matches!(strict_partial_cmp(rv, high), Some(Ordering::Less | Ordering::Equal));
                    gte && lte
                })
            },
            Filter::In(field, vals) => row.get(field).is_some_and(|rv| vals.iter().any(|v| strict_eq(rv, v))),
            
            Filter::Contains(field, val) => {
                if let (Some(DbValue::String(Some(text))), DbValue::String(Some(sub))) = (row.get(field), val) {
                    text.contains(&**sub)
                } else { false }
            },
            _ => false,
        }
    }

    fn apply_sorts(&self, rows: &mut Vec<DbRow>, sorts: &SortDefinition) {
        if sorts.is_empty() { return; }
        
        // Handle Shuffle for Random sorting [2]
        if sorts.iter().any(|s| matches!(s, Sort::Random)) {
            rows.shuffle(&mut ThreadRng::default());
            if sorts.len() == 1 { return; }
        }

        rows.sort_by(|a, b| {
            for sort in sorts {
                let (field, is_asc, nulls_first) = match sort {
                    Sort::Asc(f) => (f, true, false),
                    Sort::Desc(f) => (f, false, true),
                    Sort::AscNullsFirst(f) => (f, true, true),
                    Sort::DescNullsLast(f) => (f, false, false),
                    Sort::Random => continue,
                    _ => continue,
                };

                let cmp = match (a.get(field), b.get(field)) {
                    (None, None) => Ordering::Equal,
                    (None, Some(_)) => if nulls_first { Ordering::Less } else { Ordering::Greater },
                    (Some(_), None) => if nulls_first { Ordering::Greater } else { Ordering::Less },
                    (Some(v1), Some(v2)) => strict_partial_cmp(v1, v2).unwrap_or(Ordering::Equal),
                };

                if cmp!= Ordering::Equal {
                    return if is_asc { cmp } else { cmp.reverse() };
                }
            }
            Ordering::Equal
        });
    }
}

#[async_trait]
impl Driver for MemoryDriver {
    async fn find(&self, query: FindQuery) -> Result<Vec<DbRow>, DbError> {
        let storage = self.storage.read().map_err(|_| DbError::ConcurrencyError("Poisoned lock".into()))?;
        let table = storage.get(&query.collection).ok_or(DbError::NotFound)?;

        let mut results: Vec<DbRow> = table.iter()
           .filter(|row| self.matches_filter(row, &query.filters))
           .cloned()
           .collect();

        self.apply_sorts(&mut results, &query.sorts);

        // Pagination Logic
        let offset = query.offset.unwrap_or(0);
        let iter = results.into_iter().skip(offset);
        
        Ok(if let Some(limit) = query.limit {
            iter.take(limit).collect()
        } else {
            iter.collect()
        })
    }

    async fn insert(&self, query: InsertQuery) -> Result<u64, DbError> {
        let mut storage = self.storage.write().map_err(|_| DbError::ConcurrencyError("Poisoned lock".into()))?;
        let table = storage.entry(query.collection).or_default();
        let len = query.values.len() as u64;
        table.extend(query.values);
        Ok(len)
    }

    async fn update(&self, query: UpdateQuery) -> Result<u64, DbError> {
        let mut storage = self.storage.write().map_err(|_| DbError::ConcurrencyError("Poisoned lock".into()))?;
        let table = storage.get_mut(&query.collection).ok_or(DbError::NotFound)?;
        
        let mut count = 0;
        for row in table.iter_mut() {
            if self.matches_filter(row, &query.filters) {
                for (k, v) in &query.updates.0 {
                    row.insert(k.clone(), v.clone());
                }
                count += 1;
            }
        }
        Ok(count)
    }

    async fn delete(&self, query: DeleteQuery) -> Result<u64, DbError> {
        let mut storage = self.storage.write().map_err(|_| DbError::ConcurrencyError("Poisoned lock".into()))?;
        let table = storage.get_mut(&query.collection).ok_or(DbError::NotFound)?;
        
        let initial_len = table.len();
        table.retain(|row|!self.matches_filter(row, &query.filters));
        Ok((initial_len - table.len()) as u64)
    }

    // --- Transactional Stubs ---
    async fn transaction_begin(&self) -> Result<(), DbError> { Ok(()) }
    async fn transaction_commit(&self) -> Result<(), DbError> { Ok(()) }
    async fn transaction_rollback(&self) -> Result<(), DbError> { Ok(()) }
}