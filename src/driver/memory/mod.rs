mod comparison;

use std::{cmp::Ordering, collections::HashMap, sync::{Arc, RwLock}};

use async_trait::async_trait;

use rand::seq::SliceRandom;
use rand::rngs::ThreadRng;

use crate::{driver::{driver::{DbResult, DbRow, Driver}, memory::comparison::{strict_eq, strict_partial_cmp}}, query::{DeleteQuery, FindQuery, InsertQuery, UpdateQuery, filters::{Filter, FilterDefinition}, sorts::{Sort, SortDefinition}}, value::DbValue};

/// An In-Memory database driver that stores data in HashMaps and Vectors.
#[derive(Default, Clone)]
pub struct MemoryDriver {
    /// storage maps "collection_name" -> Vec<DbRow>
    storage: Arc<RwLock<HashMap<String, Vec<DbRow>>>>,
}

impl MemoryDriver {
    /// Creates a new instance of the MemoryDriver.
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// A helper filter functions
    fn matches_filter(&self, row: &DbRow, filter: FilterDefinition) -> bool {
        if filter.is_empty() { return true; }
        filter.into_iter().all(|f| self.evaluate_filter(row, &f))
    }

    fn evaluate_filter(&self, row: &DbRow, filter: &Filter) -> bool {
        match filter {
            // --- Null Checks ---
            Filter::IsNull(field) => row.get(field).is_none_or(|row_val| row_val.is_null()),
            Filter::IsNotNull(field) => row.get(field).is_some_and(|row_val| !row_val.is_null()),

            // --- Basic Comparisons (Usando as funções estritas) ---
            Filter::Eq(field, val) => row.get(field).is_some_and(|row_val| strict_eq(row_val, val)),
            Filter::Neq(field, val) => row.get(field).is_some_and(|row_val| !strict_eq(row_val, val)),
            Filter::Lt(field, val) => row.get(field).is_some_and(|row_val| strict_partial_cmp(row_val, val) == Some(Ordering::Less)),
            Filter::Lte(field, val) => row.get(field).is_some_and(|row_val| matches!(strict_partial_cmp(row_val, val), Some(Ordering::Less | Ordering::Equal))),
            Filter::Gt(field, val) => row.get(field).is_some_and(|row_val| strict_partial_cmp(row_val, val) == Some(Ordering::Greater)),
            Filter::Gte(field, val) => row.get(field).is_some_and(|row_val| matches!(strict_partial_cmp(row_val, val), Some(Ordering::Greater | Ordering::Equal))),

            // --- Pattern Matching ---
            Filter::StartsWith(field, val) => {
                if let (Some(DbValue::String(Some(text))), DbValue::String(Some(prefix))) = (row.get(field), val) {
                    text.starts_with(prefix)
                } else {
                    false
                }
            },
            Filter::NotStartsWith(field, val) => !self.evaluate_filter(row, &Filter::StartsWith(field.clone(), val.clone())),
            Filter::EndsWith(field, val) => {
                if let (Some(DbValue::String(Some(text))), DbValue::String(Some(suffix))) = (row.get(field), val) {
                    text.ends_with(suffix)
                } else {
                    false
                }
            },
            Filter::NotEndsWith(field, val) => !self.evaluate_filter(row, &Filter::EndsWith(field.clone(), val.clone())),
            Filter::Contains(field, val) => {
                if let (Some(DbValue::String(Some(text))), DbValue::String(Some(substr))) = (row.get(field), val) {
                    text.contains(substr)
                } else {
                    false
                }
            },
            Filter::NotContains(field, val) => !self.evaluate_filter(row, &Filter::Contains(field.clone(), val.clone())),

            // --- Regex Matching ---
            Filter::Regex(field, val) => {
                if let Some(DbValue::String(Some(text))) = row.get(field) {
                    regex::Regex::new(val).is_ok_and(|re| re.is_match(text))
                } else {
                    false
                }
            },

            // --- Range Checks ---
            Filter::Between(field, val1, val2) => row.get(field).is_some_and(|row_val| {
                let gte_val1 = matches!(strict_partial_cmp(row_val, val1), Some(Ordering::Greater | Ordering::Equal));
                let lte_val2 = matches!(strict_partial_cmp(row_val, val2), Some(Ordering::Less | Ordering::Equal));
                gte_val1 && lte_val2
            }),
            Filter::NotBetween(field, val1, val2) => !self.evaluate_filter(row, &Filter::Between(field.clone(), val1.clone(), val2.clone())),

            // --- Set Membership ---
            Filter::In(field, vals) => row.get(field).is_some_and(|row_val| {
                vals.iter().any(|v| strict_eq(row_val, v))
            }),
            Filter::NotIn(field, vals) => !self.evaluate_filter(row, &Filter::In(field.clone(), vals.clone())),

            // --- Logical Operators ---
            Filter::And(filters) => filters.iter().all(|f| self.evaluate_filter(row, f)),
            Filter::Or(filters) => filters.iter().any(|f| self.evaluate_filter(row, f)),
            Filter::Not(filter) => !self.evaluate_filter(row, filter),
        }
    }

    /// A helper sort function
    fn apply_sorts(&self, rows: &mut [DbRow], sorts: &SortDefinition) {
        if sorts.is_empty() { return; }

        if sorts.into_iter().any(|s| matches!(s, Sort::Random)) {
            let mut rng = ThreadRng::default();
            rows.shuffle(&mut rng);
            if sorts.len() == 1 { return; }
        }

        rows.sort_by(|row_a, row_b| {
            for sort in sorts.into_iter() {
                let (field, is_asc, nulls_first) = match sort {
                    Sort::Asc(f) => (f, true, false),
                    Sort::Desc(f) => (f, false, true),
                    Sort::AscNullsFirst(f) => (f, true, true),
                    Sort::AscNullsLast(f) => (f, true, false),
                    Sort::DescNullsFirst(f) => (f, false, true),
                    Sort::DescNullsLast(f) => (f, false, false),
                    Sort::Random => continue, 
                };

                let val_a = row_a.get(field);
                let val_b = row_b.get(field);

                let cmp = match (val_a, val_b) {
                    (None, None) => Ordering::Equal,
                    (None, Some(_)) => if nulls_first { Ordering::Less } else { Ordering::Greater },
                    (Some(_), None) => if nulls_first { Ordering::Greater } else { Ordering::Less },
                    (Some(a), Some(b)) => {
                        strict_partial_cmp(a, b).unwrap_or(Ordering::Equal)
                    }
                };

                if cmp != Ordering::Equal {
                    return if is_asc { cmp } else { cmp.reverse() };
                }
            }

            Ordering::Equal
        });
    }
}

#[async_trait]
impl Driver for MemoryDriver {
    async fn find(&self, query: FindQuery) -> DbResult<Vec<DbRow>> {
        // 1. Get the table (Read Lock allows concurrent reads)
        let storage = self.storage.read().unwrap();
        let Some(table) = storage.get(&query.collection) else {
            // Return an empty vector if the collection doesn't exist yet
            return Ok(Vec::new()); 
        };

        // 2. Filter (WHERE clause)
        let mut results: Vec<DbRow> = table
            .iter()
            .filter(|row| self.matches_filter(row, query.filters.clone()))
            .cloned()
            .collect();

        // 3. Sort (ORDER BY clause)
        self.apply_sorts(&mut results, &query.sorts);

        // 4. Paginate (OFFSET & LIMIT)
        let offset = query.offset.unwrap_or(0);
        let iter = results.into_iter().skip(offset);
        
        let paginated_results = if let Some(limit) = query.limit {
            iter.take(limit).collect()
        } else {
            iter.collect()
        };

        // Note: Projections (SELECT) and Groups (GROUP BY) are intentionally 
        // ignored in this in-memory driver to prioritize speed and simplicity.

        Ok(paginated_results)
    }

    async fn insert(&self, query: InsertQuery) -> DbResult<u64> {
        // 1. Write Lock (blocks other reads/writes while inserting)
        let mut storage = self.storage.write().unwrap();
        
        // 2. Get the existing table or create a new one if it doesn't exist
        let table = storage.entry(query.collection).or_insert_with(Vec::new);
        let count = query.values.len() as u64;

        // 3. Convert the Vec<(String, DbValue)> from the query into your DbRow type
        for row_data in query.values {
            let mut new_row = DbRow::default(); 
            for (field, value) in row_data {
                new_row.insert(field, value);
            }
            table.push(new_row);
        }

        Ok(count)
    }

    async fn update(&self, query: UpdateQuery) -> DbResult<u64> {
        // 1. Acquire a Write Lock to modify the storage
        let mut storage = self.storage.write().unwrap();
        
        // 2. Check if the collection exists; if not, nothing to update
        let Some(table) = storage.get_mut(&query.collection) else {
            return Ok(0);
        };

        let mut updated_count = 0;

        // 3. Iterate through all rows in the table as mutable references
        for row in table.iter_mut() {
            // 4. Use the helper to check if the row satisfies the WHERE clause
            if self.matches_filter(row, query.filters.clone()) {
                // 5. Apply each update transformation to the matching row
                for (field, new_value) in &query.updates {
                    // Update existing field or insert new one
                    row.insert(field.clone(), new_value.clone());
                }
                updated_count += 1;
            }
        }

        Ok(updated_count)
    }

    async fn delete(&self, query: DeleteQuery) -> DbResult<u64> {
        // 1. Write Lock
        let mut storage = self.storage.write().unwrap();
        let Some(table) = storage.get_mut(&query.collection) else {
            return Ok(0); // Nothing to delete
        };

        let initial_len = table.len();

        // 2. 'retain' keeps only the elements where the closure returns true.
        // Since we want to DELETE the ones that match the filter, we negate it with `!`.
        table.retain(|row| !self.matches_filter(row, query.filters.clone()));

        let deleted_count = (initial_len - table.len()) as u64;

        Ok(deleted_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Using the full tokio runtime for the test
    #[tokio::test]
    async fn test_find_with_sorting() {
        let driver = MemoryDriver::new();
        let col = "test_collection";

        // Insert mock data
        driver.insert(InsertQuery::new(col)
            .insert(vec![("name", DbValue::String(Some("Zebra".to_string()))), ("score", DbValue::I32(Some(10)))])
            .insert(vec![("name", DbValue::String(Some("Apple".to_string()))), ("score", DbValue::I32(Some(50)))])
        ).await.unwrap();

        // Query: Find all, sort by name ASC
        let query = FindQuery {
            collection: col.to_string(),
            filters: FilterDefinition::empty().eq("name", "Apple"),
            sorts: SortDefinition::empty().asc("name"),
            projections: Default::default(),
            groups: Default::default(),
            limit: None,
            offset: None,
        };

        let results = driver.find(query).await.unwrap();
        print!("Results: {:#?}", results);
        assert_eq!(results.len(), 1);
        
        // Ensure "Apple" comes before "Zebra"
        let first_name = results[0].get("name");
        
        // Use matches! to avoid ArchivedOption issues if DbValue is standard
        assert!(matches!(first_name, Some(DbValue::String(Some(s))) if s == "Apple"));
    }
}