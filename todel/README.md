# Todel

The Derailed models & shared logic crate.

## Usage

This crate was made with the idea of users directly depending on it in their
projects with mind, add it to your own project either by:

- Using `cargo add`

  ```sh
  cargo add todel
  ```

- Adding the following line to your `Cargo.toml`

  ```toml
  todel = "*"
  ```

## Features

Todel comes with 2 main features, `logic` which provides the logic-related implementation
to the models and also some types which are not used directly by the Derailed API
and `http` which houses the rocket-reliant logic and models.

If you just need the models (which will be the case most of the time) then you
can add todel with no extra features, all the models derive the `Debug`, `serde::Serialize`
and `serde::Deserialize` traits.
