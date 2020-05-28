#[macro_export]
macro_rules! read_modified {
    ($storage:expr => $reader:expr => $bitset:expr) => {
        $bitset.clear();
        for event in $storage.channel().read($reader) {
            match event {
                ComponentEvent::Modified(id) => {
                    $bitset.add(*id);
                },
                _ => (),
            };
        }
    }
}
