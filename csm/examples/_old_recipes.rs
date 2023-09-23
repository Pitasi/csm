use csm::recipe;

// this recipe for button generates a button!(...) macro that expands into a
// csm! macro, combining the values from the selected variants
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
    let css = button!("solid", "lg");
    // this will expand into:
    // let css = (
    //     "button__solid_lg",
    //     ".button__solid_lg {\ndisplay: flex;\ncolor: white;\nbackground-color: var(--danger);\nfont-size: 24px;\npadding: 8;\n}",
    // );

    println!("{:?}", css);
}
