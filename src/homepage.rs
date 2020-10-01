use crate::template_context::{ UserOptional, UserOptionalContext };
use rocket::get;
use rocket_contrib::templates::Template;
use serde::Serialize;


#[get("/")]
pub fn homepage(user: UserOptional) -> Template {
    #[derive(Debug, Serialize)]
    struct Context {
        auth: UserOptionalContext
    }

    let context = Context { auth: user.to_context() };

    println!("context: {:?}", context);

    Template::render("homepage", &context)
}
