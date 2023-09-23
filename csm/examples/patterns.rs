use csm::csm;

// patterns are macros defined by you that internally uses the csm! macro
macro_rules! circle {
    ($id:ident, $size:expr) => {
        csm! { $id,
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
    let classes = circle!(foo_id, 5rem);
    println!("{:?}", classes);
    // -> "justify_center items_center flex_0_0_auto w_5rem d_flex rounded_9999px overflow_hidden h_5rem"

    // and as a side-effect, a bundled CSS file is generated in ./target/csm/bundle.css:
    //
    // .justify_center {
    //   justify-content: center;
    // }
    //
    // .items_center {
    //   align-items: center;
    // }
    //
    // .flex_0_0_auto {
    //   flex: none;
    // }
    //
    // .w_5rem {
    //   width: 5rem;
    // }
    //
    // .d_flex {
    //   display: flex;
    // }
    //
    // .rounded_9999px {
    //   border-radius: 9999px;
    // }
    //
    // .overflow_hidden {
    //   overflow: hidden;
    // }
    //
    // .h_5rem {
    //   height: 5rem;
    // }
}
