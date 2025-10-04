
fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    
    let _guard = rt.enter();
    
    blitz::run_event_loop();
}
