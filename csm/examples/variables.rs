use csm::{csm, csm_defs};

fn main() {
    // csm_defs is a convenient way to define CSS variables for :root
    csm_defs! {
        // define a new variable called red
        red: #FF0000,

        // dollar sign is used to reference a CSS variable
        danger: $red,
    };

    // then, variables can be used in normal csm!{} calls:
    let css = csm! { some_id,
        color: $danger,
    };
    println!("{:?}", css);
    // -> "color_danger"

    // and the CSS bundle will contain:
    //
    // .color_danger {
    //   color: var(--danger);
    // }
    //
    // :root {
    //   --danger: var(--red);
    //   --red: red;
    // }

    // note that when using $danger, it gets expanded to var(--danger), but there's no guarantee
    // that --danger is defined (i.e. you won't get a compile time error)
}
