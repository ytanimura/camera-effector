# Camera Effector

**This project is for myself and not intended for your use.**

## What is this?

This is a tool to edit the image acquired from the camera and output it to a window; if you want to use it with Zoom or something, you can do it successfully with OBS.

## How to use?

At first, run `cargo run -- --init` for initialize app.
**The camera resolution, frame format and frame rate have to be manually placed in the generated `camera_setting.json`.**
After setting, run the app by `cargo run --release`.

## If you don't like Windows (or are forced to use other OS)

The default configuration only works on Windows. The backend configuration of `nokhwa` needs to be changed.
For example, on MacOS, the feature of `nokhwa` have to be replaced by `input-avfoundation`.

## Edit shader

The run shader is `src/shader.frag`. Shaders are written in Shadertoy syntax. `iTime` and `iResolution` are available, mouse input is not supported.
The camera is `iChannel0` and no other textures.
