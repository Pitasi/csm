use csm::csm;
use rscx::{component, html, props, EscapeAttribute};

#[tokio::main]
async fn main() {
    println!("{}", app().await);

    // (pretty-printed) ->
    //
    // <!DOCTYPE html>
    // <head>
    //   <link rel="stylesheet" href="/bundle.css" />
    // </head>
    // <body>
    //   <main>
    //     <p class="background_red p_2rem">
    //       hello
    //     </p>
    //     <button class="background_red color_white">
    //       click me
    //     </button>
    //   </main>
    // </body>
    //
    //
    // and the content of ./target/csm/bundle.css is:
    //
    // .p_2rem {
    //   padding: 2rem;
    // }
    //
    // .background_red {
    //   background: red;
    // }
    //
    // .color_white {
    //   color: #fff;
    // }
}

async fn app() -> String {
    html! {
        <!DOCTYPE html>
        <head>
            <link rel="stylesheet" href="/bundle.css" />
            // Caution! You need to manually copy ./target/csm/bundle.css to somewhere public after
            // Rust build and serve it from your webserver!
        </head>
        <body>
            <main>
                <RedText>hello</RedText>
                <Button>click me</Button>
            </main>
        </body>
    }
}

#[component]
async fn Button(children: String) -> String {
    html! {
        <button class={csm! { Button,
            background: red,
            color: white,
        }}>
            {children}
        </button>
    }
}

#[component]
async fn RedText(children: String) -> String {
    html! {
        <p class={csm! { RedText,
            // we use again "background: red" to show that the final CSS bundle will only contain
            // it once
            background: red,
            padding: 2rem,
        }}>
            {children}
        </p>
    }
}
