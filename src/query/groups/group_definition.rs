/// Represents the GROUP BY clause of a query.
/// It maintains a list of fields used to aggregate the result set.
#[derive(Debug, Clone, Default)]
pub struct GroupDefinition {
    /// The collection of field names for grouping.
    fields: Vec<String>,
}

impl GroupDefinition {
    /// Creates a new, empty GroupDefinition.
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
        }
    }

    /// Adds a field to the grouping definition.
    /// Uses Into<String> to accept both &str and String.
    pub fn field<F: Into<String>>(mut self, field: F) -> Self {
        self.fields.push(field.into());
        self
    }

    /// Adds multiple fields at once.
    pub fn fields<F, I>(mut self, fields: I) -> Self 
    where F: Into<String>, I: IntoIterator<Item = F>{
        for f in fields {
            self.fields.push(f.into());
        }
        self
    }

    /// Checks if the definition has any fields.
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Returns the number of fields in the definition.
    pub fn len(&self) -> usize {
        self.fields.len()
    }
}

/// Allows drivers to iterate over the grouping fields easily.
impl IntoIterator for GroupDefinition {
    type Item = String;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.fields.into_iter()
    }
}

/// Allows reference iteration for internal processing.
impl<'a> IntoIterator for &'a GroupDefinition {
    type Item = &'a String;
    type IntoIter = std::slice::Iter<'a, String>;
    fn into_iter(self) -> Self::IntoIter {
        self.fields.iter()
    }
}