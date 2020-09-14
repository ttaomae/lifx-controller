#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

mod controller;

use rocket::{response::content::Json, State};

use controller::{Devices, LifxController};

#[get("/lights?<refresh>")]
fn get_lights(controller: State<LifxController>, refresh: bool) -> Json<Devices> {
    if refresh {
        Json(controller.discover())
    } else {
        Json(controller.get_lights())
    }
}

fn main() {
    rocket::ignite()
        .manage(LifxController::new())
        .mount("/", routes![get_lights])
        .launch();
}
