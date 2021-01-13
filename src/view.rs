use lazycell::AtomicLazyCell;
use tera::Tera;


/// Single global Tera template bean.
static TERA: AtomicLazyCell<Tera> = AtomicLazyCell::new();

/// Initializes the Tera bean. This ought to be called exactly once, before any
/// templates are rendered.
pub fn init_tera() {
    let tera = match Tera::new("templates/**/*") {
        Ok(t) => t,
        Err(e) => {
            println!("Error parsing template: {}", e);
            std::process::exit(-1);
        }
    };
    TERA.fill(tera);
}

/// Gets the Tera bean. Panics if this is called before `init_tera`.
pub fn tera() -> Tera {
    TERA.get().expect("Template system not yet initialized.")
}
