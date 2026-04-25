fn main() {
    for (path, dev) in evdev::enumerate() {
        println!("{:?} {:?}", path, dev.name());
    }
}
