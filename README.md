# Rialight

> <b>NOTE</b><br>NOT READY; [TASKS](/.tasks.md)
> <br>

Build softwares and games robustly with the Rust language. Softwares are also called rich-interface applications (RIAs).

Rialight covers all the basic needs for you, such as creating new projects, packaging the app for the desired platforms and providing graphics, UI, game engine, sound, localization, file system and network capabilities. Graphics and UI are combined into a single module, `rialight::display`, and `DisplayNode` is the core type for UI controls, 2D vector graphics and 3D objects.

## Getting started

Install Rialight SDK with the following command:

```
cargo install rialight_sdk_cli
```

Create a new project with:

```
rialight new my-project-name
```

Debug the project with:

```
cd my-project-name
cargo run
```

## Documentation

[Consult the documentation](./docs/README.md)