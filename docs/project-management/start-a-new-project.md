# Start a new project

Run the following command to create a new project:

```
rialight new foo-app
```

The project structure should look as follows:

```
foo-app
|__ res
| |____ lang
| |    |___ en-US
| |         |__ _.json
|__ src
| |____ localization.rs
| |____ main.rs
|__ .gitignore
|__ Cargo.toml
|__ Rialight.toml
|__ README.md
|__ build.rs
```

When building the project, you may notice the following additional files and directories. Do not touch these.

```
foo-app
|__ target
|__ Cargo.lock
```