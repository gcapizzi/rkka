use anyhow::Result;
use redis::Commands;

#[test]
fn test_set_and_get() -> Result<()> {
    let client = redis::Client::open("redis://127.0.0.1/")?;
    println!("Connecting to Redis...");
    let mut con = client.get_connection()?;
    println!("Setting value...");
    let _: () = con.set("my_key", 42)?;
    println!("Getting value...");
    let value: i32 = con.get("my_key")?;
    assert_eq!(42, value);
    Ok(())
}
