# Gravity — sample project for demo analysis

fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn is_even(n: i32) -> bool {
    if n % 2 == 0 {
        true
    } else {
        false
    }
}

pub fn process(values: &[i32]) -> Vec<i32> {
    values.iter().filter(|&&v| is_even(v)).copied().collect()
}

pub async fn fetch_data(url: &str) -> Result<String, String> {
    if url.is_empty() {
        return Err("Empty URL".into());
    }
    Ok(format!("Data from {url}"))
}

struct Config {
    debug: bool,
    timeout: u32,
}

impl Config {
    pub fn new(debug: bool, timeout: u32) -> Self {
        Config { debug, timeout }
    }

    pub fn is_debug(&self) -> bool {
        self.debug
    }
}
