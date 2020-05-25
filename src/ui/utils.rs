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

#[macro_export]
macro_rules! within_tooltip {
    ($ui:expr => $block:block) => {
        let token = $ui.begin_tooltip();
        $block;
        token.end($ui);
    };
}

#[macro_export]
macro_rules! styled {
    ($style:expr, $ui:expr => $block:block) => {
        let token = $ui.push_style_var($style);
        $block;
        token.pop($ui);
    };
}
