fn main() {
    let client = redis::Client::open("redis://localhost:6379").unwrap();
    let mut con = client.get_connection().unwrap();

    // 原子加锁：SET key value NX PX ttl
    let result: Option<String> = redis::cmd("SET")
        .arg("lock:my_key")
        .arg("123456")
        .arg("NX")
        .arg("PX")
        .arg(10000)
        .query(&mut con)
        .unwrap();

    println!("{:?}", result);

    let result: Option<String> = redis::cmd("SET")
        .arg("lock:my_key")
        .arg("123456123")
        .arg("NX")
        .arg("PX")
        .arg(10000)
        .query(&mut con)
        .unwrap();

    println!("{:?}", result);
}
