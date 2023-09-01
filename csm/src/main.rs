use csm_macros::{csm, csm_defs, recipe};

macro_rules! container {
    ($max_width:expr) => {
        csmcss_macro::csm! {
            max-width: $max_width,
            margin-left: auto,
            margin-right: auto,
        }
    };
}

macro_rules! circle {
    ($size:expr) => {
        csm! {
            display: flex,
            align-items: center,
            justify-content: center,
            flex: 0 0 auto,
            width: $size,
            height: $size,
            border-radius: 9999px,
            overflow: hidden,
        }
    };
}

recipe! {
    button,
    base: {
        display: flex,
    },
    variants: {
        visual: {
            solid: {
                background-color: $danger,
                color: white,
            },
            outline: {
                border-width: 1px,
                border-color: $danger,
            },
        },
        size: {
            sm: {
                padding: 4,
                font-size: 12px,
            },
            lg: {
                padding: 8,
                font-size: 24px,
            },
        },
    },
}

fn main() {
    let root_css = csm_defs! {
        // tokens
        red: #EE0F0F,

        // semantic tokens
        danger: $red,
    };

    let (classes, css) = button!("solid", "lg");

    let res = format!(
        r#"
    <style>
    {root_css}
    </style>
    <style>
    {css}
    </style>
    <button class="{classes}">
        Heya
    </button>
        "#
    );

    println!("{}", res);
}
