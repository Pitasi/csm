use csm::{csm, csm_defs};

fn main() {
    // csm_defs is a convenient way to define CSS variables
    let css = csm_defs! {
        // define a new variable called red
        red: #FF0000,

        // dollar sign is used to reference a CSS variable
        danger: $red,
    };
    // this will expand into:
    // let css = ":root {--red: #FF0000;--danger: var(--red);}";
    println!("{:?}", css);

    // then, variables can be used in normal csm macro calls:
    let css2 = csm! {
        color: $red,
    };
    // this will expand into:
    // let css2 = [("color_red", ".color_red { color: var(--red); }")];
    println!("{:?}", css2);

    // note that csm and csm_defs do not keep any internal state, they expand
    // dollar signs into var(--var_name) but there's no guarantee that the
    // variable was defined before
}
