use std::{cell::RefCell, collections::HashMap, rc::Rc};

use csm::csm;
use rscx::{
    component,
    context::{expect_context, provide_context, spawn_local},
    html, props,
};
use tokio::task::LocalSet;

/// This examples is slightly more complicated and provides a starting point for the integration
/// with other UI frameworks, in this case, RSCx.
///
/// The flow is:
///   1. We render the "app", which is going to be the actual body of the page.
///   2. While rendering the app, we collect the rendered style in a global HashMap, stored in the
///      cotext.
///   3. We render the "shell", which is the HTML page (html, head, and body tags). The head will
///      contain a <style> tag with the de-duplicated result of the collected styles. The body will
///      contain the rendered app.

#[derive(Clone)]
/// StyleCtx is a shared global context that will collect and deduplicate CSS classes rendered by
/// csm.
struct StyleCtx {
    map: Rc<RefCell<HashMap<String, String>>>,
}

impl StyleCtx {
    /// Add new CSS classes to the context, and return a string with the classes separated by a
    /// space, ready to be used in the `class` attribute of the HTML element.
    fn append(&self, styles: &[(&str, &str)]) -> String {
        self.map
            .borrow_mut()
            .extend(styles.iter().map(|(k, v)| (k.to_string(), v.to_string())));
        let mut res = String::with_capacity(
            styles.len() + styles.iter().map(|(k, _)| k.len()).sum::<usize>(),
        );
        for (k, _) in styles {
            res.push_str(k);
            res.push_str(" ");
        }
        res
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let local = LocalSet::new();
    let res = local
        .run_until(async move {
            spawn_local(async {
                // provide a context of type String
                let style_cx = StyleCtx {
                    map: Rc::new(RefCell::new(HashMap::new())),
                };
                let style_cx2 = style_cx.clone();
                provide_context(style_cx);

                // 1. Render the app (and collect the styles)
                let rendered_app = app().await;

                // 2. Extract the rendered style from the context
                let rendered_style = {
                    let len = style_cx2.map.borrow().iter().map(|(_, v)| v.len()).sum();
                    let mut res = String::with_capacity(len);
                    for (_, v) in style_cx2.map.borrow().iter() {
                        res.push_str(v);
                    }
                    res
                };

                // 3. Render the shell
                shell(rendered_style, rendered_app).await
            })
            .await
        })
        .await
        .unwrap();

    println!("{}", res);
    Ok(())
}

async fn shell(s: String, body: String) -> String {
    html! {
        <!DOCTYPE html>
        <head>
            <style>{s}</style>
            // the final CSS will be:
            //   .p_2rem { padding: 2rem; }
            //   .background_red { background: red; }
            //   .color_white { color: white; }
        </head>
        <body>
            {body}
        </body>
    }
}

async fn app() -> String {
    html! {
        <main>
            <RedText>hello</RedText>
            <Button>click me</Button>
        </main>
    }
}

#[props]
pub struct ButtonProps {
    pub children: String,
}

#[component]
async fn Button(props: ButtonProps) -> String {
    let style_ctx = expect_context::<StyleCtx>();
    let classes = style_ctx.append(&csm! {
        background: red,
        color: white,
    });
    html! {
        <button class={classes}>
            {props.children}
        </button>
    }
}

#[props]
pub struct RedTextProps {
    pub children: String,
}

#[component]
async fn RedText(props: RedTextProps) -> String {
    let style_ctx = expect_context::<StyleCtx>();
    let classes = style_ctx.append(&csm! {
        // we use again background: red to show that the final CSS in the <head> will only contain
        // it once
        background: red,
        padding: 2rem,
    });
    html! {
        <p class={classes}>
            {props.children}
        </p>
    }
}
