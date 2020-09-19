#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

mod config;
mod controller;
mod forms;

use std::{fs::File, io, collections::HashMap};

use forms::{Brightness, Duration, Temperature, Selector, Hsbk, Preset};
use io::{ErrorKind, Read};
use rocket::{config::Environment, request::Form, response::content::Json, Config, State};

use controller::{Devices, LifxController, Presets};

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

#[post("/lights/<selector>/toggle", data = "<form>")]
fn toggle_lights(controller: State<LifxController>, selector: String, form: Option<Form<Duration>>) {
    let selector = Selector::parse(&selector);
    let duration = match form {
        Some(duration) => duration.0.duration,
        None => 0,
    };
    controller.toggle(selector, duration);
}

#[post("/lights/<selector>/on", data = "<form>")]
fn lights_on(controller: State<LifxController>, selector: String, form: Option<Form<Duration>>) {
    let selector = Selector::parse(&selector);
    let duration = match form {
        Some(duration) => duration.0.duration,
        None => 0,
    };
    controller.on(selector, duration);
}

#[post("/lights/<selector>/off", data = "<form>")]
fn lights_off(controller: State<LifxController>, selector: String, form: Option<Form<Duration>>) {
    let selector = Selector::parse(&selector);
    let duration = match form {
        Some(duration) => duration.0.duration,
        None => 0,
    };
    controller.off(selector, duration);
}

#[post("/lights/<selector>/brightness", data = "<form>")]
fn lights_brightness(controller: State<LifxController>, selector: String, form: Form<Brightness>) {
    let selector = Selector::parse(&selector);
    let brightness = form.0.brightness;

    let duration = if let Some(d) = form.0.duration {
        d
    } else {
        0
    };
    controller.set_brightness(selector, brightness, duration);
}

#[post("/lights/<selector>/temperature", data = "<form>")]
fn lights_temperature(controller: State<LifxController>, selector: String, form: Form<Temperature>) {
    let selector = Selector::parse(&selector);
    let kelvin = form.0.kelvin;
    let duration = if let Some(d) = form.0.duration {
        d
    } else {
        0
    };
    controller.set_temperature(selector, kelvin, duration);
}

#[patch("/lights/<selector>/state", data = "<form>")]
fn update_lights(controller: State<LifxController>, selector: String, form: Form<Hsbk>) {
    let selector = Selector::parse(&selector);
    let hue = form.0.hue;
    let saturation = form.0.saturation;
    let brightness = form.0.brightness;
    let kelvin = form.0.kelvin;
    let duration = form.0.duration;
    controller.update_lights(selector, hue, saturation, brightness, kelvin, duration);
}

#[get("/presets")]
fn get_presets(controller: State<LifxController>) -> Json<Presets> {
    Json(controller.presets())
}

#[put("/presets/<label>", format = "json", data = "<preset>")]
fn set_preset(controller: State<LifxController>, label: String, preset: rocket_contrib::json::Json<Preset>) {
    controller.set_preset(preset.0);
}

#[post("/presets/<label>")]
fn execute_preset(controller: State<LifxController>, label: String) {
    controller.execute_preset(label);
}

fn main() -> io::Result<()> {
    let controller = match File::open("Lifx.toml") {
        Ok(mut lifx_toml) => {
            let mut buf = String::new();
            lifx_toml.read_to_string(&mut buf)?;
            LifxController::from_config(toml::from_str(&buf)?)
        }
        Err(ref e) if e.kind() == ErrorKind::NotFound => LifxController::new(),
        Err(e) => return Result::Err(e),
    };

    let config = Config::build(Environment::Staging)
        .address("0.0.0.0")
        .finalize()
        .unwrap();

    rocket::custom(config)
        .manage(controller)
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
                get_presets,
                set_preset,
                execute_preset,
            ],
        )
        .launch();

    Result::Ok(())
}
