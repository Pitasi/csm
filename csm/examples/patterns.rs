use csm::csm;

// patterns are macros defined by you that internally uses the csm! macro
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

fn main() {
    let css = circle!(5rem);
    // this will expand into:
    // let css = {
    //     [
    //         ("w_5rem", ".w_5rem { width: 5rem; }"),
    //         ("d_flex", ".d_flex { display: flex; }"),
    //         ("items_center", ".items_center { align-items: center; }"),
    //         ("h_5rem", ".h_5rem { height: 5rem; }"),
    //         ("flex_0_0_auto", ".flex_0_0_auto { flex: 0 0 auto; }"),
    //         ("overflow_hidden", ".overflow_hidden { overflow: hidden; }"),
    //         ("rounded_9999px", ".rounded_9999px { border-radius: 9999px; }"),
    //         ("justify_center", ".justify_center { justify-content: center; }"),
    //     ]
    // };

    println!("{:?}", css);
}
