
pub const TEST_JS: &str = r#"
console.log("Hello, World!");
var timeoutId = setTimeout(() => {
  console.log("Delayed for 1 second.");
}, 1000);
//clearTimeout(timeoutId);
"#;

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    
    let _guard = rt.enter();
    
    blitz::run(TEST_JS);
}
