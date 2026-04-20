use std::time::Instant;

use simple_db::DbContext;

type TestFunction = fn(context: &DbContext) -> bool;

pub struct TestCase {
    name: &'static str,
    test_fn: TestFunction,
    initial_time: Option<Instant>,
    final_time: Option<Instant>,
    result: Option<bool>,
}

impl TestCase {
    pub fn new(name: &'static str, test_fn: TestFunction) -> Self {
        Self {
            name: name,
            test_fn: test_fn,
            initial_time: None,
            final_time: None,
            result: None,
        }
    }

    pub fn run(&mut self, context: &DbContext) {
        let start = Instant::now();
        self.initial_time = Some(start);

        self.result = Some((self.test_fn)(context));

        let end = Instant::now();
        self.final_time = Some(end);
    }

    pub fn print(&self) {
        if self.result.is_none() {
            println!("Test '{}' has not been run yet.", self.name);
            return;
        }
        println!(
            "🧪 {} | {} | ⏱️ {:?}",
            self.name,
            if self.result.unwrap() { "✅ Passed" } else { "❌ Failed" },
            self.final_time.unwrap() - self.initial_time.unwrap()
        );
    }
}

const TESTS: &[TestCase] = &[
    TestCase::new("single insert", single_insert_test),
    TestCase::new("bulk insert", bulk_insert_test),
];

// Insert
fn single_insert_test(context: &DbContext) -> bool {
    true
}

fn bulk_insert_test(context: &DbContext) -> bool {
    true
}