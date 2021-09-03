#[macro_use]
extern crate rocket;

mod config;
mod controller;
mod forms;

use std::{fs::File, io};

use forms::{Brightness, Duration, HsbkDuration, Preset, Selector, Temperature};
use io::{ErrorKind, Read};
use rocket::{form::Form, http::Status, response::content::Json, response::status, State};

use controller::{Devices, LifxController, Presets};

#[get("/lights?<update>")]
fn get_lights(
    controller: &State<LifxController>,
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
fn delete_lights(controller: &State<LifxController>) -> Result<(), status::Custom<String>> {
    controller
        .delete_lights()
        .map_err(|e| status::Custom(Status::InternalServerError, e.to_string()))
}

#[post("/lights/<selector>/toggle", data = "<form>")]
fn toggle_lights(
    controller: &State<LifxController>,
    selector: String,
    form: Form<Duration>,
) -> Status {
    let selector = Selector::parse(&selector);
    let duration = form.duration.unwrap_or(0);
    controller
        .toggle(selector, duration)
        .map_or(Status::InternalServerError, |_| Status::NoContent)
}

#[post("/lights/<selector>/on", data = "<form>")]
fn lights_on(
    controller: &State<LifxController>,
    selector: String,
    form: Form<Duration>,
) -> Result<(), status::Custom<String>> {
    let selector = Selector::parse(&selector);
    let duration = form.duration.unwrap_or(0);
    controller
        .on(selector, duration)
        .map_err(|e| status::Custom(Status::InternalServerError, e.to_string()))
}

#[post("/lights/<selector>/off", data = "<form>")]
fn lights_off(
    controller: &State<LifxController>,
    selector: String,
    form: Form<Duration>,
) -> Result<(), status::Custom<String>> {
    let selector = Selector::parse(&selector);
    let duration = form.duration.unwrap_or(0);
    controller
        .off(selector, duration)
        .map_err(|e| status::Custom(Status::InternalServerError, e.to_string()))
}

#[post("/lights/<selector>/brightness", data = "<form>")]
fn lights_brightness(
    controller: &State<LifxController>,
    selector: String,
    form: Form<Brightness>,
) -> Result<(), status::Custom<String>> {
    let selector = Selector::parse(&selector);
    let brightness = form.brightness;
    let duration = form.duration.unwrap_or(0);
    controller
        .set_brightness(selector, brightness, duration)
        .map_err(|e| status::Custom(Status::InternalServerError, e.to_string()))
}

#[post("/lights/<selector>/temperature", data = "<form>")]
fn lights_temperature(
    controller: &State<LifxController>,
    selector: String,
    form: Form<Temperature>,
) -> Result<(), status::Custom<String>> {
    let selector = Selector::parse(&selector);
    let kelvin = form.kelvin;
    let duration = form.duration.unwrap_or(0);
    controller
        .set_temperature(selector, kelvin, duration)
        .map_err(|e| status::Custom(Status::InternalServerError, e.to_string()))
}

#[patch("/lights/<selector>/state", data = "<form>")]
fn update_lights(
    controller: &State<LifxController>,
    selector: String,
    form: Form<HsbkDuration>,
) -> Result<(), status::Custom<String>> {
    let selector = Selector::parse(&selector);
    let hue = form.hue;
    let saturation = form.saturation;
    let brightness = form.brightness;
    let kelvin = form.kelvin;
    let duration = form.duration.unwrap_or(0);
    controller
        .update_lights(selector, hue, saturation, brightness, kelvin, duration)
        .map_err(|e| status::Custom(Status::InternalServerError, e.to_string()))
}

#[get("/presets")]
fn get_presets(
    controller: &State<LifxController>,
) -> Result<Json<Presets>, status::Custom<String>> {
    let presets = controller
        .presets()
        .map_err(|e| status::Custom(Status::InternalServerError, e.to_string()))?;
    Result::Ok(Json(presets))
}

#[put("/presets/<label>", format = "json", data = "<preset>")]
fn set_preset(
    controller: &State<LifxController>,
    label: String,
    preset: rocket::serde::json::Json<Preset>,
) -> Result<(), Status> {
    controller
        .set_preset(label, preset.0)
        .map_err(|_| Status::InternalServerError)
}

#[post("/presets/<label>")]
fn execute_preset(
    controller: &State<LifxController>,
    label: String,
) -> Result<(), status::Custom<String>> {
    controller
        .execute_preset(label)
        .map_err(|e| status::Custom(Status::InternalServerError, e.to_string()))
}

#[launch]
fn rocket() -> rocket::Rocket<rocket::Build> {
    rocket::build().manage(new_controller().unwrap()).mount(
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
}

fn new_controller() -> controller::Result<LifxController> {
    match File::open("Lifx.toml") {
        Ok(mut lifx_toml) => {
            let mut buf = String::new();
            lifx_toml.read_to_string(&mut buf)?;
            let config = toml::from_str(&buf)?;
            LifxController::from_config(config)
        }
        Err(ref e) if e.kind() == ErrorKind::NotFound => LifxController::new(),
        Err(e) => controller::Result::Err(controller::Error(e.to_string())),
    }
}
