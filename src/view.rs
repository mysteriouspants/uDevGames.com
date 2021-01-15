use std::sync::Arc;
use lazy_static::lazy_static;
use arc_swap::ArcSwapOption;
use serde::Serialize;
use tera::{Context, Tera};


lazy_static! {
    static ref TERA: ArcSwapOption<Tera> = ArcSwapOption::empty();
}

pub fn init_tera() {
    let mut t = Tera::new("templates/**/*.tera")
        .expect("Unable to construct Tera template engine");
    t.autoescape_on(vec![".html", "html.tera"]);
    TERA.store(Some(Arc::new(t)));
}

pub fn render_template(tname: &str, ctxt: &impl Serialize) -> String {
    let t = TERA.load();
    t.iter().next()
        .expect("The template system is uninitialized")
        .render(tname, &Context::from_serialize(ctxt).unwrap())
        .expect("Could not render template")
}
