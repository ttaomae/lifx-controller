#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

mod controller;
mod forms;

use forms::{Brightness, Duration, Temperature, Hsb};
use rocket::{request::Form, response::content::Json, State, Config, config::Environment};

use controller::{Devices, LifxController};

#[get("/lights?<update>")]
fn get_lights(controller: State<LifxController>, update: bool) -> Json<Devices> {
    if update {
        Json(controller.update())
    } else {
        Json(controller.get_lights())
    }
}

#[delete("/lights")]
fn delete_lights(controller: State<LifxController>) {
    controller.delete_lights();
}

#[post("/lights/toggle", data = "<form>")]
fn toggle_lights(controller: State<LifxController>, form: Option<Form<Duration>>) {
    match form {
        Some(duration) => controller.toggle(duration.0.duration),
        None => controller.toggle(0),
    }
}

#[post("/lights/on", data = "<form>")]
fn lights_on(controller: State<LifxController>, form: Option<Form<Duration>>) {
    match form {
        Some(duration) => controller.on(duration.0.duration),
        None => controller.on(0),
    }
}

#[post("/lights/off", data = "<form>")]
fn lights_off(controller: State<LifxController>, form: Option<Form<Duration>>) {
    match form {
        Some(duration) => controller.off(duration.0.duration),
        None => controller.off(0),
    }
}

#[post("/lights/brightness", data = "<form>")]
fn lights_brightness(controller: State<LifxController>, form: Form<Brightness>) {
    let brightness = form.0.brightness;
    match form.0.duration {
        Some(duration) => controller.set_brightness(brightness, duration),
        None => controller.set_brightness(brightness, 0),
    }
}

#[post("/lights/temperature", data = "<form>")]
fn lights_temperature(controller: State<LifxController>, form: Form<Temperature>) {
    let kelvin = form.0.kelvin;
    match form.0.duration {
        Some(duration) => controller.set_temperature(kelvin, duration),
        None => controller.set_temperature(kelvin, 0),
    }
}

#[patch("/light/state", data = "<form>")]
fn update_lights(controller: State<LifxController>, form: Form<Hsb>) {
    dbg!(&form);
    let hue = form.0.hue;
    let saturation = form.0.saturation;
    let brightness = form.0.brightness;
    let duration = form.0.duration;
    controller.update_lights(hue, saturation, brightness, duration);
}

fn main() {
    let config = Config::build(Environment::Staging)
        .address("0.0.0.0")
        .finalize()
        .unwrap();

    rocket::custom(config)
        .manage(LifxController::new())
        .mount(
            "/",
            routes![
                get_lights,
                delete_lights,
                toggle_lights,
                lights_on,
                lights_off,
                lights_brightness,
                lights_temperature,
                update_lights,
            ],
        )
        .launch();
}
