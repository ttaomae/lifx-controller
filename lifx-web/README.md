# LIFX Web
This is a web service which exposes REST endpoints to control LIFX devices on a LAN network.

## API
### Find lights
#### `GET /lights?<update>`
Returns a JSON object listing all lights known to the application.

If `update=true`, it will attempt to find new devices.

#### `DELETE /lights`
Forgets all lights currently known to the application.

### Control Lights
There are various endpoints which control lights. Each of these endpoints have a *selector*, which
identifies which lights will be affected. There are four different types of selector.
* `label:<name>` - Select a specifc light with a particular label.
* `group:<name>` - Select all lights which belong to a particular group.
* `location:<name>` - Select all lights which are in a particular location.
* `all` - Select all lights.

These control endpoints accept `Content-Type: application/x-www-form-urlencoded` data. Each endpoint
minimally accepts an optional `duration` parameter. This parameter is the transition time in
milliseconds. It can be omitted to transition instantly.

#### `POST /lights/<selector>/toggle`
Toggles the power state (on/off) of the selected lights.

#### `POST /lights/<selector>/on`
Turns the selected lights on.

#### `POST /lights/<selector>/off`
Turns the selected lights off.

#### `POST /lights/<selector>/brightness`
Sets the brightness of the selected lights.
##### Additional Parameters
* `brightness` - A number between 0.0 (0%) and 1.0 (100%) representing the target brightness.

#### `POST /lights/<selector>/temperature`
Changes the selected lights to a white light (zero saturation) with the specified temperature.
##### Additional Parameters
* `kelvin` - An integer between 2500 and 9000 representing the target temperature in Kelvin.

#### `PATCH /lights/<selector>/state`
Changes the color state of the selected lights by applything the specified parameters. Unspecified
parameters will remain the same.

LIFX uses a combination of a
[HSB (hue, saturation, brightness)](https://en.wikipedia.org/wiki/HSL_and_HSV) and
[color temperature](https://en.wikipedia.org/wiki/Color_temperature) model. Although LIFX devices
treat these as a single model, this endpoint treats them as two separate controls. You can either
control the HSB value or the temperature (and brightness) with preference given to HSB. If the hue
and/or saturation is specified, then the temperature is ignored.

##### Additional Parameters
* `hue` - A number between 0.0 and 360.0 representing the target hue in degrees.
* `saturation` - A number between 0.0 (0%) and 1.0 (100%) representing the target saturation.
* `brightness` - A number between 0.0 (0%) and 1.0 (100%) representing the target brightness.
* `kelvin` - An integer between 2500 and 9000 representing the target temperature in Kelvin.

### Presets
You can create and execute named presets. A preset consists of one or more actions which are
equivalent to performing a `PATCH /lights/<selector>/state`.

#### `GET /presets`
Returns a JSON object describing all known presets.

Below is an informal description of the returned object.
```
{
  "presets": {
    "<label-1>": {
      "actions": [
        {
          "selector": "string",
          "duration": "integer",
          "hsbk": {
            "hue": "number",
          }
        },
        { additional action },
        ...
      ]
    },
    "<label-2>": {
      "actions": [ ... ]
    },
    ...
  }
}
```

#### `PUT /presets/<label>`
Creates or replaces a preset with the specified label.

Accepts `Content-Type: application/json` with the following structure:
```
{
  "actions": [
    {
      "selector": "string",
      "duration": "integer",
      "hsbk": {
        "hue": "number",
      }
    },
    { additional action },
    ...
    ]
},
```

#### `POST /presets/<label>`
Executes the preset with the specified label.

## `Lifx.toml`
You can pre-configure devices and presets using a `Lifx.toml` file. Below is an informal description
of the format. You can run `cargo run --example find_and_save devices.txt` from the root of the
project to find and save a list of devices in the required format.

```toml
devices = [
    "<mac-address>#<ip-address:port>",
    ...
]

# Create a preset for <label>.
[[presets.<label>.actions]]
# Selector is required.
selector = '<selector>'
# Other values are optional depending on desired action.
hsbk.hue = <hue>
hsbk.saturation = <saturation>
hsbk.brightness = <brightness>
hsbk.kelvin = <kelvin>
duration = <duration>

# Repeat the same label to perform multiple actions.
[[presets.<label>.actions]]
...
[[presets.<label>.actions]]
...
```

## Build and Run
This application is built with [Rocket](https://rocket.rs/) 0.4.x and requires a nightly version of
Rust. Run `cargo build --release` from the current directory to build the application or run
`cargo build --bin lifx-web --release` from the root of the project.

```
cargo build --release
```

The executable will be available at `../target/release/lifx-web`. Run the executable to launch the
application. It will look for a `Lifx.toml` file in the current directory.