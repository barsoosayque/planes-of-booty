#[macro_export]
macro_rules! within_window {
    ($builder:expr, $ui:expr => $block:block) => {
        if let Some(token) = $builder.begin($ui) {
            $block;
            token.end($ui);
        }
    };
}

#[macro_export]
macro_rules! within_group {
    ($ui:expr => $block:block) => {
        let token = $ui.begin_group();
        $block;
        token.end($ui);
    };
}
