#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;

mod config;
mod controller;
mod forms;

use std::{fs::File, io};

use forms::{Brightness, Duration, HsbkDuration, Preset, Selector, Temperature};
use io::{ErrorKind, Read};
use rocket::{
    config::Environment, http::Status, request::Form, response::content::Json, response::status,
    Config, State,
};

use controller::{Devices, LifxController, Presets};

#[get("/lights?<update>")]
fn get_lights(
    controller: State<LifxController>,
    update: bool,
) -> Result<Json<Devices>, status::Custom<String>> {
    let device_result = if update {
        controller.update()
    } else {
        controller.get_lights()
    };

    match device_result {
        Ok(devices) => Result::Ok(Json(devices)),
        Err(e) => Result::Err(status::Custom(Status::InternalServerError, e.to_string())),
    }
}

#[delete("/lights")]
fn delete_lights(controller: State<LifxController>) -> Result<(), status::Custom<String>> {
    controller
        .delete_lights()
        .map_err(|e| status::Custom(Status::InternalServerError, e.to_string()))
}

#[post("/lights/<selector>/toggle", data = "<form>")]
fn toggle_lights(
    controller: State<LifxController>,
    selector: String,
    form: Form<Duration>,
) -> Status {
    let selector = Selector::parse(&selector);
    let duration = form.0.duration.unwrap_or(0);
    controller
        .toggle(selector, duration)
        .map_or(Status::InternalServerError, |_| Status::NoContent)
}

#[post("/lights/<selector>/on", data = "<form>")]
fn lights_on(
    controller: State<LifxController>,
    selector: String,
    form: Form<Duration>,
) -> Result<(), status::Custom<String>> {
    let selector = Selector::parse(&selector);
    let duration = form.0.duration.unwrap_or(0);
    controller
        .on(selector, duration)
        .map_err(|e| status::Custom(Status::InternalServerError, e.to_string()))
}

#[post("/lights/<selector>/off", data = "<form>")]
fn lights_off(
    controller: State<LifxController>,
    selector: String,
    form: Form<Duration>,
) -> Result<(), status::Custom<String>> {
    let selector = Selector::parse(&selector);
    let duration = form.0.duration.unwrap_or(0);
    controller
        .off(selector, duration)
        .map_err(|e| status::Custom(Status::InternalServerError, e.to_string()))
}

#[post("/lights/<selector>/brightness", data = "<form>")]
fn lights_brightness(
    controller: State<LifxController>,
    selector: String,
    form: Form<Brightness>,
) -> Result<(), status::Custom<String>> {
    let selector = Selector::parse(&selector);
    let brightness = form.0.brightness;
    let duration = form.0.duration.unwrap_or(0);
    controller
        .set_brightness(selector, brightness, duration)
        .map_err(|e| status::Custom(Status::InternalServerError, e.to_string()))
}

#[post("/lights/<selector>/temperature", data = "<form>")]
fn lights_temperature(
    controller: State<LifxController>,
    selector: String,
    form: Form<Temperature>,
) -> Result<(), status::Custom<String>> {
    let selector = Selector::parse(&selector);
    let kelvin = form.0.kelvin;
    let duration = form.0.duration.unwrap_or(0);
    controller
        .set_temperature(selector, kelvin, duration)
        .map_err(|e| status::Custom(Status::InternalServerError, e.to_string()))
}

#[patch("/lights/<selector>/state", data = "<form>")]
fn update_lights(
    controller: State<LifxController>,
    selector: String,
    form: Form<HsbkDuration>,
) -> Result<(), status::Custom<String>> {
    let selector = Selector::parse(&selector);
    let hue = form.0.hue;
    let saturation = form.0.saturation;
    let brightness = form.0.brightness;
    let kelvin = form.0.kelvin;
    let duration = form.0.duration.unwrap_or(0);
    controller
        .update_lights(selector, hue, saturation, brightness, kelvin, duration)
        .map_err(|e| status::Custom(Status::InternalServerError, e.to_string()))
}

#[get("/presets")]
fn get_presets(controller: State<LifxController>) -> Result<Json<Presets>, status::Custom<String>> {
    let presets = controller
        .presets()
        .map_err(|e| status::Custom(Status::InternalServerError, e.to_string()))?;
    Result::Ok(Json(presets))
}

#[put("/presets/<label>", format = "json", data = "<preset>")]
fn set_preset(
    controller: State<LifxController>,
    label: String,
    preset: rocket_contrib::json::Json<Preset>,
) -> Result<(), Status> {
    controller
        .set_preset(label, preset.0)
        .map_err(|_| Status::InternalServerError)
}

#[post("/presets/<label>")]
fn execute_preset(
    controller: State<LifxController>,
    label: String,
) -> Result<(), status::Custom<String>> {
    controller
        .execute_preset(label)
        .map_err(|e| status::Custom(Status::InternalServerError, e.to_string()))
}

fn main() -> controller::Result<()> {
    let controller = match File::open("Lifx.toml") {
        Ok(mut lifx_toml) => {
            let mut buf = String::new();
            lifx_toml.read_to_string(&mut buf)?;
            let config = toml::from_str(&buf).map_err(|_| "".to_string())?;
            LifxController::from_config(config)
        }
        Err(ref e) if e.kind() == ErrorKind::NotFound => LifxController::new(),
        Err(e) => return Result::Err(controller::Error(e.to_string())),
    }?;

    let config = Config::build(Environment::Staging)
        .address("0.0.0.0")
        .finalize()
        .map_err(|e| e.to_string())?;

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
                execute_preset
            ],
        )
        .launch();

    Result::Ok(())
}
