use tera::Tera;

pub fn init_tera() -> Tera {
    let mut t = Tera::new("templates/**/*.tera")
        .expect("Unable to construct Tera template engine");
    t.autoescape_on(vec![".html", "html.tera"]);
    t
}
